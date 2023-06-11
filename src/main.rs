use serde::Deserialize;
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
#[cfg(target_family = "windows")]
use tokio::signal;
#[cfg(target_family = "unix")]
use tokio::signal::{
    self,
    unix::{signal, SignalKind},
};
use tokio::{select, time};
use tokio_trace::{debug, info, warn};
use tokio_util::sync::CancellationToken;
use twitter_v2::{authorization::Oauth1aToken, query::UserField, TwitterApi};

#[derive(Debug, Deserialize)]
struct Oauth1Fields {
    pub consumer_key: String,
    consumer_secret: String,
    token: String,
    secret: String,
}

async fn do_work_interval(
    auth: Oauth1aToken,
    pagination_token: &Option<String>,
    pool: &SqlitePool,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // We can't get away [400 Bad Request] with null length &str as next_token so:

    // 1) let's prepare common query parameters.
    let field_array = [
        UserField::Id,
        UserField::Username,
        UserField::Name,
        UserField::Verified,
        //UserField::Location,
        UserField::CreatedAt,
        //UserField::Description,
        //UserField::PublicMetrics,
        //UserField::ProfileImageUrl,
    ];
    let max = 1000;

    // 2)   code separate queries, depending on presence of pagination token
    //      for looking up followers in user's context.
    let api_response;

    if let Some(next_token) = pagination_token {
        api_response = TwitterApi::new(auth)
        .with_user_ctx()
        .await?
        //.expect("Schnauzfall bim konteggscht")
        .get_my_followers()
        .user_fields(field_array)
        .max_results(max)
        .pagination_token(next_token)
        .send()
        .await?
        //.expect("Schnauzfall bi dä follis")
        ;
    } else {
        api_response = TwitterApi::new(auth)
        .with_user_ctx()
        .await?
        //.expect("Schnauzfall bim konteggscht")
        .get_my_followers()
        .user_fields(field_array)
        .max_results(max)
        .send()
        .await?
        //.expect("Schnauzfall bi dä follis")
        ;
    }

    // De-structure API metadata and get next_token.
    let next_token: Option<String>;
    if let Some(meta) = &api_response.meta {
        next_token = meta.next_token.as_ref().map(String::from);
        info!(
            "result_count: {}, previous_token: {:?}",
            meta.result_count, &meta.previous_token
        );
        println!(
            "result_count: {}, previous_token: {:?}",
            meta.result_count, &meta.previous_token
        );
    } else {
        next_token = None;
    }

    let my_followers = api_response.into_data();

    // Zero followers will be `None`, so we don't even enter here.
    if let Some(my_followers) = my_followers {
        //
        // Write persistence meta entry when we start updating.
        if pagination_token.is_none() {
            sqlx::query("UPDATE meta SET curr_start = DATETIME('now')")
                .execute(pool)
                .await?;
        }

        // for statistics
        let fetches = my_followers.len();
        let mut inserts = 0;
        let mut updates = 0;

        for follower in my_followers {
            // persistence (SQLite)
            println!(
                "{:20} {:?} {}",
                follower.username,
                follower.verified,
                follower.created_at.unwrap()
            );

            let id_string = follower.id.as_u64().to_string();
            let user_handle = follower.username;
            let user_name = follower.name;

            let is_new = sqlx::query(
                "
                SELECT id FROM follow
                WHERE id == $1
                AND last_seen >= (SELECT last_start FROM meta)
                ",
            )
            .bind(&id_string)
            .fetch_optional(pool) // at most one
            .await?
            .is_none();

            if is_new {
                sqlx::query(
                    "INSERT INTO follow (
                        id,
                        first_seen,
                        last_seen,
                        last_handle_seen,
                        last_name_seen
                    )
                    VALUES ($1, DATETIME('now'), DATETIME('now'), $2, $3)
                    ",
                )
                .bind(&id_string)
                .bind(&user_handle)
                .bind(&user_name)
                .execute(pool)
                .await?;
                inserts += 1;
            } else {
                sqlx::query(
                    "
                    UPDATE follow SET
                        last_seen = DATETIME('now'),
                        last_handle_seen = $2,
                        last_name_seen = $3
                    WHERE id == $1
                    AND last_seen >= (SELECT last_start FROM meta)
                    ",
                )
                .bind(&id_string)
                .bind(&user_handle)
                .bind(&user_name)
                .execute(pool)
                .await?;
                updates += 1;
            }
        }

        info!("api fetches: {fetches}, db inserts: {inserts}, db updates: {updates}");
        println!("api fetches: {fetches}, db inserts: {inserts}, db updates: {updates}");

        // Write persistence meta entry when we're done updating.
        if next_token.is_none() {
            sqlx::query(
                "
                UPDATE meta SET last_start = (
                    SELECT curr_start FROM meta
                );
                UPDATE meta SET curr_start = NULL;
                UPDATE meta SET last_end = DATETIME('now');
                UPDATE meta SET initial_end = DATETIME('now') WHERE initial_end is NULL
                ",
            )
            .execute(pool)
            .await?;
        }
    };

    Ok(next_token)
}

const DB_URL: &str = "sqlite://db.sl3";

#[tokio::main]
async fn main() {
    // Get authorization:
    let creds =
        serde_json::from_str::<Oauth1Fields>(include_str!("../credentials/Oauth1WmfLeft.json"))
            .expect("Oauth1 token d3s3r1al1Ze tr0ubl3");

    let auth = Oauth1aToken::new(
        creds.consumer_key,
        creds.consumer_secret,
        creds.token,
        creds.secret,
    );

    // Retrieve Twitter user id for checks.
    let data = TwitterApi::new(auth.clone())
        .get_users_me()
        .send()
        .await
        .expect("Couldn't verify Oauth1a creds")
        .into_data();

    let my_id;

    if let Some(user) = data {
        info!("Starting for @{1} ({0})", user.id, user.username);
        my_id = user.id.as_u64().to_string();
    } else {
        panic!("Serious #Twitterfail, can't continue...");
    }

    // Prepare persistence layer.
    let db_existed = Sqlite::database_exists(DB_URL).await.unwrap_or(false);

    if !db_existed {
        info!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => debug!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(20)
        //.min_connections(10)
        .connect(DB_URL)
        .await
        .expect("DB failed creating connection pool");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("DB failed running migrate");

    if !db_existed {
        // Ownership initialization is required.
        sqlx::query("INSERT INTO meta (owner) VALUES ($1)")
            .bind(my_id)
            .execute(&pool)
            .await
            .unwrap();
    } else {
        // Ownership check is required.
        sqlx::query("SELECT owner FROM meta WHERE owner == $1")
            .bind(&my_id)
            .fetch_one(&pool)
            .await
            .unwrap_or_else(|_| panic!("Owner of persistence store != {my_id}, can't continue..."));
    }

    // Start doing work.
    let mut interval = time::interval(time::Duration::from_secs(60));
    let mut next: Option<String> = None; // next_token

    debug!("{:#?}", pool.options());

    // Prepare tasks.
    let loops_token = CancellationToken::new();
    let loop_forever1 = loops_token.clone();

    #[cfg(target_family = "unix")]
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    #[cfg(target_family = "unix")]
    let mut sighup = signal(SignalKind::hangup()).unwrap();

    let forever1 = tokio::spawn(async move {
        loop {
            #[cfg(target_family = "windows")]
            select! {
                    _ = loop_forever1.cancelled() => {
                        // Cleanup work at end is done here.
                        //
                        // TODO: but what if we have several loops?
                        pool.close().await;
                        info!("SIGINT... closed DB");
                        break;
                    }
                     _ = interval.tick() => do_work(auth.clone(), &pool, &mut next).await,
            }
            #[cfg(target_family = "unix")]
            select! {
                    _ = loop_forever1.cancelled() => {
                        // Cleanup work at end is done here.
                        //
                        // TODO: but what if we have several loops?
                        pool.close().await;
                        info!("SIGINT... closed DB");
                        break;
                    }
                    _ = sigterm.recv() => {
                        pool.close().await;
                        info!("SIGTERM... closed DB");
                        break;
                    }
                    _ = sighup.recv() => {
                        pool.close().await;
                        info!("SIGHUP... closed DB");
                        break;
                    }
                     _ = interval.tick() => do_work(auth.clone(), &pool, &mut next).await,
            }
        }
    });

    async fn do_work(auth: Oauth1aToken, pool: &SqlitePool, next: &mut Option<String>) {
        let result = do_work_interval(auth, next, pool).await;
        //
        // All errors treated here...
        if let Err(e) = result {
            /*
                TODO: so far (stupidly) we don't know how to match here!

                awful!!!    Api(ApiError {
                                title: "Unauthorized",
                                kind: "about:blank",
                                status: 401,
                                detail: "Unauthorized", errors: []
                            })

                            ERROR: [403 Forbidden]
                                When authenticating requests to the Twitter API v2 endpoints,
                                you must use keys and tokens from a Twitter developer App that is attached to a Project.
                                You can create a project via the developer portal.

                benign :)   Request(reqwest::Error {
                                kind: Request, url: Url {
                                    scheme: "https",
                                    cannot_be_a_base: false,
                                    username: "",
                                    password: None,
                                    host: Some(Domain("api.twitter.com")),
                                    port: None, path: "/2/users/me",
                                    query: None,
                                    fragment: None
                                },
                                source: hyper::Error(Connect, ConnectError("tcp connect error", Os {
                                    code: 101,
                                    kind: NetworkUnreachable,
                                    message: "Network is unreachable"
                                }))
                            })

                            Api(ApiError {
                                title: "Too Many Requests",
                                kind: "about:blank",
                                status: 429,
                                detail: "Too Many Requests", errors: []
                            })

                Possible solution using the `thiserror` crate with '?-operator' and `tokio`?
            */
            warn!("ERROR: {e}");
            eprintln!("ERROR: {e:?}");
        } else {
            //
            // Manage next_token, be it empty or not.
            *next = match result {
                Ok(t) => t,
                Err(_) => None,
            };
        }
    }
    // Install this handler in the background.
    tokio::spawn(async move {
        signal::ctrl_c()
            .await
            .expect("Failed to listen for SIGINT/ctrl_c");
        loops_token.cancel();
    });

    forever1.await.unwrap();
}

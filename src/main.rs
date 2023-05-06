use serde::Deserialize;
use tokio::time;
use twitter_v2::{authorization::Oauth1aToken, query::UserField, TwitterApi};

#[derive(Debug, Deserialize)]
struct Oauth1Fields {
    pub consumer_key: String,
    consumer_secret: String,
    token: String,
    secret: String,
}

async fn task_that_takes_a_second(auth: Oauth1aToken) -> Result<(), Box<dyn std::error::Error>> {
    // Try lookin' up followers in user's context.
    let my_followers = TwitterApi::new(auth)
        .with_user_ctx()
        .await?
        //.expect("Schnauzfall bim konteggscht")
        .get_my_followers()
        .user_fields([UserField::Id, UserField::Username])
        .send()
        .await?
        //.expect("Schnauzfall bi dä follis")
        .into_data();

    // 0 follies will be `None` and not enter here.
    if let Some(my_followers) = my_followers {
        let subtot = my_followers.len();
        for follower in my_followers {
            println!(
                "{:<32} https://twitter.com/i/user/{}",
                follower.username,
                follower.id.as_u64()
            );
        }
        println!(" --total: {subtot}\n");
    };

    Ok(())
}

#[tokio::main]
async fn main() {
    // Get authorization:
    let creds = serde_json::from_str::<Oauth1Fields>(include_str!(
        "../../ferristw2/Oauth1UsrCtxRight.json"
    ))
    .expect("Oauth1 token d3s3r1al1Ze tr0ubl3");

    let auth = Oauth1aToken::new(
        creds.consumer_key,
        creds.consumer_secret,
        creds.token,
        creds.secret,
    );

    // Start doing the work.
    let mut interval = time::interval(time::Duration::from_secs(30));
    loop {
        interval.tick().await;
        if let Err(e) = task_that_takes_a_second(auth.clone()).await {
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
            eprintln!("ERROR: {e}");
        }
    }
}

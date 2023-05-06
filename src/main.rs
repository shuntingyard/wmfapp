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

async fn do_work_interval(
    auth: Oauth1aToken,
    pagination_token: &Option<String>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // We can't get away [400 Bad Request] with null length &str as next_token so:

    // 1) let's prepare common query parms.
    let field_array = [UserField::Id, UserField::Username];
    let max = 30;

    // 2)   code separate queries, depending on presence of pagination token
    //      for lookin' up followers in user's context.
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

    // Destructure metadata and get next_token.
    let next_token: Option<String>;
    if let Some(meta) = &api_response.meta {
        next_token = match &meta.next_token {
            Some(t) => Some(String::from(t)),
            _ => None,
        };
        println!(
            "result_count: {}, previous_token: {:?}",
            meta.result_count, &meta.previous_token
        );
    } else {
        next_token = None;
    }

    let my_followers = api_response.into_data();

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

    Ok(next_token)
}

#[tokio::main]
async fn main() {
    // Get authorization:
    let creds =
        serde_json::from_str::<Oauth1Fields>(include_str!("../../ferristw2/Oauth1UsrCtxLeft.json"))
            .expect("Oauth1 token d3s3r1al1Ze tr0ubl3");

    let auth = Oauth1aToken::new(
        creds.consumer_key,
        creds.consumer_secret,
        creds.token,
        creds.secret,
    );

    // Start doing the work.
    let mut interval = time::interval(time::Duration::from_secs(30));
    let mut next: Option<String> = None; // next_token

    loop {
        interval.tick().await;
        let result = do_work_interval(auth.clone(), &next).await;
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
            eprintln!("ERROR: {e}");
        } else {
            //
            // Manage next_token, be it empty or not.
            next = match result {
                Ok(t) => t,
                Err(_) => None,
            };
        }
    }
}

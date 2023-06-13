use std::{env, fs, path::PathBuf};

use serde::Deserialize;
use twitter_v2::authorization::Oauth1aToken;
use twitter_v2::{query::UserField, TwitterApi};

#[derive(Debug, Deserialize)]
struct Oauth1Fields {
    consumer_key: String,
    consumer_secret: String,
    oauth_token: String,
    oauth_token_secret: String,
}

#[tokio::main]
async fn main() {
    let credfile: PathBuf = env::args().nth(1).expect("Arg1: Oauth1 creds json").into();
    let credfile = fs::read_to_string(credfile).unwrap();
    let creds =
        serde_json::from_str::<Oauth1Fields>(&credfile).expect("Oauth1 token d3s3r1al1Ze tr0ubl3");

    let auth = Oauth1aToken::new(
        creds.consumer_key,
        creds.consumer_secret,
        creds.oauth_token,
        creds.oauth_token_secret,
    );
    do_tests(auth).await;
}

async fn do_tests(auth: Oauth1aToken) {
    let my_followers = TwitterApi::new(auth)
        .with_user_ctx()
        .await
        .unwrap()
        .get_my_followers()
        .user_fields([UserField::Username])
        .max_results(20)
        .send()
        .await
        .unwrap()
        .into_data();

    dbg!({ my_followers });
}

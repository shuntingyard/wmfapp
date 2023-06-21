use std::{env, fs, path::PathBuf};

use reqwest::Client;
use serde::Deserialize;
use twitter_api_v1::{endpoints::users::lookup::show_user_by_screen_name, TokenSecrets};
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
    let api_version: u8 = env::args()
        .nth(1)
        .expect("Arg1: 1 or 2 for API version")
        .parse()
        .expect("Arg1: bad value");
    let credfile: PathBuf = env::args().nth(2).expect("Arg2: Oauth1 creds json").into();
    let credfile = fs::read_to_string(credfile).unwrap();
    let creds =
        serde_json::from_str::<Oauth1Fields>(&credfile).expect("Oauth1 token d3s3r1al1Ze tr0ubl3");

    match api_version {
        1 => {
            let auth = TokenSecrets::new(
                creds.consumer_key,
                creds.consumer_secret,
                creds.oauth_token,
                creds.oauth_token_secret,
            );
            do_tests_v1_1(&auth).await;
        }
        2 => {
            let auth = Oauth1aToken::new(
                creds.consumer_key,
                creds.consumer_secret,
                creds.oauth_token,
                creds.oauth_token_secret,
            );
            do_tests_v2(auth).await;
        }
        _ => panic!("Arg1 (API version) not recognized"),
    }
}

async fn do_tests_v1_1(secrets: &TokenSecrets) {
    dbg!(
        show_user_by_screen_name(secrets, Client::new(), "mafik74", Some(true))
            .await
            .unwrap()
    );
}

async fn do_tests_v2(auth: Oauth1aToken) {
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

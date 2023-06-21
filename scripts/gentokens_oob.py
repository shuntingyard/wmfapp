""" Generate access tokens out-of-band: that is by requesting a PIN known to the user.
"""
import json
from sys import stderr, stdin
from typing import Dict

from requests_oauthlib import OAuth1Session


def post_request_token_oob(consumer_key: str, consumer_secret: str) -> str:
    """
    1) The value for oauth_callback must be set to 'oob' during the
       POST oauth/request_token call.
    """
    oauth1 = OAuth1Session(consumer_key, consumer_secret, callback_uri="oob")

    ENDPOINT = "https://api.twitter.com/oauth/request_token"

    try:
        response = oauth1.fetch_request_token(ENDPOINT)
        return str(response.get("oauth_token"))

    except Exception as e:
        print(e, file=stderr)
        exit(-1)


def get_user_authorization(oauth_token: str) -> str:
    """
    2) Request PIN from user.
    """
    ENDPOINT = "https://api.twitter.com/oauth/authorize"
    user_url = f"{ENDPOINT}?oauth_token={oauth_token}"

    # Interact with user.
    print(f"Go to '{user_url}'", file=stderr)
    print("and enter PIN generated: ", file=stderr, end="")
    stderr.flush()

    return stdin.readline().strip()


def post_acess_token(
    client_key: str,
    client_secret: str,
    resource_owner_key: str,
    verifier: str,
) -> Dict:
    """
    3) Application uses PIN number as the oauth_verifier in POST oauth/access_token
       to obtain an access_token.
    """

    oauth1 = OAuth1Session(
        client_key,
        client_secret,
        resource_owner_key,
    )

    ENDPOINT = "https://api.twitter.com/oauth/access_token"

    try:
        return oauth1.fetch_access_token(ENDPOINT, verifier)

    except Exception as e:
        print(e, file=stderr)
        exit(-1)


if __name__ == "__main__":
    # As we want results on stdout, all prompting business
    # has to be done via stderr/ stdin.
    try:
        print("Consumer (aka api) key   : ", file=stderr, end="")
        stderr.flush()
        consumer_key = stdin.readline().strip()

        print("Consumer (aka api) secret: ", file=stderr, end="")
        stderr.flush()
        consumer_secret = stdin.readline().strip()

        oauth_token = post_request_token_oob(consumer_key, consumer_secret)

        oauth_verifier = get_user_authorization(oauth_token)

        creds = post_acess_token(
            consumer_key,
            consumer_secret,
            oauth_token,
            oauth_verifier,
        )

        creds["consumer_key"] = consumer_key
        creds["consumer_secret"] = consumer_secret
        print(json.dumps(creds, indent=2))

    except KeyboardInterrupt:
        pass

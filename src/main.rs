extern crate reqwest;
extern crate serde;
extern crate serde_json;

use std::string::String;

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

const TOKEN_ENV_VAR: &'static str = "TG_BOT_TOKEN";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Response<T> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct User {
    id: i64,
    is_bot: bool,
    first_name: String,
    last_name: Option<String>,
    username: Option<String>,
    language_code: Option<String>,
}

fn request<T: DeserializeOwned>(
    client: &reqwest::Client,
    token: &str,
    method: &str,
) -> Response<T> {
    const BASE_URL: &'static str = "https://api.telegram.org/";

    let url = {
        let mut url_str = String::from(BASE_URL);
        url_str.push_str("bot");
        url_str.push_str(token);
        url_str.push('/');
        url_str.push_str(method);

        reqwest::Url::parse(&url_str).unwrap()
    };

    serde_json::from_str(&client.get(url).send().unwrap().text().unwrap()).unwrap()
}

fn main() {
    let token = std::env::var(TOKEN_ENV_VAR).expect("Missing TG_BOT_TOKEN env var");
    let client = reqwest::Client::new();

    let me: Response<User> = request(&client, &token, "getMe");
    println!("{:?}", me);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_url_format() {
        let token = "123:abc-xyz";
        let method = "fooBar";
        let expected =
            reqwest::Url::parse("https://api.telegram.org/bot123:abc-xyz/fooBar").unwrap();
        assert_eq!(get_api_url(token, method), expected);
    }
}

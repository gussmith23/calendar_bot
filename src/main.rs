extern crate reqwest;
extern crate serde;
extern crate serde_json;

mod tg;

use std::string::String;

use serde::de::DeserializeOwned;

const TOKEN_ENV_VAR: &'static str = "TG_BOT_TOKEN";

fn request<T: DeserializeOwned>(
    client: &reqwest::Client,
    token: &str,
    method: &str,
) -> tg::Response<T> {
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

    let me: tg::Response<tg::User> = request(&client, &token, "getMe");
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

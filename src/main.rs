extern crate reqwest;

use std::string::String;

const TOKEN_ENV_VAR: &'static str = "TG_BOT_TOKEN";

fn get_api_url(token: &str, method: &str) -> reqwest::Url {
    const BASE_URL: &'static str = "https://api.telegram.org/";

    let mut url_str = String::from(BASE_URL);
    url_str.push_str("bot");
    url_str.push_str(token);
    url_str.push('/');
    url_str.push_str(method);

    reqwest::Url::parse(&url_str).unwrap()
}

fn main() {
    let token = std::env::var(TOKEN_ENV_VAR).expect("Missing TG_BOT_TOKEN env var");
    let client = reqwest::Client::new();

    let me = client.get(get_api_url(&token, "getMe")).send().unwrap().text().unwrap();
    println!("{}", me);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_url_format() {
        let token = "123:abc-xyz";
        let method = "fooBar";
        let expected = reqwest::Url::parse("https://api.telegram.org/bot123:abc-xyz/fooBar").unwrap();
        assert_eq!(get_api_url(token, method), expected);
    }
}

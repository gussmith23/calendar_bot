pub mod types;

pub use self::types::*;

use std::string::String;

use futures::Future;
use serde::de::DeserializeOwned;

/// A client for the Telegram Bot API.
pub struct Client<S> {
    token: String,
    send: S,
}

impl<S, F, E> Client<S>
where
    S: Fn(&str) -> F,
    F: Future<Item = String, Error = E>,
{
    /// Creates a new `Client`. `token` is the bot token given by the
    /// Botfather. `send` is a function that will be called with a URL
    /// string and should return a `Future` yielding the response
    /// body.
    pub fn new(token: String, send: S) -> Client<S> {
        Client {
            token: token,
            send: send,
        }
    }

    /// Fires off an API request, where `method` is the API method
    /// (e.g. "getUpdates" or "sendMessage").
    fn request<T>(&self, method: &str) -> impl Future<Item = T, Error = E>
    where
        T: DeserializeOwned,
    {
        const BASE_URL: &'static str = "https://api.telegram.org/";

        let mut url_str = String::from(BASE_URL);
        url_str.push_str("bot");
        url_str.push_str(&self.token);
        url_str.push('/');
        url_str.push_str(method);

        (self.send)(&url_str)
            .map(|s| serde_json::from_str(&s).expect("Received invalid JSON response"))
    }

    pub fn get_me(&self) -> impl Future<Item = Response<User>, Error = E> {
        self.request("getMe")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::future;
    use serde::Deserialize;
    use serde::Serialize;

    #[test]
    /// Tests that `request` formats its request correctly.
    fn request_format() {
        const TOKEN: &'static str = "123:abc-xyz";
        const METHOD: &'static str = "fooBar";
        const EXPECTED_URL: &'static str = "https://api.telegram.org/bot123:abc-xyz/fooBar";

        // Our `send` implementation that will verify what `request`
        // sends.
        let mock_send = |url: &str| {
            assert_eq!(url, EXPECTED_URL);

            future::ok::<String, ()>(serde_json::to_string(&()).unwrap())
        };

        let client = Client::new(String::from(TOKEN), mock_send);
        client.request::<()>(METHOD).wait().unwrap();
    }

    #[test]
    /// Tests that `request` correctly returns the result it receives.
    fn request_result() {
        #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
        struct Fromble {
            n: u32,
            b: bool,
        }

        let expected_result = Fromble { n: 1, b: true };

        let stub_send =
            |_: &str| future::ok::<String, ()>(serde_json::to_string(&expected_result).unwrap());

        let client = Client::new(String::from(""), stub_send);
        let result: Fromble = client.request("").wait().unwrap();

        assert_eq!(result, expected_result);
    }
}

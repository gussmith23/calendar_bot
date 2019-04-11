pub mod types;

pub use self::types::*;

use std::string::String;

use futures::stream;
use futures::stream::Stream;
use futures::Future;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;

/// A client for the Telegram Bot API. Methods correspond to API
/// calls.
pub struct Client<S> {
    token: String,
    send: S,
}

type Result<T> = std::result::Result<T, ()>;

fn to_result<T>(r: Response<T>) -> Result<T> {
    if r.ok {
        Ok(r.result.unwrap())
    } else {
        Err(())
    }
}

impl<S, F, E> Client<S>
where
    S: Fn(String, Option<String>) -> F,
    F: Future<Item = String, Error = E>,
{
    /// Creates a new `Client`. `token` is the bot token given by the
    /// Botfather. `send` is a function that will be called with a URL
    /// and request body, and should return a `Future` yielding the
    /// response body.
    pub fn new(token: String, send: S) -> Client<S> {
        Client {
            token: token,
            send: send,
        }
    }

    pub fn get_me(&self) -> impl Future<Item = Result<User>, Error = E> {
        self.request("getMe", None as Option<()>).map(to_result)
    }

    pub fn get_updates(
        &self,
        args: GetUpdates,
    ) -> impl Future<Item = Result<Vec<Update>>, Error = E> {
        self.request("getUpdates", Some(args)).map(to_result)
    }

    pub fn send_message(&self, arg: SendMessage) -> impl Future<Item = Result<Message>, Error = E> {
        self.request("sendMessage", Some(arg)).map(to_result)
    }

    /// Fires off an API request, where `method` is the API method
    /// (e.g. "getUpdates" or "sendMessage").
    fn request<T, U>(&self, method: &str, body: Option<T>) -> impl Future<Item = U, Error = E>
    where
        T: Serialize,
        U: DeserializeOwned,
    {
        const BASE_URL: &'static str = "https://api.telegram.org/";

        let mut url_str = String::from(BASE_URL);
        url_str.push_str("bot");
        url_str.push_str(&self.token);
        url_str.push('/');
        url_str.push_str(method);

        let body_string = body.as_ref().map(|o| serde_json::to_string(o).unwrap());

        (self.send)(url_str, body_string)
            .map(|s| serde_json::from_str(&s).expect("Received invalid JSON response"))
    }
}

/// Gets a `Stream` of updates from the API.
///
/// This relieves the user of stringing together `Future`s, dealing
/// with the `Vec` in the return type of `Client::get_updates`, and of
/// passing the correct update offset each time.
pub fn update_stream<'a, S, F, E>(
    client: &'a Client<S>,
    poll_timeout: u64,
) -> impl Stream<Item = Update, Error = E> + 'a
where
    S: Fn(String, Option<String>) -> F,
    F: 'a + Future<Item = String, Error = E>,
    E: 'a,
{
    assert_ne!(poll_timeout, 0);

    stream::unfold(None, move |offset| {
        let updates_req = GetUpdates {
            offset: offset,
            limit: Some(100),
            timeout: Some(poll_timeout as _),
            allowed_updates: None,
        };

        Some(
            client
                .get_updates(updates_req)
                .map(Result::unwrap)
                .map(|updates| {
                    // We need to get the last update ID to pass the
                    // correct offset on the next get_updates() call.
                    let next_offset = updates.last().map(|u| u.update_id + 1);
                    (stream::iter_ok(updates), next_offset)
                }),
        )
    })
    .flatten()
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

        let body: (u32, char) = (1, 'A');

        // Our `send` implementation that will verify what `request`
        // sends.
        let mock_send = |url: String, body_string: Option<String>| {
            assert_eq!(url.as_str(), EXPECTED_URL);
            body_string.map(|s| assert_eq!(s, serde_json::to_string(&body).unwrap()));

            future::ok::<String, ()>(serde_json::to_string(&()).unwrap())
        };

        let client = Client::new(String::from(TOKEN), mock_send);
        client
            .request::<_, ()>(METHOD, None as Option<()>)
            .wait()
            .unwrap();
        client.request::<_, ()>(METHOD, Some(&body)).wait().unwrap();
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

        let stub_send = |_: String, _: Option<String>| {
            future::ok::<String, ()>(serde_json::to_string(&expected_result).unwrap())
        };

        let client = Client::new(String::from(""), stub_send);
        let result: Fromble = client.request("", None as Option<()>).wait().unwrap();

        assert_eq!(result, expected_result);
    }
}

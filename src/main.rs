extern crate chrono;
extern crate futures;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

mod cal;
mod tg;

use std::string::String;

use futures::future;
use futures::Future;
use futures::Stream;

fn main() {
    let token = std::env::var(TOKEN_ENV_VAR).expect("Missing TG_BOT_TOKEN env var");
    let http_client = reqwest::Client::new();
    let tg_client = tg::Client::new(token, |url, body| synchronous_send(&http_client, url, body));
    let me = tg_client.get_me().wait().unwrap().unwrap();
    println!("{:?}", me);

    tg::update_stream(&tg_client, 10)
        .wait()
        .map(Result::unwrap)
        .for_each(|update| {
            println!("{:?}", update);

            if let Some(ref recv_msg) = update.message {
                let msg = tg::SendMessage {
                    chat_id: recv_msg.chat.id,
                    text: String::from("frack my sack"),
                };
                tg_client.send_message(msg).wait().unwrap().unwrap();
            }
        });
}

/// Adapter for using reqwest with futures.
fn synchronous_send(
    client: &reqwest::Client,
    url: String,
    body: Option<String>,
) -> impl Future<Item = String, Error = reqwest::Error> {
    let mut req = client.get(&url);
    if let Some(b) = body {
        req = req
            .body(b)
            .header(reqwest::header::CONTENT_TYPE, "application/json");
    }
    future::result::<String, reqwest::Error>(req.send().and_then(|mut resp| resp.text()))
}

const TOKEN_ENV_VAR: &'static str = "TG_BOT_TOKEN";

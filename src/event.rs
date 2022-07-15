use std::collections::HashMap;

use ewebsock::{WsEvent, WsReceiver, WsSender};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use url::Url;

pub struct EventReceiver {
    url: Url,
    // don't drop the sender or the connection will be closed
    _sender: WsSender,
    receiver: WsReceiver,
}

impl EventReceiver {
    pub fn connect(url: Url, wakeup: impl Fn() + Send + Sync + 'static) -> ewebsock::Result<Self> {
        match ewebsock::connect_with_wakeup(url.as_str(), wakeup) {
            Ok((sender, receiver)) => Ok(EventReceiver { url, _sender: sender, receiver }),
            Err(e) => Err(e),
        }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn try_recv(&self) -> Option<WsEvent> {
        self.receiver.try_recv()
    }
}

#[derive(Debug, Deserialize)]
pub struct Event {
    #[serde(deserialize_with = "time::serde::iso8601::option::deserialize")]
    pub time: Option<time::OffsetDateTime>,
    pub state: Option<String>,
    pub service: Option<String>,
    pub host: Option<String>,
    pub description: Option<String>,
    #[serde(deserialize_with = "deserialize_collections")]
    pub tags: Vec<String>,
    pub ttl: Option<f32>,
    pub time_micros: Option<i64>,
    pub metric: Option<f32>,
    #[serde(flatten)]
    pub attributes: HashMap<String, Value>,
}

fn deserialize_collections<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    // Deserialize null to empty Vec
    Deserialize::deserialize(deserializer).or(Ok(vec![]))
}

use serde::de::DeserializeOwned;
use serde::Serialize;
use seed::fetch::{Request, Method};
use seed::log;

use crate::api::{Initial, Config};

async fn post(url: &str, data: &impl Serialize) -> Result<String, &'static str> {
    Request::new(format!("http://localhost:8000/{}", url))
        .method(Method::Post)
        .json(data)
        .map_err(|_| "Could not serialize to json")?
        .fetch()
        .await
        .map_err(|_| "Could not execute request")?
        .check_status()
        .map_err(|_| "Response doesn't have 2xx status")?
        .text()
        .await
        .map_err(|_| "Could not parse response to string")
}

pub async fn start(pair: (Initial, Config)) {
    log!(post("api/start", &pair).await)
}

pub async fn stop() {
    log!(post("api/stop", &()).await)
}

pub async fn update(config: Config) {
    log!(post("api/update", &config).await)
}
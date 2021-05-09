use seed::fetch::{Method, Request};
use seed::log;
use serde::de::DeserializeOwned;
use serde::{Serialize, Deserialize};

use crate::api::{Config, InitConfig, State};

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

async fn get<T:  DeserializeOwned + 'static>(url: &str) -> Result<T, &'static str> {
	Request::new(format!("http://localhost:8000/{}", url))
		.method(Method::Get)
		.fetch()
		.await
		.map_err(|_| "Could not execute request")?
		.check_status()
		.map_err(|_| "Response doesn't have 2xx status")?
		.json()
		.await
		.map_err(|_| "Could not parse response to json")
}

pub async fn start(pair: (InitConfig, Config)) {
	log!(post("api/start", &pair).await)
}

pub async fn stop() {
	log!(post("api/stop", &()).await)
}

pub async fn update(config: Config) {
	log!(post("api/update", &config).await)
}

pub async fn query() -> Option<State> {
	get("api/query").await.ok()
}

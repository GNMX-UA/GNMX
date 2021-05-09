use seed::fetch::{Method, Request};
use seed::log;
use serde::de::DeserializeOwned;
use serde::{Serialize, Deserialize};

use crate::api::{Config, InitConfig, State};

async fn post(url: &str, data: &impl Serialize) -> Result<(), &'static str> {
	Request::new(format!("http://localhost:8000/{}", url))
		.method(Method::Post)
		.json(data)
		.map_err(|_| "Could not serialize to json")?
		.fetch()
		.await
		.map_err(|_| "Could not execute request")?
		.check_status()
		.map_err(|_| "Response doesn't have 2xx status")
		.map(|_|())
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

pub async fn start(pair: (InitConfig, Config)) -> Result<(), &'static str> {
	post("api/start", &pair).await
}

pub async fn stop() -> Result<(), &'static str> {
	post("api/stop", &()).await
}

pub async fn update(config: Config) -> Result<(), &'static str> {
	post("api/update", &config).await
}

pub async fn query() -> Result<State, &'static str> {
	get("api/query").await
}

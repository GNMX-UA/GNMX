use seed::fetch::{Method, Request};
use seed::log;
use serde::de::DeserializeOwned;
use serde::{Serialize, Deserialize};

use crate::api::{Config, InitConfig, State};

async fn post(url: &str, data: &impl Serialize) -> Result<(), String> {
	match Request::new(format!("http://localhost:8000/{}", url))
		.method(Method::Post)
		.json(data)
		.map_err(|_| "Could not serialize to json")?
		.fetch()
		.await
		.map_err(|_| "Could not execute request")?
		.check_status()
		.map_err(|_| "Response doesn't have 2xx status")?
		.json::<Result<(), String>>()
		.await
		.map_err(|_| "Could not parse response to json"){
		Ok(Ok(_)) => Ok(()),
		Ok(Err(e)) => Err(e),
		Err(e) => Err(e.to_string())
	}
}

async fn get<T:  DeserializeOwned + 'static>(url: &str) -> Result<T, String> {
	match Request::new(format!("http://localhost:8000/{}", url))
		.method(Method::Get)
		.fetch()
		.await
		.map_err(|_| "Could not execute request")?
		.check_status()
		.map_err(|_| "Response doesn't have 2xx status")?
		.json::<Result<T, String>>()
		.await
		.map_err(|_| "Could not parse response to json") {
		Ok(Ok(t)) => Ok(t),
		Ok(Err(e)) => Err(e),
		Err(e) => Err(e.to_string())
	}
}

pub async fn start(pair: (InitConfig, Config)) -> Result<(), String> {
	post("api/start", &pair).await
}

pub async fn stop() -> Result<(), String> {
	post("api/stop", &()).await
}

pub async fn update(config: Config) -> Result<(), String> {
	post("api/update", &config).await
}

pub async fn query() -> Result<State, String> {
	get("api/query").await
}

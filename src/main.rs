#[macro_use]
extern crate rocket;

use std::sync::{mpsc, Arc, Mutex};

use rocket::State;
use rocket_contrib::{json::Json, serve::StaticFiles};
use rocket_cors::CorsOptions;
use serde::{Deserialize, Serialize};
use simulation::{init, step};

#[derive(Deserialize, Clone, Debug)]
struct Initial {
	ticks: Option<u64>,
}

#[derive(Deserialize, Clone, Debug)]
struct Config;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Data {
	pub size:      f32,
	pub phenotype: f32,
}

struct Inner {
	pub config: Config,
	pub values: Vec<Data>,
	pub killer: mpsc::Sender<()>,
}

type Shared = Arc<Mutex<Option<Inner>>>;

fn simulate(initial: Initial, shared: Shared, kill: mpsc::Receiver<()>) {
	let mut state = init();

	for _ in 0 .. initial.ticks.unwrap_or(u64::MAX) {
		if let Err(_) = kill.try_recv() {
			return;
		}

		let mut lock = shared.lock().unwrap();
		// step(&mut state, &inner.config);
		println!("step");

		lock.as_mut().unwrap().values.push(Data {
			size:      5.,
			phenotype: 2.,
		})
	}
}

fn start(initial: Initial, config: Config, shared: &Shared) {
	let (sender, receiver) = mpsc::channel();

	let inner = Inner {
		config,
		values: vec![],
		killer: sender,
	};
	shared.lock().unwrap().get_or_insert(inner);
	let cloned = shared.clone();

	std::thread::spawn(move || simulate(initial, cloned, receiver));
}

#[post("/start", data = "<pair>")]
fn start_route(pair: Json<(Initial, Config)>, shared: State<Shared>) -> &'static str {
	let result = shared
		.lock()
		.unwrap()
		.as_ref()
		.map(|_| "already running")
		.unwrap_or_default();

	// double lock of mutex in one function
	let (initial, config) = pair.into_inner();
	start(initial, config, shared.inner());

	result
}

#[post("/stop")]
fn stop_route(shared: State<Shared>) -> String {
	match &mut *shared.lock().unwrap() {
		Some(Inner { killer, .. }) => killer.send(()).unwrap(),
		None => return "not running".to_string(),
	}
	String::new()
}

#[post("/update", data = "<config>")]
fn update_route(config: Json<Config>, shared: State<Shared>) -> String {
	match &mut *shared.lock().unwrap() {
		Some(inner) => inner.config = config.into_inner(),
		None => return "not running".to_string(),
	}
	String::new()
}

#[tokio::main]
async fn main() -> Result<(), rocket::error::Error> {
	let routes = routes![start_route, stop_route, update_route];
	let managed: Shared = Arc::new(Mutex::default());

	rocket::build()
		.attach(CorsOptions::default().to_cors().unwrap())
		.mount("/api", routes)
		.mount("/", StaticFiles::from("static"))
		.manage(managed)
		.launch()
		.await
}

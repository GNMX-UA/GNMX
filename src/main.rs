#[macro_use]
extern crate rocket;

use std::sync::{mpsc, Arc, Mutex};

use rocket::State;
use rocket_contrib::{json::Json, serve::StaticFiles};
use rocket_cors::CorsOptions;
use serde::{Deserialize, Serialize};
use simulation::{init, step, Config, InitConfig};

static POISON_ERROR: &str =
	"The simulation thread crashed and poisoned the simulation, this is an internal error";
static ALREADY_STARTED_ERROR: &str = "The simulation was already started, ignoring request";
static NOT_STARTED_ERROR: &str = "The simulation was not yet started, ignoring request";
static SETUP_FAILED: &str = "Simulation setup failed, this is an internal error";

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Data {
	pub size: f32,
	pub phenotype: f32,
}

struct Inner {
	pub config: Config,
	pub values: Vec<Data>,
	pub state: simulation::State,
	pub killer: mpsc::Sender<()>,
}

type Shared = Arc<Mutex<Option<Inner>>>;

fn simulate(ticks: u64, shared: Shared, kill: mpsc::Receiver<()>) {
	for _ in 0..ticks {
		match kill.try_recv() {
			Err(mpsc::TryRecvError::Empty) => (),
			_ => return,
		}

		match shared.lock().unwrap().as_mut() {
			Some(inner) => {
				step(&mut inner.state, &inner.config);
				inner.state.tick += 1;
				inner.values.push(Data {
					size: 5.,
					phenotype: 2.,
				})
			}
			None => warn!("{}", SETUP_FAILED),
		}
	}
}

fn start(initial: InitConfig, config: Config, shared: &Shared) {
	let (sender, receiver) = mpsc::channel();

	let ticks = initial.t_max.unwrap_or(u64::MAX);
	let inner = Inner {
		config,
		values: vec![],
		killer: sender,
		state: init(initial).unwrap(),
	};

	if let Ok(mut lock) = shared.lock() {
		lock.get_or_insert(inner);
	}

	let cloned = shared.clone();
	std::thread::spawn(move || simulate(ticks, cloned, receiver));
}

#[post("/start", data = "<pair>")]
fn start_route(
	pair: Json<(InitConfig, Config)>,
	shared: State<Shared>,
) -> Json<Result<(), &'static str>> {
	let result = match shared.lock(){
		Ok(mut lock) => match &mut *lock {
			Some(_) => Err(ALREADY_STARTED_ERROR),
			None => Ok(()),
		}
		Err(_) => Err(POISON_ERROR)
	};

	if result.is_ok() {
		let (initial, config) = pair.into_inner();
		start(initial, config, shared.inner());
	}

	Json(result)
}

#[post("/stop")]
fn stop_route(shared: State<Shared>) -> Json<Result<(), &'static str>> {
	let response = match shared.lock() {
		Ok(mut lock) => {
			let result = match &mut *lock {
				Some(inner) => match inner.killer.send(()){
					Ok(_) => Ok(()),
					Err(_) => Err(POISON_ERROR)
				}
				None => Err(NOT_STARTED_ERROR),
			};
			*lock = None;
			result
		},
		Err(_) => Err(POISON_ERROR)
	};
	Json(response)
}

#[post("/update", data = "<config>")]
fn update_route(config: Json<Config>, shared: State<Shared>) -> Json<Result<(), &'static str>> {
	let response = match shared.lock() {
		Ok(mut lock) => match &mut *lock{
			Some(inner) => {
				inner.config = config.into_inner();
				Ok(())
			}
			None => Err(NOT_STARTED_ERROR),
		},
		Err(_) => Err(POISON_ERROR)
	};
	Json(response)
}

#[get("/query")]
fn query_route(shared: State<Shared>) -> Json<Result<simulation::State, &'static str>> {
	let response = match shared.lock() {
		Ok(lock) => lock
			.as_ref()
			.map(|inner| inner.state.clone())
			.ok_or(NOT_STARTED_ERROR),
		Err(_) => Err(POISON_ERROR),
	};
	Json(response)
}

#[tokio::main]
async fn main() -> Result<(), rocket::error::Error> {
	let routes = routes![start_route, stop_route, update_route, query_route];
	let managed: Shared = Arc::new(Mutex::default());

	rocket::build()
		.attach(CorsOptions::default().to_cors().unwrap())
		.mount("/api", routes)
		.mount("/", StaticFiles::from("static"))
		.manage(managed)
		.launch()
		.await
}

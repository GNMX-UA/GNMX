use std::time::{Duration, Instant};

use futures::StreamExt;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::{Filter, Reply};

use rand::seq::SliceRandom;
use simulation::{init, patch::Patch, step, Config, InitConfig, TempEnum};

static ERROR: &str = "Internal server error, an illegal message was received.";
static DROPPED: &str = "The receiver on the simulation thread were dropped, most likely due to a crash. Please refresh the page or restart.";
static NAN: &str = "Encountered NaN in loci or population size has become 0, stopping simulation.";
static WS: &str = "Websocket was closed while the simulation thread was still running, stopping simulation.";

static SAMPLE_SIZE: usize = 100;
static INTERVAL: u64 = 100;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphData {
	population: u64,
	phenotype_variance: f64,
	phenotype_distance: f64,

	// (patch_index, phenotype) - max: SAMPLE_SIZE
	phenotype_sample: Vec<(usize, f64)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Query {
	Stop,
	Start(Config),
	Pause,
	Resume,
	Update(Config),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Notification {
	Pause,
	Resume,
	Update(Config),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Response {
	Stopped,
	Started,
	Info(String),
	State(u64, GraphData),
	Error(String),
}

fn extract_graph_data(patches: &[(Patch, f64)]) -> Option<GraphData> {
	let phenotypes: Vec<_> = patches
		.iter()
		.enumerate()
		.map(|(index, (patch, _))| {
			patch
				.individuals
				.iter()
				.map(move |indiv| (index, indiv.phenotype()))
		})
		.flatten()
		.collect();

	let mean = match phenotypes.len() {
		0 => None,
		_ => Some(phenotypes.iter().map(|(_, p)| p).sum::<f64>() / phenotypes.len() as f64),
	}?;

	let variance = phenotypes
		.iter()
		.map(|(_, f)| (f - mean) * (f - mean))
		.sum::<f64>()
		/ phenotypes.len() as f64;

	let max = phenotypes
		.iter()
		.max_by(|(_, x), (_, y)| x.partial_cmp(y).unwrap())?
		.1;
	let min = phenotypes
		.iter()
		.min_by(|(_, x), (_, y)| x.partial_cmp(y).unwrap())?
		.1;

	Some(GraphData {
		population: patches
			.iter()
			.map(|(patch, _)| patch.individuals.len())
			.sum::<usize>() as u64,
		phenotype_variance: variance,
		phenotype_distance: max - min,
		phenotype_sample: phenotypes
			.choose_multiple(&mut rand::thread_rng(), SAMPLE_SIZE)
			.cloned()
			.collect(),
	})
}

fn notify(notifier: &std::sync::mpsc::Sender<Notification>, notification: Notification) {
	if let Err(_) = notifier.send(notification) {
		error!("{}", DROPPED)
	}
}

async fn respond(responder: &mpsc::Sender<Response>, response: Response) {
	if let Err(_) = responder.send(response).await {
		error!("{}", WS)
	}
}

fn blocking_respond(responder: &mpsc::Sender<Response>, response: Response) {
	if let Err(_) = responder.blocking_send(response) {
		error!("{}", WS)
	}
}

fn simulate(
	initial: InitConfig,
	mut config: Config,
	receiver: std::sync::mpsc::Receiver<Notification>,
	sender: mpsc::Sender<Response>,
) {
	blocking_respond(&sender, Response::Started);
	info!("new simulation thread started");

	let ticks = initial.t_max.unwrap_or(u64::MAX);
	let mut state = init(initial).unwrap();

	let mut paused = false;

	let mut last = Instant::now();
	let interval = Duration::from_millis(INTERVAL);

	loop {
		if state.tick > ticks {
			blocking_respond(&sender, Response::Info("Simulation has ended successfully".to_string()));
			return;
		}

		match receiver.try_recv() {
			Err(std::sync::mpsc::TryRecvError::Disconnected) => {
				blocking_respond(&sender, Response::Info("Simulation was successfully stopped".to_string()));
				return;
			}
			Ok(Notification::Update(new)) => config = new,
			Ok(Notification::Pause) => paused = true,
			Ok(Notification::Resume) => paused = false,
			Err(std::sync::mpsc::TryRecvError::Empty) => (),
		}

		// to prevent a pure busy loop we sleep for 20 milliseconds each time we are paused
		if paused {
			std::thread::sleep(Duration::from_millis(20));
			continue;
		}

		step(&mut state, &config);
		state.tick += 1;

		if last.elapsed() > interval {
			debug!("sending state {}", state.tick);
			std::thread::yield_now();
			last = Instant::now();

			let data = extract_graph_data(&state.patches).expect(NAN);
			blocking_respond(&sender, Response::State(state.tick, data));
		}
	}
}

async fn receive(connection: WebSocket) {
	let (sink, mut stream) = connection.split();
	let (responder, response_receiver) = mpsc::channel(128);

	let mut notifier = None;

	let rx = ReceiverStream::new(response_receiver);
	tokio::task::spawn(
		rx.map(|resp| Ok(Message::text(serde_json::to_string(&resp).unwrap())))
			.forward(sink),
	);

	while let Some(Ok(message)) = stream.next().await {
		if message.is_text() {
			let msg = serde_json::from_slice(message.as_bytes()).unwrap();
			match (msg, &mut notifier) {
				(Query::Start(config), None) => {
					let initial = InitConfig {
						t_max: None,
						kind: TempEnum::Default,
						patches: 5,
						individuals: 100,
						loci: 5,
					};

					let (notif_sender, notif_receiver) = std::sync::mpsc::channel();
					notifier = Some(notif_sender);

					let cloned = responder.clone();
					std::thread::spawn(move || simulate(initial, config, notif_receiver, cloned));
				}
				(Query::Stop, notifier) => {
					*notifier = None;
					respond(&responder, Response::Stopped).await
				}
				(Query::Update(config), Some(notifier)) => {
					notify(notifier, Notification::Update(config))
				}
				(Query::Pause, Some(notifier)) => {
					notify(notifier, Notification::Pause)
				}
				(Query::Resume, Some(notifier)) => {
					notify(notifier, Notification::Resume)
				}
				_ => respond(&responder, Response::Error(ERROR.to_string())).await,
			}
		} else {
			// TODO: also handle disconnect instead of this error
			warn!("received message is not text type")
		}
	}
	info!("websocket disconnected");
}

#[tokio::main]
async fn main() {
	pretty_env_logger::init();

	let types = |reply: warp::filters::fs::File| {
		if reply.path().ends_with("wasm.js") {
			let reply = warp::reply::with_header(reply, "Cache-Control", "no-store");
			let reply = warp::reply::with_header(reply, "Content-Type", "text/javascript");
			reply.into_response()
		} else {
			reply.into_response()
		}
	};

	let ws = warp::path("ws")
		.and(warp::ws())
		.map(|ws: warp::ws::Ws| ws.on_upgrade(receive));

	let files = warp::fs::dir("static").map(types);
	let routes = ws.or(files);

	warp::serve(routes).run(([127, 0, 0, 1], 3030)).await
}

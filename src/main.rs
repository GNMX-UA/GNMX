use std::time::{Duration, Instant};

use futures::StreamExt;
use log::{debug, error, info, warn};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use simulation::{init, step, Config, InitConfig, Patch, TempEnum};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use warp::{
	ws::{Message, WebSocket},
	Filter, Reply,
};

static ERROR: &str = "Internal server error, an illegal message was received.";
static DROPPED: &str = "The receiver or sender in the simulation thread were dropped, most likely \
                        due to a crash, please restart the simulation.";
static NAN: &str = "Encountered NaN in loci or population size has become 0, stopping simulation.";

static SAMPLE_SIZE: usize = 100;
static INTERVAL: u64 = 100;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphData {
	population:         u64,
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
pub enum Response {
	Stopped,
	Started,
	State(u64, GraphData),
	Error(String),
}

#[derive(Clone, Debug)]
pub enum Notification {
	Update(Config),
	Pause,
	Resume,
	Stop,
}

fn extract_graph_data(patches: &[(Patch, f64)]) -> Option<GraphData> {
	let phenotypes: Vec<_> = patches
		.iter()
		.enumerate()
		.map(|(index, (patch, _))| patch.iter().map(move |indiv| (index, indiv.phenotype())))
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
		population:         patches.iter().map(|(patch, _)| patch.len()).sum::<usize>() as u64,
		phenotype_variance: variance,
		phenotype_distance: max - min,
		phenotype_sample:   phenotypes
			.choose_multiple(&mut rand::thread_rng(), SAMPLE_SIZE)
			.cloned()
			.collect(),
	})
}

fn simulate(
	initial: InitConfig,
	mut config: Config,
	receiver: std::sync::mpsc::Receiver<Notification>,
	sender: mpsc::Sender<Response>,
) {
	let ticks = initial.t_max.unwrap_or(u64::MAX);
	let mut state = init(initial).unwrap();
	let mut paused = false;

	let mut last = Instant::now();
	let interval = Duration::from_millis(INTERVAL);

	sender.blocking_send(Response::Started).unwrap();
	info!("simulation started");

	loop {
		if state.tick > ticks {
			info!("simulation ended");
			return;
		}

		match receiver.try_recv() {
			Err(std::sync::mpsc::TryRecvError::Disconnected) => {
				warn!("{}", DROPPED);
				return;
			},
			Ok(Notification::Stop) => {
				info!("Simulation was stopped.");
				return;
			},
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

			if let Err(_) = sender.blocking_send(Response::State(state.tick, data)) {
				info!("{}", DROPPED);
				return;
			}
		}
	}
}

async fn receive(connection: WebSocket) {
	let (sink, mut stream) = connection.split();

	let mut sink = Some(sink);
	let mut notifier = None;
	let mut responder = None;

	while let Some(Ok(message)) = stream.next().await {
		if message.is_text() {
			let msg = serde_json::from_slice(message.as_bytes()).unwrap();
			match (msg, &mut notifier, &mut responder) {
				(Query::Start(config), None, _) => {
					let initial = InitConfig {
						t_max:       None,
						kind:        TempEnum::Default,
						patches:     5,
						individuals: 10000,
						loci:        500,
					};

					let (response_sender, response_receiver) = mpsc::channel(128);
					let (notif_sender, notif_receiver) = std::sync::mpsc::channel();
					notifier = Some(notif_sender);
					responder = Some(response_sender.clone());

					std::thread::spawn(move || {
						simulate(initial, config, notif_receiver, response_sender)
					});

					if let Some(sink) = sink.take() {
						let rx = ReceiverStream::new(response_receiver);
						tokio::task::spawn(
							rx.map(|resp| Ok(Message::text(serde_json::to_string(&resp).unwrap())))
								.forward(sink),
						);
					}
				},
				(Query::Stop, Some(sender), _) =>
					if let Err(_) = sender.send(Notification::Stop) {
						error!("{}", DROPPED)
					},
				(Query::Update(config), Some(sender), _) => {
					if let Err(_) = sender.send(Notification::Update(config)) {
						error!("{}", DROPPED)
					}
				},
				(Query::Pause, Some(sender), _) => {
					if let Err(_) = sender.send(Notification::Pause) {
						error!("{}", DROPPED)
					}
				},
				(Query::Resume, Some(sender), _) => {
					if let Err(_) = sender.send(Notification::Resume) {
						error!("{}", DROPPED)
					}
				},
				(_, _, Some(responder)) => {
					if let Err(err) = responder.send(Response::Error(ERROR.to_string())).await {
						error!("{}", err)
					}
				},
				_ => error!("{}", ERROR),
			}
		} else {
			error!("received message is not text type")
		}
		warn!("websocket disconnected");
	}
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

mod api;
mod components;
mod fields;
mod forms;
mod graphs;

use seed::{prelude::*, *};

use crate::api::{Config, InitConfig};
use crate::forms::{Action, ConfigForm};
use crate::graphs::{line, scatter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::ops::Range;
use std::time::Duration;
use wasm_timer::Instant;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphData {
	population: u64,
	phenotype_variance: f64,
	phenotype_distance: f64,
	phenotype_sample: Vec<(usize, f64)>, // (patch_index, phenotype)
}

#[derive(Clone, Debug, Default)]
pub struct GraphRanges {
	population: Range<f64>,
	phenotype_variance: Range<f64>,
	phenotype_distance: Range<f64>,
	phenotype_sample: Range<f64>,
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

#[derive(Debug)]
pub enum Msg {
	Config(crate::forms::config::Msg),
	Delete(usize),
	Draw(u64),
	Ws(WebSocketMessage),
}

struct Model {
	config: ConfigForm,
	ws: WebSocket,

	history: Vec<(u64, GraphData)>,
	ranges: GraphRanges,

	messages: HashMap<usize, String>,
	current_id: usize,
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
	Model {
		config: ConfigForm::new(),
		history: vec![],
		ranges: Default::default(),
		ws: WebSocket::builder("ws://127.0.0.1:3030/ws", orders)
			.on_message(Msg::Ws)
			.build_and_open()
			.expect("websocket could not connect"),
		messages: HashMap::new(),
		current_id: 1,
	}
}

fn handle_error(model: &mut Model, error: String) {
	model.messages.insert(model.current_id, error);
	model.current_id += 1;
}

fn handle_action(action: Action, ws: &mut WebSocket) {
	match action {
		Action::Start(_, config) => log!(ws.send_json(&Query::Start(config))),
		Action::Update(config) => log!(ws.send_json(&Query::Update(config))),
		Action::Stop => log!(ws.send_json(&Query::Stop)),
		Action::Pause => log!(ws.send_json(&Query::Pause)),
		Action::Resume => log!(ws.send_json(&Query::Resume)),
		Action::None => {}
	}
}

fn update_ranges(data: &GraphData, ranges: &mut GraphRanges) {
	ranges.population.start = ranges.population.start.min(data.population as f64);
	ranges.population.end = ranges.population.end.max(data.population as f64);

	ranges.phenotype_variance.start = ranges.phenotype_variance.start.min(data.phenotype_variance);
	ranges.phenotype_variance.end = ranges.phenotype_variance.end.max(data.phenotype_variance);

	ranges.phenotype_distance.start = ranges.phenotype_distance.start.min(data.phenotype_distance);
	ranges.phenotype_distance.end = ranges.phenotype_distance.end.max(data.phenotype_distance);

	// use a crate for this abomination
	ranges.phenotype_sample.start = ranges.phenotype_sample.start.min(
		*data
			.phenotype_sample
			.iter()
			.map(|(a, b)| b)
			.min_by(|a, b| a.partial_cmp(b).unwrap())
			.unwrap(),
	);
	ranges.phenotype_sample.end = ranges.phenotype_sample.end.max(
		*data
			.phenotype_sample
			.iter()
			.map(|(a, b)| b)
			.max_by(|a, b| a.partial_cmp(b).unwrap())
			.unwrap(),
	);

	log(ranges);
}

fn draw_graphs(model: &mut Model) -> Duration {
	let start = Instant::now();

	line::draw(
		"canvas_pop",
		&model.history,
		|data| data.population as f64,
		model.ranges.population.clone(),
		"population size"
	)
	.expect("could not draw");

	scatter::draw(
		"canvas_pheno",
		&model.history,
		model.ranges.phenotype_sample.clone(),
		"phenotypes per patch"
	)
	.expect("could not draw");

	line::draw(
		"canvas_var",
		&model.history,
		|data| data.phenotype_variance,
		model.ranges.phenotype_variance.clone(),
		"phenotypes variation"
	)
	.expect("could not draw");

	let mapper = |data: &GraphData| data.phenotype_distance;
	line::draw(
		"canvas_dist",
		&model.history,
		mapper,
		model.ranges.phenotype_distance.clone(),
		"phenotypes distance"
	)
	.expect("could not draw");

	start.elapsed()
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
	match msg {
		Msg::Config(msg) => handle_action(
			model.config.update(msg, &mut orders.proxy(Msg::Config)),
			&mut model.ws,
		),
		Msg::Delete(id) => {
			model.messages.remove(&id);
		}
		Msg::Ws(message) => match message.json::<Response>() {
			Ok(Response::State(tick, data)) => {
				update_ranges(&data, &mut model.ranges);
				model.history.push((tick, data));

				// Start draw loop
				if model.history.len() == 1 {
					orders.send_msg(Msg::Draw(tick));
				}
			}
			Ok(Response::Started) => log!("simulation started"),
			Ok(Response::Error(error)) => handle_error(model, error),
			Ok(Response::Stopped) => model.config.stop(),
			Err(err) => handle_error(model, format!("{:?}", err)),
		},
		Msg::Draw(last) => {
			let tick = model.history.last().unwrap().0;
			log!(tick, last);

			let duration = if last == tick {
				Duration::from_millis(100)
			} else {
				draw_graphs(model)
			};

			orders.perform_cmd(cmds::timeout(duration.as_millis() as u32, move || {
				Msg::Draw(tick)
			}));
		}
	}
}

fn view_messages(messages: &HashMap<usize, String>) -> Vec<Node<Msg>> {
	messages
		.iter()
		.map(|(id, msg)| {
			let copy = *id;
			div![
				C!["notification"],
				button![C!["delete"], ev(Ev::Click, move |_| Msg::Delete(copy))],
				&msg
			]
		})
		.collect()
}

fn view(model: &Model) -> Node<Msg> {
	div![
		C!["columns"],
		div![
			C!["column is-8 ml-4 mt-5"],
			view_messages(&model.messages),
			canvas![attrs! {At::Id => "canvas_pop", At::Width => "800", At::Height => "300"}],
			canvas![attrs! {At::Id => "canvas_pheno", At::Width => "800", At::Height => "300"}],
			canvas![attrs! {At::Id => "canvas_var", At::Width => "800", At::Height => "300"}],
			canvas![attrs! {At::Id => "canvas_dist", At::Width => "800", At::Height => "300"}]
		],
		div![C!["column"], model.config.view().map_msg(Msg::Config)],
	]
}

#[wasm_bindgen(start)]
pub fn start() {
	App::start("app", init, update, view);
}

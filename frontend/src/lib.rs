mod api;
mod components;
mod fields;
mod forms;
mod graphs;

use seed::{prelude::*, *};

use crate::api::{Config, InitConfig, Patch, State};
use crate::forms::{Action, ConfigForm};
use crate::graphs::{area, line};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::future::Future;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Query {
	Stop,
	Start(Config),
	Update(Config),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Response {
	Stopped,
	Started,
	State(State),
	Error(String),
}

#[derive(Debug)]
pub enum Msg {
	Config(crate::forms::config::Msg),
	Delete(usize),
	Ws(WebSocketMessage),
}
pub struct GraphData {
	population: u64,
	phenotype_variance: f64,
	phenotype_distance: f64,
}

struct Model {
	config: ConfigForm,
	history: Vec<(u64, GraphData)>,
	ws: WebSocket,

	messages: HashMap<usize, String>,
	current_id: usize,
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
	Model {
		config: ConfigForm::new(),
		history: vec![],
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

fn extract_graph_data(patches: Vec<(Patch, f64)>) -> Option<GraphData> {
	let phenotypes: Vec<_> = patches
		.iter()
		.map(|(patch, _)| patch.individuals.iter().map(|indiv| indiv.phenotype()))
		.flatten()
		.collect();

	let mean = match phenotypes.len() {
		0 => None,
		_ => Some(phenotypes.iter().sum::<f64>() / phenotypes.len() as f64),
	}?;

	let variance = phenotypes
		.iter()
		.map(|f| (f - mean) * (f - mean))
		.sum::<f64>()
		/ phenotypes.len() as f64;

	let max = phenotypes.iter().max_by(|x, y| x.partial_cmp(y).unwrap());

	let min = phenotypes.iter().min_by(|x, y| x.partial_cmp(y).unwrap());

	Some(GraphData {
		population: patches
			.iter()
			.map(|(patch, _)| patch.individuals.len())
			.sum::<usize>() as u64,
		phenotype_variance: variance,
		phenotype_distance: max? - min?,
	})
}

fn handle_action(action: Action, ws: &mut WebSocket) {
	match action {
		Action::Start(_, config) => log!(ws.send_json(&Query::Start(config))),
		Action::Update(config) => log!(ws.send_json(&Query::Update(config))),
		Action::Stop => log!(ws.send_json(&Query::Stop)),
		Action::None => {}
	}
}

fn handle_state_change(model: &mut Model, state: State) {
	if let Some(data) = extract_graph_data(state.patches) {
		model.history.push((state.tick, data));
	}

	log!("before drawing population");
	area::draw("canvas_pop", &model.history, |data| data.population as f64)
		.expect("could not draw");

	// let pop_iter = model.history.iter().map(|(x, data)| (x, *data.population));
	// area::draw("canvas_pheno", pop_iter).expect("could not draw");

	log!("before drawing variation");
	line::draw("canvas_var", &model.history, |data| data.phenotype_variance)
		.expect("could not draw");

	log!("before drawing distance");
	// let mapper = |data| data.phenotype_distance;
	// area::draw("canvas_dist", &model.history, mapper).expect("could not draw");
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
	match msg {
		Msg::Config(msg) => handle_action(model.config.update(msg, &mut orders.proxy(Msg::Config)), &mut model.ws),
		Msg::Delete(id) => {
			model.messages.remove(&id);
		}
		Msg::Ws(message) => match message.json::<Response>() {
			Ok(Response::State(state)) => handle_state_change(model, state),
			Ok(Response::Started) => log!("simulation started"),
			Ok(Response::Error(error)) => handle_error(model, error),
			Ok(Response::Stopped) => model.config.stop(),
			Err(err) => handle_error(model, format!("{:?}", err))
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

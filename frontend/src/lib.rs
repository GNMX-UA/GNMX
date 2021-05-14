mod api;
mod components;
mod fields;
mod forms;
mod graphs;

use seed::{prelude::*, *};

use crate::api::{Config, InitConfig};
use crate::forms::selection::SelectionForm;
use crate::forms::{ConfigForm, InitConfigForm, SimulationForm};
use crate::graphs::scheduler::{DrawScheduler, GraphData};

use crate::components::Button;
use plotters_canvas::CanvasBackend;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::ops::Range;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Query {
	Reset,
	Start(InitConfig, Config),
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

#[derive(Debug)]
pub enum Msg {
	Config(crate::forms::config::Msg),
	InitConfig(crate::forms::init::Msg),
	SimulationConfig(crate::forms::simulation::Msg),

	Start,
	Reset,
	Pause,
	Resume,

	Delete(usize),
	Resize,
	Ws(WebSocketMessage),
}

struct Model {
	config: ConfigForm,
	init: InitConfigForm,
	simulation: SimulationForm,

	ws: WebSocket,
	scheduler: DrawScheduler,

	messages: HashMap<usize, (bool, String)>,
	current_id: usize,

	start: Button<Msg>,
	reset: Button<Msg>,
	pause: Button<Msg>,
	resume: Button<Msg>,

	paused: bool,
	started: bool,
	gamer: bool,
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
	orders.stream(streams::window_event(Ev::Resize, |_| Msg::Resize));
	Model {
		config: ConfigForm::new(),
		init: InitConfigForm::new(),
		simulation: SimulationForm::new(),

		ws: WebSocket::builder("ws://127.0.0.1:3030/ws", orders)
			.on_message(Msg::Ws)
			.build_and_open()
			.expect("websocket could not connect"),
		scheduler: DrawScheduler::new("canvas"),

		messages: HashMap::new(),
		current_id: 1,

		start: Button::new("start", "is-success", "fa-play", || Msg::Start),
		reset: Button::new("reset", "is-danger is-outlined", "fa-square", || Msg::Reset),
		pause: Button::new("pause", "is-light", "fa-pause", || Msg::Pause),
		resume: Button::new("resume", "is-light", "fa-play", || Msg::Resume),

		paused: false,
		started: false,
		gamer: false,
	}
}

fn handle_notification(model: &mut Model, text: impl Into<String>, is_error: bool) {
	model
		.messages
		.insert(model.current_id, (is_error, text.into()));
	model.current_id += 1;
}

fn send_json(model: &mut Model, query: &Query) {
	let _ = model.ws.send_json(query).map_err(|e| log!(e));
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
	match msg {
		Msg::Config(msg) => {
			// TODO Add timer to reduce load on websocket maybe?
			let _ = model.config.update(msg, &mut orders.proxy(Msg::Config));
			if let Some(config) = model.config.extract() {
				send_json(model, &Query::Update(config));
			}
		}
		Msg::InitConfig(msg) => {
			let _ = model.init.update(msg, &mut orders.proxy(Msg::InitConfig));
		}
		Msg::SimulationConfig(msg) => {
			let _ = model
				.simulation
				.update(msg, &mut orders.proxy(Msg::SimulationConfig));
		}
		Msg::Start => match (model.init.extract(), model.config.extract()) {
			(Some(init), Some(config)) => send_json(model, &Query::Start(init, config)),
			_ => handle_notification(
				model,
				"Cannot start simulation as some parameters are wrong",
				true,
			),
		},
		Msg::Reset => {
			send_json(model, &Query::Reset);
		}
		Msg::Pause => {
			model.paused = true;
			send_json(model, &Query::Pause)
		},
		Msg::Resume => {
			model.paused = false;
			send_json(model, &Query::Resume)
		},
		Msg::Delete(id) => {
			model.messages.remove(&id);
		}
		Msg::Ws(message) => match message.json::<Response>() {
			Ok(Response::State(tick, data)) => {
				if let Some(error) = model.scheduler.update_data(tick, data) {
					handle_notification(model, error, false)
				}
			}
			Ok(Response::Started) => {
				model.started = true;
				log!("simulation started")
			},
			Ok(Response::Info(info)) => handle_notification(model, info, false),
			Ok(Response::Error(error)) => handle_notification(model, error, true),
			Ok(Response::Stopped) => {
				model.scheduler.stop();
				model.started = false;
			}
			Err(err) => log!(err),
		},
		Msg::Resize => {
			if let Some(error) = model.scheduler.update_size() {
				model
					.messages
					.insert(model.current_id, (true, error.to_string()));
				model.current_id += 1;
			}
		}
	}
}

fn view_messages(messages: &HashMap<usize, (bool, String)>) -> Node<Msg> {
	div![
		style!{St::Position => "absolute"},
		messages
			.iter()
			.map(|(id, (is_error, msg))| {
				let copy = *id;
				div![
					C!["notification" IF!(*is_error => "is-warning")],
					button![C!["delete"], ev(Ev::Click, move |_| Msg::Delete(copy))],
					&msg
				]
			})
			.collect::<Vec<_>>()
	]
}

fn view(model: &Model) -> Node<Msg> {
	div![
		C!["columns"],
		div![
			C!["column is-8 ml-4 mt-5"],
			view_messages(&model.messages),
			model.scheduler.view(),
		],
		div![
			C!["column p-6"],
			style! {St::BoxShadow => "-10px 0px 10px 1px #eeeeee"},
			style! {St::OverflowY => "auto", St::Height => "100vh"},

			model.init.view(model.started).map_msg(Msg::InitConfig),
			hr![],
			model.config.view().map_msg(Msg::Config),
			hr![],
			model.simulation.view().map_msg(Msg::SimulationConfig),
			div![C!["pb-4"]],

			div![
				C!["buttons"],
				model.start.view(false, model.started),
				model.reset.view(false, !model.started),
				model.pause.view(!model.started, model.paused),
				model.resume.view(!model.started, !model.paused),
			],
			div![C!["pb-2"]],
		],

	]
}

#[wasm_bindgen(start)]
pub fn start() {
	App::start("app", init, update, view);
}

mod api;
mod components;
mod fields;
mod forms;
mod graphs;

use seed::{prelude::*, *};

use crate::api::{Config, InitConfig};
use crate::forms::selection::SelectionForm;
use crate::forms::{Action, ConfigForm};
use crate::graphs::scheduler::{DrawScheduler, GraphData};

use plotters_canvas::CanvasBackend;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::ops::Range;

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
	Info(String),
	State(u64, GraphData),
	Error(String),
}

#[derive(Debug)]
pub enum Msg {
	Config(crate::forms::config::Msg),
	Selection(crate::forms::selection::Msg),
	Delete(usize),
	Resize,
	Ws(WebSocketMessage),
}

struct Model {
	config: ConfigForm,
	selection: SelectionForm,

	ws: WebSocket,
	scheduler: DrawScheduler,

	messages: HashMap<usize, (bool, String)>,
	current_id: usize,
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
	orders.stream(streams::window_event(Ev::Resize, |_| Msg::Resize));
	Model {
		config: ConfigForm::new(),
		selection: SelectionForm::new(),

		ws: WebSocket::builder("ws://127.0.0.1:3030/ws", orders)
			.on_message(Msg::Ws)
			.build_and_open()
			.expect("websocket could not connect"),
		scheduler: DrawScheduler::new("canvas"),

		messages: HashMap::new(),
		current_id: 1,
	}
}

fn handle_action(model: &mut Model, action: Action) {
	match action {
		Action::Start(_, config) => {
			model.selection.set_loci(5);
			model.scheduler.update_selection(model.selection.extract());
			log!(model.ws.send_json(&Query::Start(config)))
		}
		Action::Update(config) => log!(model.ws.send_json(&Query::Update(config))),
		Action::Stop => {
			model.selection.set_loci(0);
			log!(model.ws.send_json(&Query::Stop))
		}
		Action::Pause => log!(model.ws.send_json(&Query::Pause)),
		Action::Resume => log!(model.ws.send_json(&Query::Resume)),
		Action::None => {}
	}
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
	match msg {
		Msg::Config(msg) => {
			let action = model.config.update(msg, &mut orders.proxy(Msg::Config));
			handle_action(model, action)
		}
		Msg::Selection(msg) => {
				let selection = model.selection.update(msg, &mut orders.proxy(Msg::Selection));
				model.scheduler.update_selection(selection);
		}
		Msg::Delete(id) => {
			model.messages.remove(&id);
		}
		Msg::Ws(message) => match message.json::<Response>() {
			Ok(Response::State(tick, data)) => {
				if let Some(error) = model.scheduler.update_data(tick, data) {
					model
						.messages
						.insert(model.current_id, (true, error.to_string()));
					model.current_id += 1;
				}
			}
			Ok(Response::Started) => log!("simulation started"),
			Ok(Response::Info(info)) => {
				model.messages.insert(model.current_id, (false, info));
				model.current_id += 1;
			}
			Ok(Response::Error(error)) => {
				model.messages.insert(model.current_id, (true, error));
				model.current_id += 1;
			}
			Ok(Response::Stopped) => {
				model.config.stop();
				model.scheduler.stop();
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

fn view_messages(messages: &HashMap<usize, (bool, String)>) -> Vec<Node<Msg>> {
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
		.collect()
}

fn view(model: &Model) -> Node<Msg> {
	div![
		C!["columns"],
		div![
			C!["column is-8 ml-4 mt-5"],
			attrs! {At::Id => "main"},
			view_messages(&model.messages),
			model.selection.view().map_msg(Msg::Selection)
	,		canvas![attrs! {At::Id => "canvas", At::Width => "800", At::Height => "1000"}],
		],
		div![C!["column"], model.config.view().map_msg(Msg::Config)],
	]
}

#[wasm_bindgen(start)]
pub fn start() {
	App::start("app", init, update, view);
}

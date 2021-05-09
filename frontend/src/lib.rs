mod api;
mod components;
mod fields;
mod forms;
mod graphs;

use seed::{prelude::*, *};

use crate::api::{query, Config, InitConfig, State};
use crate::forms::{Action, ConfigForm};
use crate::graphs::simple;
use std::collections::HashMap;
use std::future::Future;

#[derive(Debug)]
pub enum Msg {
	Config(crate::forms::config::Msg),
	Query(Result<State, &'static str>),
	Delete(usize),
	Result(Result<(), &'static str>),
}

struct Model {
	config: ConfigForm,
	history: Vec<State>,
	started: bool,
	crashed: bool, // a bool indicating something has gone horribly wrong and everything needs to be reset

	messages: HashMap<usize, &'static str>,
	current_id: usize,
}

// very dirty trick to make the order code look good
pub fn order<F: Future<Output = ()> + 'static>(
	f: impl FnOnce() -> F + 'static,
	orders: &mut impl Orders<Msg>,
) {
	orders.skip().perform_cmd(async move {
		f().await;
		let t: Option<Msg> = None;
		t
	});
}

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
	Model {
		config: ConfigForm::new(),
		history: vec![],
		started: false,
		crashed: false,
		messages: HashMap::new(),
		current_id: 1,
	}
}

fn handle_error(model: &mut Model, error: &'static str) {
	model.messages.insert(model.current_id, error);
	model.current_id += 1;
	model.crashed = true;

	model.config.stop();
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
	match msg {
		Msg::Config(msg) => match model.config.update(msg, &mut orders.proxy(Msg::Config)) {
			Action::Start(initial, config) => {
				orders.perform_cmd(async {
					cmds::timeout(5000, || ()).await;
					Msg::Query(api::query().await)
				});
				orders.perform_cmd(async { Msg::Result(api::start((initial, config)).await) });
			}
			Action::Update(config) => {
				orders.perform_cmd(async { Msg::Result(api::update(config).await) });
			}
			Action::Stop => {
				model.started = false;
				orders.perform_cmd(async {
					let _ = api::stop().await;
					Option::<Msg>::None
				});
			}
			Action::None => (),
		},

		Msg::Query(result) => match result {
			Ok(state) => {
				log!("i'm here");

				if model.started {
					orders.perform_cmd(async {
						cmds::timeout(5000, || ()).await;
						Msg::Query(api::query().await)
					});
				}

				model.history.push(state);
				simple::draw("canvas", &model.history).expect("could not draw");
			}
			Err(err) => {
				log!("i'm here it failed", err);
				handle_error(model, err);
				orders.perform_cmd(async { Msg::Result(api::stop().await) });
			}
		},
		Msg::Result(result) => {
			if let Err(err) = result {
				handle_error(model, err);
				orders.perform_cmd(async {
					let _ = api::stop().await;
					Option::<Msg>::None
				});
			}
		}
		Msg::Delete(id) => {
			model.messages.remove(&id);
		}
	}
}

fn view_messages(messages: &HashMap<usize, &'static str>) -> Vec<Node<Msg>> {
	messages
		.iter()
		.map(|(id, msg)| {
			let copy = *id;
			div![
				C!["notification"],
				button![C!["delete"], ev(Ev::Click, move |_| Msg::Delete(copy))],
				msg
			]
		})
		.collect()
}

fn view(model: &Model) -> Node<Msg> {
	div![
		C!["columns"],
		div![
			C!["column is-8"],
			view_messages(&model.messages),
			canvas![attrs! {At::Id => "canvas", At::Width => "600", At::Height => "400"}]
		],
		div![C!["column"], model.config.view().map_msg(Msg::Config)],
	]
}

#[wasm_bindgen(start)]
pub fn start() {
	App::start("app", init, update, view);
}

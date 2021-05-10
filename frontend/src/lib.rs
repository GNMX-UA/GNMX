mod api;
mod components;
mod fields;
mod forms;
mod graphs;

use seed::{prelude::*, *};

use crate::api::{query, Config, InitConfig, Patch, State};
use crate::forms::{Action, ConfigForm};
use crate::graphs::{area, line};
use std::collections::HashMap;
use std::future::Future;

#[derive(Debug)]
pub enum Msg {
	Config(crate::forms::config::Msg),
	Query(Result<State, String>),
	Delete(usize),
	Result(Result<(), String>),
}
pub struct GraphData {
	population: u64,
	phenotype_variance: f64,
	phenotype_distance: f64,
}

struct Model {
	config: ConfigForm,
	history: HashMap<u64, GraphData>,
	started: bool,

	messages: HashMap<usize, String>,
	current_id: usize,
}

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
	Model {
		config: ConfigForm::new(),
		history: HashMap::new(),
		started: false,
		messages: HashMap::new(),
		current_id: 1,
	}
}

fn handle_error(model: &mut Model, error: String) {
	model.messages.insert(model.current_id, error);
	model.current_id += 1;

	model.config.stop();
}

fn extract_graph_data(patches: Vec<Patch>) -> Option<GraphData> {
	let phenotypes: Vec<_> = patches
		.iter()
		.map(|patch| {
			patch
				.individuals
				.iter()
				.map(|indiv| indiv.loci.iter().sum())
		})
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
			.map(|patch| patch.individuals.len())
			.sum::<usize>() as u64,
		phenotype_variance: variance,
		phenotype_distance: max? - min?,
	})
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
	match msg {
		Msg::Config(msg) => match model.config.update(msg, &mut orders.proxy(Msg::Config)) {
			Action::Start(initial, config) => {
				model.started = true;
				orders.perform_cmd(async {
					cmds::timeout(100, || ()).await;
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
				if model.started {
					orders.perform_cmd(async {
						cmds::timeout(100, || ()).await;
						Msg::Query(api::query().await)
					});
				}

				if let Some(data) = extract_graph_data(state.patches) {
					model.history.insert(state.tick, data);
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
			Err(err) => {
				model.started = false;
				handle_error(model, err);
				orders.perform_cmd(async {
					let _ = api::stop().await;
					Option::<Msg>::None
				});
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
			canvas![attrs! {At::Id => "canvas_pop", At::Width => "600", At::Height => "400"}],
			canvas![attrs! {At::Id => "canvas_pheno", At::Width => "600", At::Height => "400"}],
			canvas![attrs! {At::Id => "canvas_var", At::Width => "600", At::Height => "400"}],
			canvas![attrs! {At::Id => "canvas_dist", At::Width => "600", At::Height => "400"}]
		],
		div![C!["column"], model.config.view().map_msg(Msg::Config)],
	]
}

#[wasm_bindgen(start)]
pub fn start() {
	App::start("app", init, update, view);
}

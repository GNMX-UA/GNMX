mod api;
mod components;
mod fields;
mod forms;
mod graphs;

use seed::{prelude::*, *};

use crate::api::{Config, InitConfig};
use crate::forms::{Action, ConfigForm};
use crate::graphs::{line, scatter};
use ord_subset::OrdSubsetIterExt;
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
	Info(String),
	State(u64, GraphData),
	Error(String),
}

#[derive(Debug)]
pub enum Msg {
	Config(crate::forms::config::Msg),
	Delete(usize),
	Draw(u64),
	Resize,
	Ws(WebSocketMessage),
}

struct Model {
	config: ConfigForm,
	ws: WebSocket,

	history: Vec<(u64, GraphData)>,
	drawing: bool,
	ranges: GraphRanges,

	messages: HashMap<usize, (bool, String)>,
	current_id: usize,
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
	orders.stream(streams::window_event(Ev::Resize, |_| Msg::Resize));
	Model {
		config: ConfigForm::new(),
		history: vec![],
		drawing: false,
		ranges: Default::default(),
		ws: WebSocket::builder("ws://127.0.0.1:3030/ws", orders)
			.on_message(Msg::Ws)
			.build_and_open()
			.expect("websocket could not connect"),
		messages: HashMap::new(),
		current_id: 1,
	}
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

fn min_assign(a: &mut f64, b: f64) {
	*a = a.min(b);
}

fn max_assign(a: &mut f64, b: f64) {
	*a = a.max(b);
}

fn update_ranges(data: &GraphData, ranges: &mut GraphRanges) {
	min_assign(&mut ranges.population.start, data.population as f64);
	max_assign(&mut ranges.population.end, data.population as f64);

	min_assign(&mut ranges.phenotype_variance.start, data.phenotype_variance);
	max_assign(&mut ranges.phenotype_variance.end, data.phenotype_variance);

	min_assign(&mut ranges.phenotype_distance.start, data.phenotype_distance);
	max_assign(&mut ranges.phenotype_distance.end, data.phenotype_distance);

	let sample_min = data.phenotype_sample
		.iter()
		.map(|x| x.1)
		.ord_subset_min()
		.unwrap();

	let sample_max = data.phenotype_sample
		.iter()
		.map(|x| x.1)
		.ord_subset_max()
		.unwrap();

	min_assign(&mut ranges.phenotype_sample.start, sample_min);
	max_assign(&mut ranges.phenotype_sample.end, sample_max);
}

fn draw_graphs(model: &mut Model) -> Duration {
	let start = Instant::now();

	line::draw(
		"canvas_pop",
		&model.history,
		|data| data.population as f64,
		model.ranges.population.clone(),
		"population size",
	)
	.expect("could not draw");

	scatter::draw(
		"canvas_pheno",
		&model.history,
		model.ranges.phenotype_sample.clone(),
		"phenotypes per patch",
	)
	.expect("could not draw");

	line::draw(
		"canvas_var",
		&model.history,
		|data| data.phenotype_variance,
		model.ranges.phenotype_variance.clone(),
		"phenotypes variation",
	)
	.expect("could not draw");

	let mapper = |data: &GraphData| data.phenotype_distance;
	line::draw(
		"canvas_dist",
		&model.history,
		mapper,
		model.ranges.phenotype_distance.clone(),
		"phenotypes distance",
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

				if !model.drawing {
					orders.send_msg(Msg::Draw(tick));
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
				model.history.clear();
				model.drawing = false;
			}
			Err(err) => log!(err),
		},
		Msg::Resize => {
			// TODO: maybe make resize optional if problem found
			let document = window().document().expect("window has no document");
			let width = document
				.get_element_by_id("canvasses")
				.expect("could not find element canvasses")
				.dyn_into::<web_sys::HtmlDivElement>()
				.expect("could not turn canvasses into div element")
				.offset_width() as u32;

			let resizer = |id: &str| {
				document
					.get_element_by_id(id)
					.expect("could not find canvas element")
					.dyn_into::<web_sys::HtmlCanvasElement>()
					.expect("could not turn canvas into canvas element")
					.set_width(width);
			};

			resizer("canvas_pop");
			resizer("canvas_pheno");
			resizer("canvas_var");
			resizer("canvas_dist");
		}
		Msg::Draw(last) if model.history.len() > 0 => {
			model.drawing = true;
			let tick = model.history.last().unwrap().0;

			// TODO: why the fuck does matching not work
			let duration = match tick {
				_ if last == tick => Duration::from_millis(100),
				_ => draw_graphs(model)
			};

			let cmd = cmds::timeout(duration.as_millis() as u32, move || Msg::Draw(tick));
			orders.perform_cmd(cmd);
		}
		Msg::Draw(_) => {}
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
			attrs! {At::Id => "canvasses"},
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

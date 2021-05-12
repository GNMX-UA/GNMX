use seed::{prelude::*, *};

use crate::api::{Config, InitConfig, Suggestion, Suggestions, TempEnum};
use crate::components::Button;
use crate::fields::slider::SliderField;
use crate::fields::{Field, InputField, SelectField};
use seed::futures::StreamExt;

#[derive(Clone, Debug)]
pub enum Msg {
	TMax(<InputField<String> as Field>::Msg),
	PopulationSize(<InputField<u64> as Field>::Msg),
	PopulationType(<SelectField as Field>::Msg),
	EnvironmentSize(<InputField<u64> as Field>::Msg),
	EnvironmentType(<SelectField as Field>::Msg),

	MutationMu(<SliderField as Field>::Msg),
	MutationSigma(<SliderField as Field>::Msg),
	MutationStep(<SliderField as Field>::Msg),
	Rec(<SliderField as Field>::Msg),
	RMax(<InputField<f64> as Field>::Msg),
	SelectionSigma(<SliderField as Field>::Msg),
	Gamma(<SliderField as Field>::Msg),
	Diploid(<InputField<bool> as Field>::Msg),
	M(<SliderField as Field>::Msg),

	Start,
	Update,
	Stop,
	Pause,
	Resume
}

pub struct ConfigForm {
	// initial values
	t_max: InputField<u64>,
	population_size: InputField<u64>,
	population_type: SelectField,
	patch_amount: InputField<u64>,
	patch_type: SelectField,

	// configurable values
	mutation_mu: SliderField,
	mutation_sigma: SliderField,
	mutation_step: SliderField,
	rec: SliderField,
	r_max: InputField<f64>,
	selection_sigma: SliderField,
	gamma: SliderField,
	diploid: InputField<bool>,
	m: SliderField,

	// buttons
	start: Button<Msg>,
	update: Button<Msg>,
	stop: Button<Msg>,
	pause: Button<Msg>,
	resume: Button<Msg>,

	// state
	started: bool,
	paused: bool,
}

pub enum Action {
	Start(InitConfig, Config),
	Update(Config),
	Pause,
	Resume,
	Stop,
	None,
}

fn make_suggestions(names: &[&str]) -> Suggestions {
	names
		.iter()
		.enumerate()
		.map(
			(|(i, s)| Suggestion {
				name: s.to_string(),
				value: i as i64,
			}),
		)
		.collect()
}

impl ConfigForm {
	pub fn new() -> Self {
		let pop_types = make_suggestions(&["uniform", "normal", "equal"]);
		let env_type = make_suggestions(&["uniform", "normal", "equal"]);

		Self {
			t_max: InputField::new("Ticks", false)
				.with_placeholder("leave empty to run indefinitely")
				.with_initial(Some(100_000)),
			population_size: InputField::new("Population size", false).with_initial(Some(6_000)),
			population_type: SelectField::new("Type", pop_types, false),
			patch_amount: InputField::new("Patch amount", false).with_initial(Some(2)),
			patch_type: SelectField::new("Type", env_type, false),

			mutation_mu: SliderField::new("Mutation Mu", 0.0..1., 0.01),
			mutation_sigma: SliderField::new("Mutation Sigma", 0.0..1., 0.01),
			mutation_step: SliderField::new("Mutation Step", 0.01..1., 0.01),
			rec: SliderField::new("Recombinational probability", 0.0..1., 0.01),
			r_max: InputField::new("Max amount of offspring", false).with_initial(Some(1000.)),
			selection_sigma: SliderField::new("Selection Strength (Sigma)", 0.01..1., 0.01),
			gamma: SliderField::new("Generation Overlap (Gamma)", 0.0..1., 0.01),
			diploid: InputField::new("Diploid", false).with_initial(Some(false)),
			m: SliderField::new("Dispersal parameter (M)", 0.0..1., 0.01),
			start: Button::new("start", "is-success", "fa-play", || Msg::Start),
			update: Button::new("update", "is-link", "fa-wrench", || Msg::Update),
			stop: Button::new("stop", "is-danger is-outlined", "fa-square", || Msg::Stop),
			pause: Button::pause(|| Msg::Pause),
			resume: Button::resume(|| Msg::Resume),
			started: false,
			paused: false
		}
	}

	pub fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) -> Action {
		match msg {
			Msg::Start => {
				return match self.extract() {
					Some((a, b)) => {
						self.started = true;
						Action::Start(a, b)
					}
					None => Action::None,
				}
			}
			Msg::Update => {
				return match self.extract_config() {
					Some(a) => Action::Update(a),
					None => Action::None,
				}
			}
			Msg::Stop => {
				self.started = false;
				self.paused = false;
				return Action::Stop;
			}
			Msg::Pause => {
				self.paused = true;
				return Action::Pause
			},
			Msg::Resume => {
				self.paused = false;
				return Action::Resume
			}
			_ => (),
		}

		// so many copy pasta but oh well
		match msg {
			Msg::TMax(msg) => self.t_max.update(msg, &mut orders.proxy(Msg::TMax)),
			Msg::PopulationSize(msg) => self
				.population_size
				.update(msg, &mut orders.proxy(Msg::PopulationSize)),
			Msg::PopulationType(msg) => self
				.population_type
				.update(msg, &mut orders.proxy(Msg::PopulationType)),
			Msg::EnvironmentSize(msg) => self
				.patch_amount
				.update(msg, &mut orders.proxy(Msg::EnvironmentSize)),
			Msg::EnvironmentType(msg) => self
				.patch_type
				.update(msg, &mut orders.proxy(Msg::EnvironmentType)),
			Msg::MutationMu(msg) => self
				.mutation_mu
				.update(msg, &mut orders.proxy(Msg::MutationMu)),
			Msg::MutationSigma(msg) => self
				.mutation_sigma
				.update(msg, &mut orders.proxy(Msg::MutationSigma)),
			Msg::MutationStep(msg) => self
				.mutation_step
				.update(msg, &mut orders.proxy(Msg::MutationStep)),
			Msg::Rec(msg) => self.rec.update(msg, &mut orders.proxy(Msg::Rec)),
			Msg::RMax(msg) => self.r_max.update(msg, &mut orders.proxy(Msg::RMax)),
			Msg::SelectionSigma(msg) => self
				.selection_sigma
				.update(msg, &mut orders.proxy(Msg::SelectionSigma)),
			Msg::Gamma(msg) => self.gamma.update(msg, &mut orders.proxy(Msg::Gamma)),
			Msg::Diploid(msg) => self.diploid.update(msg, &mut orders.proxy(Msg::Diploid)),
			Msg::M(msg) => self.m.update(msg, &mut orders.proxy(Msg::M)),
			_ => unreachable!("all other cases must be handled in previous match"),
		}
		Action::None
	}

	pub fn stop(&mut self) {
		self.started = false;
	}

	fn extract_initial(&self) -> Option<InitConfig> {
		let t_max = self.t_max.value(true);
		let population_size = self.population_size.value(true);
		let population_type = self.population_type.value(true);
		let patch_amount = self.patch_amount.value(true);
		let patch_type = self.patch_type.value(true);

		// Some(InitConfig {
		// 	t_max,
		// 	population_size: population_size?,
		// 	population_type: population_type?,
		// 	patch_amount: patch_amount?,
		// 	patch_type: patch_type?,
		// })

		Some(InitConfig {
			t_max,
			kind: TempEnum::Default,
		})
	}

	fn extract_config(&self) -> Option<Config> {
		let mutation_mu = self.mutation_mu.value(true);
		let mutation_sigma = self.mutation_sigma.value(true);
		let mutation_step = self.mutation_step.value(true);
		let rec = self.rec.value(true);
		let r_max = self.r_max.value(true);
		let selection_sigma = self.selection_sigma.value(true);
		let gamma = self.gamma.value(true);
		let diploid = self.diploid.value(true);
		let m = self.m.value(true);

		Some(Config {
			mutation_mu: mutation_mu?,
			mutation_sigma: mutation_sigma?,
			mutation_step: mutation_step?,
			rec: rec?,
			r_max: r_max?,
			selection_sigma: selection_sigma?,
			gamma: gamma?,
			diploid: diploid?,
			m: m?,
		})
	}

	fn extract(&self) -> Option<(InitConfig, Config)> {
		Some((self.extract_initial()?, self.extract_config()?))
	}

	pub fn view(&self) -> Node<Msg> {
		div![
			C!["p-6"],
			style! {St::BoxShadow => "-10px 0px 10px 1px #eeeeee"},
			self.t_max.view(self.started).map_msg(Msg::TMax),
			div![
				C!["columns"],
				div![
					C!["column"],
					self.population_size
						.view(false)
						.map_msg(Msg::PopulationSize)
				],
				div![
					C!["column is-narrow"],
					self.population_type
						.view(false)
						.map_msg(Msg::PopulationType)
				],
			],
			div![
				C!["columns"],
				div![
					C!["column"],
					self.patch_amount
						.view(false)
						.map_msg(Msg::EnvironmentSize)
				],
				div![
					C!["column is-narrow"],
					self.patch_type
						.view(false)
						.map_msg(Msg::EnvironmentType)
				],
			],
			hr![],
			self.mutation_mu.view(false).map_msg(Msg::MutationMu),
			self.mutation_sigma.view(false).map_msg(Msg::MutationSigma),
			self.mutation_step.view(false).map_msg(Msg::MutationStep),
			hr![],
			self.rec.view(false).map_msg(Msg::Rec),
			self.r_max.view(false).map_msg(Msg::RMax),
			self.selection_sigma
				.view(false)
				.map_msg(Msg::SelectionSigma),
			self.gamma.view(false).map_msg(Msg::Gamma),
			self.diploid.view(false).map_msg(Msg::Diploid),
			self.m.view(false).map_msg(Msg::M),
			div![
				C!["buttons pt-4"],
				self.start.view(false, self.started),
				self.stop.view(false, !self.started),
				self.resume.view(!self.started, !self.paused),
				self.pause.view(!self.started, self.paused),
				self.update.view(!self.started, false),
			]
		]
	}
}

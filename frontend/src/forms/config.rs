use seed::{prelude::*, *};

use crate::api::{Config, Environment, InitConfig, Suggestion, Suggestions};
use crate::components::Button;
use crate::fields::slider::SliderField;
use crate::fields::{Field, InputField, SelectField};
use seed::futures::StreamExt;

#[derive(Clone, Debug)]
pub enum Msg {
	MutationMu(<SliderField as Field>::Msg),
	MutationSigma(<SliderField as Field>::Msg),
	MutationStep(<SliderField as Field>::Msg),
	Environment(<SelectField as Field>::Msg),
	Rec(<SliderField as Field>::Msg),
	SelectionSigma(<SliderField as Field>::Msg),
	Gamma(<SliderField as Field>::Msg),
	Diploid(<InputField<bool> as Field>::Msg),
	M(<SliderField as Field>::Msg),
}

pub struct ConfigForm {
	mutation_mu: SliderField,
	mutation_sigma: SliderField,
	mutation_step: SliderField,
	rec: SliderField,
	environment: SelectField,
	selection_sigma: SliderField,
	gamma: SliderField,
	diploid: InputField<bool>,
	m: SliderField,
}

impl ConfigForm {
	pub fn new() -> Self {
		Self {
			mutation_mu: SliderField::new("Mutation Mu", 0.0..1., 0.01),
			mutation_sigma: SliderField::new("Mutation Sigma", 0.0..1., 0.01),
			mutation_step: SliderField::new("Mutation Step", 0.01..1., 0.01),
			rec: SliderField::new("Recombinational probability", 0.0..1., 0.01),
			environment: SelectField::new("Environment", vec![], false),
			selection_sigma: SliderField::new("Selection Strength (Sigma)", 0.01..1., 0.01),
			gamma: SliderField::new("Generation Overlap (Gamma)", 0.0..1., 0.01),
			diploid: InputField::new("Diploid", false).with_initial(Some(false)),
			m: SliderField::new("Dispersal parameter (M)", 0.0..1., 0.01),
		}
	}

	pub fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) -> bool {
		// so much copy pasta but oh well
		match msg {
			Msg::MutationMu(msg) => self
				.mutation_mu
				.update(msg, &mut orders.proxy(Msg::MutationMu)),
			Msg::MutationSigma(msg) => self
				.mutation_sigma
				.update(msg, &mut orders.proxy(Msg::MutationSigma)),
			Msg::MutationStep(msg) => self
				.mutation_step
				.update(msg, &mut orders.proxy(Msg::MutationStep)),
			Msg::Environment(msg) => self
				.environment
				.update(msg, &mut orders.proxy(Msg::Environment)),
			Msg::Rec(msg) => self.rec.update(msg, &mut orders.proxy(Msg::Rec)),
			Msg::SelectionSigma(msg) => self
				.selection_sigma
				.update(msg, &mut orders.proxy(Msg::SelectionSigma)),
			Msg::Gamma(msg) => self.gamma.update(msg, &mut orders.proxy(Msg::Gamma)),
			Msg::Diploid(msg) => self.diploid.update(msg, &mut orders.proxy(Msg::Diploid)),
			Msg::M(msg) => self.m.update(msg, &mut orders.proxy(Msg::M)),
		}
	}

	pub fn extract(&self) -> Option<Config> {
		let mutation_mu = self.mutation_mu.value(true);
		let mutation_sigma = self.mutation_sigma.value(true);
		let mutation_step = self.mutation_step.value(true);
		let rec = self.rec.value(true);
		let selection_sigma = self.selection_sigma.value(true);
		let gamma = self.gamma.value(true);
		let diploid = self.diploid.value(true);
		let m = self.m.value(true);
		let environment = self.environment.value(true);

		let environment = match environment {
			Some(0) => Environment::Random,
			Some(1) => Environment::AlternatingHalf,
			Some(2) => Environment::AlternatingThird,
			Some(3) => Environment::Sine,
			Some(4) => Environment::RandomWalk,
			Some(5) => Environment::Constant,
			Some(6) => Environment::ConstantWithJumps,
			Some(_) | None => return None,
		};

		Some(Config {
			mutation_mu: mutation_mu?,
			mutation_sigma: mutation_sigma?,
			mutation_step: mutation_step?,
			rec: rec?,
			selection_sigma: selection_sigma?,
			gamma: gamma?,
			diploid: diploid?,
			m: m?,
			environment,
		})
	}

	pub fn view(&self) -> Node<Msg> {
		div![
			self.mutation_mu.view(false).map_msg(Msg::MutationMu),
			self.mutation_sigma.view(false).map_msg(Msg::MutationSigma),
			self.mutation_step.view(false).map_msg(Msg::MutationStep),
			hr![],
			self.environment.view(false).map_msg(Msg::Environment),
			self.rec.view(false).map_msg(Msg::Rec),
			self.selection_sigma
				.view(false)
				.map_msg(Msg::SelectionSigma),
			self.gamma.view(false).map_msg(Msg::Gamma),
			self.diploid.view(false).map_msg(Msg::Diploid),
			self.m.view(false).map_msg(Msg::M),
		]
	}
}

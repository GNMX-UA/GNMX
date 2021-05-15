use seed::{futures::StreamExt, prelude::*, *};

use crate::api::make_suggestions;
use crate::{
	api::{Config, InitConfig, InitialPopulation, Suggestion, Suggestions},
	components::Button,
	fields::{slider::SliderField, Field, InputField, SelectField},
};

#[derive(Clone, Debug)]
pub enum Msg {
	TMax(<InputField<String> as Field>::Msg),
	Kind(<SelectField as Field>::Msg),
	Individuals(<InputField<u64> as Field>::Msg),
	Patches(<InputField<u64> as Field>::Msg),
	Loci(<InputField<u64> as Field>::Msg),
	Diploid(<InputField<bool> as Field>::Msg),
}

pub struct InitConfigForm {
	t_max: InputField<u64>,
	kind: SelectField,
	individuals: InputField<u64>,
	patches: InputField<u64>,
	loci: InputField<u64>,
	diploid: InputField<bool>,
}

impl InitConfigForm {
	pub fn new() -> Self {
		let kind_suggestions = make_suggestions(&[
			"Loci uniformly distributed in individual",
			"Loci uniformly distributed in patch",
			"Loci uniformly distributed in population",
			"Loci constant in individual",
			"Loci constant in patch",
			"Loci constant in population",
			"Loci normally distributed in individual",
			"Loci normally distributed in patch",
			"Loci normally distributed in population",
			"Alternating with 50% chance",
			"Alternating with 67% chance"
		]);

		Self {
			t_max: InputField::new("Ticks", true)
				.with_placeholder("leave empty to run indefinitely"),
			kind: SelectField::new("Initial population distribution", kind_suggestions, false),
			individuals: InputField::new("Population size", false).with_initial(Some(10_000))
				.with_validator(|&value| (value == 0).then(|| "Number must be strictly positive.".to_string())),
			patches: InputField::new("Patch amount", false).with_initial(Some(6))
				.with_validator(|&value| (value == 0).then(|| "Number must be strictly positive.".to_string())),
			loci: InputField::new("Locus amount", false).with_initial(Some(1))
				.with_validator(|&value| (value == 0).then(|| "Number must be strictly positive.".to_string())),
			diploid: InputField::new("Diploid", false).with_initial(Some(false)),
		}
	}

	pub fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) -> bool {
		// so much copy pasta but oh well
		match msg {
			Msg::TMax(msg) => self.t_max.update(msg, &mut orders.proxy(Msg::TMax)),
			Msg::Kind(msg) => self.kind.update(msg, &mut orders.proxy(Msg::Kind)),
			Msg::Individuals(msg) => self
				.individuals
				.update(msg, &mut orders.proxy(Msg::Individuals)),
			Msg::Patches(msg) => self.patches.update(msg, &mut orders.proxy(Msg::Patches)),
			Msg::Loci(msg) => self.loci.update(msg, &mut orders.proxy(Msg::Loci)),
			Msg::Diploid(msg) => self.diploid.update(msg, &mut orders.proxy(Msg::Diploid)),
		}
	}

	pub fn extract(&self) -> Option<InitConfig> {
		let t_max = self.t_max.value(true);
		let kind = self.kind.value(true);
		let individuals = self.individuals.value(true);
		let patches = self.patches.value(true);
		let loci = self.loci.value(true);
		let diploid = self.diploid.value(true);

		let kind = match kind {
			Some(0) => InitialPopulation::UniformI,
			Some(1) => InitialPopulation::UniformP,
			Some(2) => InitialPopulation::Uniform,
			Some(3) => InitialPopulation::ConstantI,
			Some(4) => InitialPopulation::ConstantP,
			Some(5) => InitialPopulation::Constant,
			Some(6) => InitialPopulation::NormalI,
			Some(7) => InitialPopulation::NormalP,
			Some(8) => InitialPopulation::Normal,
			Some(9) => InitialPopulation::AlternatingHalf,
			Some(10) => InitialPopulation::AlternatingThird,
			Some(_) | None => return None,
		};

		Some(InitConfig {
			t_max,
			kind,
			individuals: individuals? as usize,
			patches: patches? as usize,
			loci: loci? as usize,
			diploid: diploid?,
		})
	}

	pub fn view(&self, disabled: bool) -> Node<Msg> {
		div![
			self.t_max.view(disabled).map_msg(Msg::TMax),
			self.kind.view(disabled).map_msg(Msg::Kind),
			self.individuals.view(disabled).map_msg(Msg::Individuals),
			self.patches.view(disabled).map_msg(Msg::Patches),
			self.loci.view(disabled).map_msg(Msg::Loci),
			hr![],
			self.diploid.view(disabled).map_msg(Msg::Diploid),
		]
	}
}

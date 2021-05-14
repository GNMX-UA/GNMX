use seed::{futures::StreamExt, prelude::*, *};

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
}

pub struct InitConfigForm {
	t_max: InputField<u64>,
	kind: SelectField,
	individuals: InputField<u64>,
	patches: InputField<u64>,
	loci: InputField<u64>,
}

impl InitConfigForm {
	pub fn new() -> Self {
		let kind_suggestions = make_suggestions(&["uniform", "normal", "equal"]);

		Self {
			t_max:       InputField::new("Ticks", false)
				.with_placeholder("leave empty to run indefinitely"),
			kind:        SelectField::new(
				"Initial population distribution",
				kind_suggestions,
				false,
			),
			individuals: InputField::new("Population size", false).with_initial(Some(6_000)),
			patches:     InputField::new("Patch amount", false).with_initial(Some(5)),
			loci:        InputField::new("Locus amount", false).with_initial(Some(2)),
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
		}
	}

	pub fn extract(&self) -> Option<InitConfig> {
		let t_max = self.t_max.value(true);
		let kind = self.kind.value(true);
		let individuals = self.individuals.value(true);
		let patches = self.patches.value(true);
		let loci = self.loci.value(true);

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
		})
	}

	pub fn view(&self, disabled: bool) -> Node<Msg> {
		div![
			self.t_max.view(disabled).map_msg(Msg::TMax),
			self.kind.view(disabled).map_msg(Msg::Kind),
			self.individuals.view(disabled).map_msg(Msg::Individuals),
			self.patches.view(disabled).map_msg(Msg::Patches),
			self.loci.view(disabled).map_msg(Msg::Loci),
		]
	}
}

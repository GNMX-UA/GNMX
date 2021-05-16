use seed::{futures::StreamExt, prelude::*, *};

use crate::{
	api::{Config, InitConfig, Suggestion, Suggestions},
	components::Button,
	fields::{slider::SliderField, Field, InputField, SelectField},
};

pub struct SimulationConfig {
	pub gamer_mode: bool,
}

#[derive(Clone, Debug)]
pub enum Msg {
	GamerMode(<InputField<bool> as Field>::Msg),
}

pub struct SimulationForm {
	gamer_mode: InputField<bool>,
}

impl SimulationForm {
	pub fn new() -> Self {
		Self {
			gamer_mode: InputField::new("Precision mode", false).with_initial(Some(false)),
		}
	}

	pub fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) -> bool {
		// so much copy pasta but oh well
		match msg {
			Msg::GamerMode(msg) => self
				.gamer_mode
				.update(msg, &mut orders.proxy(Msg::GamerMode)),
		}
	}

	pub fn extract(&self) -> Option<SimulationConfig> {
		let gamer_mode = self.gamer_mode.value(true);

		Some(SimulationConfig {
			gamer_mode: gamer_mode?,
		})
	}

	pub fn view(&self, disabled: bool) -> Node<Msg> {
		div![self.gamer_mode.view(disabled).map_msg(Msg::GamerMode)]
	}
}

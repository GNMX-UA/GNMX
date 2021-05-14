use seed::{prelude::*, *};

use crate::api::{Config, InitConfig, Suggestion, Suggestions, TempEnum};
use crate::components::Button;
use crate::fields::slider::SliderField;
use crate::fields::{Field, InputField, SelectField};
use seed::futures::StreamExt;

pub struct SimulationConfig {
    gamer_mode: bool
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
            gamer_mode: InputField::new("Gamer mode 😎", false).with_initial(Some(false)),
        }
    }

    pub fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) -> bool {
        // so much copy pasta but oh well
        match msg {
            Msg::GamerMode(msg) => self.gamer_mode.update(msg, &mut orders.proxy(Msg::GamerMode)),
        }
    }

    fn extract(&self) -> Option<SimulationConfig> {
        let gamer_mode = self.gamer_mode.value(true);

        Some(SimulationConfig{gamer_mode: gamer_mode?})
    }

    pub fn view(&self) -> Node<Msg> {
        div![self.gamer_mode.view(false).map_msg(Msg::GamerMode)]
    }
}

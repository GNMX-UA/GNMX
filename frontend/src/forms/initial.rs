use seed::{prelude::*, *};

use crate::api::Initial;
use crate::components::Button;
use crate::fields::{Field, InputField};

#[derive(Clone, Debug)]
pub enum Msg {
    Ticks(<InputField<String> as Field>::Msg),
    Start,
}

pub struct InitialForm {
    ticks: InputField<u64>,
    start: Button<Msg>,
}

impl InitialForm {
    pub fn new() -> Self {
        Self {
            ticks: InputField::new("Ticks", false).with_placeholder("leave empty to run indefinitely"),
            start: Button::new("start simulation", "is-success", "fa-play", || Msg::Start),
        }
    }

    pub fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) -> Option<Initial> {
        match msg {
            Msg::Start => self.extract(true),
            Msg::Ticks(msg) => {
                self.ticks.update(msg, &mut orders.proxy(Msg::Ticks));
                None
            }
        }
    }

    fn extract(&self, submit: bool) -> Option<Initial> {
        let ticks = self.ticks.value(submit);

        Some(Initial { ticks })
    }

    pub fn view(&self) -> Node<Msg> {
        div![
            C!["box"],
            self.ticks.view().map_msg(Msg::Ticks),
            self.start.view(false)
        ]
    }
}

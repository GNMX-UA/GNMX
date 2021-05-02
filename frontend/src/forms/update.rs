use seed::{prelude::*, *};

use crate::api::Update;
use crate::components::Button;
use crate::fields::{Field, InputField};

#[derive(Clone, Debug)]
pub enum Msg {
    Param1(<InputField<String> as Field>::Msg),
    Update,
}

pub struct UpdateForm {
    param1: InputField<String>,
    update: Button<Msg>,
}

impl UpdateForm {
    pub fn new() -> Self {
        Self {
            param1: InputField::new("param1", false),
            update: Button::new("update parameters", "is-link", "fa-wrench", || Msg::Update),
        }
    }

    pub fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) -> Option<Update> {
        match msg {
            Msg::Update => self.extract(true),
            Msg::Param1(msg) => {
                self.param1.update(msg, &mut orders.proxy(Msg::Param1));
                None
            }
        }
    }

    fn extract(&self, submit: bool) -> Option<Update> {
        let param1 = self.param1.value(submit);

        Some(Update { param1: param1? })
    }

    pub fn view(&self) -> Node<Msg> {
        div![
            C!["box"],
            self.param1.view().map_msg(Msg::Param1),
            self.update.view(false)
        ]
    }
}

use seed::{prelude::*, *};

use crate::api::{Config, InitConfig, Suggestion, Suggestions};
use crate::components::Button;
use crate::fields::slider::SliderField;
use crate::fields::{Field, InputField, SelectField};
use seed::futures::StreamExt;

pub struct ForgetConfig {
    pub forget: bool
}

#[derive(Clone, Debug)]
pub enum Msg {
    Forget(<InputField<bool> as Field>::Msg),
}

pub struct ForgetForm {
    forget: InputField<bool>,
}

impl ForgetForm {
    pub fn new() -> Self {
        Self {
            forget: InputField::new("Forget", false).with_initial(Some(true)),
        }
    }

    pub fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) -> bool {
        // so much copy pasta but oh well
        match msg {
            Msg::Forget(msg) => self.forget.update(msg, &mut orders.proxy(Msg::Forget)),
        }
    }

    pub fn extract(&self) -> Option<ForgetConfig> {
        let forget = self.forget.value(true);

        Some(ForgetConfig {forget: forget?})
    }

    pub fn view(&self, disabled: bool) -> Node<Msg> {
        div![self.forget.view(disabled).map_msg(Msg::Forget)]
    }
}

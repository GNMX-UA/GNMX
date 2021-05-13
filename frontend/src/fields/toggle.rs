use std::future::Future;
use std::marker::PhantomData;

use seed::prelude::web_sys::{HtmlInputElement, HtmlSelectElement};
use seed::{prelude::*, *};

use crate::api::{Suggestions, Suggestion};

use crate::components::Component;
use crate::fields::Field;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub struct ToggleField {
    label: &'static str,

    initial: bool,
    value: bool,
}

impl ToggleField {
    pub fn new(label: &'static str, initial: bool) -> Self {
        Self {
            label,
            value: initial,
            ..Default::default()
        }
    }
}

impl Field for ToggleField {
    type Msg = bool;
    type Value = bool;

    fn set(&mut self, _: Option<Self::Value>) {}

    fn value(&self, _: bool) -> Option<Self::Value> {
        Some(self.value)
    }

    fn update(&mut self, msg: Self::Msg, _: &mut impl Orders<Self::Msg>) {
        self.value = msg;
    }

    fn view(&self, readonly: bool) -> Vec<Node<Self::Msg>> {
        vec![label![C!["switch"], input![attrs!{At::Type => "checkbox"}], span![C!["toggle"]]]]
    }
}

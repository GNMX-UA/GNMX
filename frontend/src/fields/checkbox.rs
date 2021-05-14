use seed::{prelude::*, *};

use crate::api::{Suggestion, Suggestions};

use crate::components::Component;
use crate::fields::Field;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub struct CheckboxField {
    label: String,
    value: bool,
}

impl CheckboxField {
    pub fn new(label: impl Into<String>, initial: bool) -> Self {
        Self {
            label: label.into(),
            value: initial,
            ..Default::default()
        }
    }
}

impl Field for CheckboxField {
    type Msg = ();
    type Value = bool;

    fn set(&mut self, _: Option<Self::Value>) {}

    fn value(&self, _: bool) -> Option<Self::Value> {
        Some(self.value)
    }

    fn update(&mut self, _: Self::Msg, _: &mut impl Orders<Self::Msg>) -> bool {
        self.value = !self.value;
        true
    }

    fn view(&self, readonly: bool) -> Vec<Node<Self::Msg>> {
        vec![label![
			C!["checkbox"],
			input![attrs! {At::Type => "checkbox", At::Checked => self.value.as_at_value() }],
			input_ev(Ev::Input, |_| ()),
			&self.label
		]]
    }
}

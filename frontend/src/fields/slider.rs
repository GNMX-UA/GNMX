use std::future::Future;
use std::marker::PhantomData;

use seed::prelude::web_sys::{HtmlInputElement, HtmlSelectElement};
use seed::{prelude::*, *};

use crate::api::{Suggestion, Suggestions};

use crate::components::Component;
use crate::fields::Field;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub struct SliderField {
	label: &'static str,

	initial: Option<f64>,
	value: f64,

	range: Range<f64>,
}

impl SliderField {
	pub fn new(label: &'static str, range: Range<f64>, initial: f64) -> Self {
		Self {
			label,
			range,
			initial: Some(initial),
			..Default::default()
		}
	}
}

impl Field for SliderField {
	type Msg = f64;
	type Value = f64;

	fn set(&mut self, value: Option<Self::Value>) {
		self.initial = value;
	}

	fn value(&self, submit: bool) -> Option<Self::Value> {
		Some(self.value)
	}

	fn update(&mut self, msg: Self::Msg, _: &mut impl Orders<Self::Msg>) {
		self.initial = None;
		self.value = msg;
	}

	fn view(&self, readonly: bool) -> Vec<Node<Self::Msg>> {
		vec![div![
			C!["field"],
			label![C!["label"], self.label],
			div![
				C!["control"],
				input![
					C!["slider"],
					attrs! {At::Min => 0, At::Max => 0, At::Type => "range"}
				]
			]
		]]
	}
}

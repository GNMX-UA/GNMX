use std::future::Future;
use std::marker::PhantomData;

use seed::prelude::web_sys::{HtmlInputElement, HtmlSelectElement};
use seed::{prelude::*, *};

use crate::api::{Suggestion, Suggestions};

use crate::components::Component;
use crate::fields::Field;
use std::num::NonZeroU64;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct SliderField {
	label: &'static str,

	value: f64,
	steps: NonZeroU64,

	range: Range<f64>,
}

impl SliderField {
	pub fn new(label: &'static str, range: Range<f64>, initial: f64) -> Self {
		Self {
			label,
			range,
			steps: NonZeroU64::new(100).unwrap(),
			value: initial,
		}
	}
}

impl Field for SliderField {
	type Msg = f64;
	type Value = f64;

	fn set(&mut self, value: Option<Self::Value>) {
		self.value = value.unwrap();
	}

	fn value(&self, submit: bool) -> Option<Self::Value> {
		Some(self.value)
	}

	fn update(&mut self, msg: Self::Msg, _: &mut impl Orders<Self::Msg>) -> bool {
		self.value = msg;
		true
	}

	fn view(&self, readonly: bool) -> Vec<Node<Self::Msg>> {
		vec![div![
			C!["field"],
			label![C!["label"], self.label],
			div![
				C!["control pb-4"],
				input![
					C!["slider"],
					attrs! { At::Min => self.range.start, At::Max => self.range.end,
					At::Step => (self.range.end - self.range.start) / self.steps.get() as f64,
					At::Type => "range", At::Value => self.value },
					input_ev(Ev::Input, |str| str.parse::<f64>().ok()),
				],
				p![
					style! {St::Position => "absolute",
						St::Left => unit!(92.*(self.value - self.range.start) / (self.range.end - self.range.start), %)},
					format!("{:.2}", self.value)
				]
			]
		]]
	}
}

use std::future::Future;
use std::marker::PhantomData;

use seed::prelude::web_sys::{HtmlInputElement, HtmlSelectElement};
use seed::{prelude::*, *};

use crate::api::{Suggestions, Suggestion};

use crate::components::Component;
use crate::fields::types::Shared;
use crate::fields::Field;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub struct SelectField {
    label: &'static str,

    initial: Option<i64>,
    value: Option<i64>,

    suggestions: Shared<Suggestions>,

    optional: bool,
    submitted: AtomicBool,
    loading: bool,
}

impl SelectField {
    pub fn new(label: &'static str, suggestions: Shared<Suggestions>) -> Self {
        Self {
            label,
            suggestions,
            ..Default::default()
        }
    }

    pub fn with_initial(mut self, initial: Option<i64>) -> Self {
        self.initial = initial;
        self.value = initial;
        self
    }

    fn view_option(&self, suggestion: &Suggestion) -> Node<<Self as Field>::Msg> {
        let selected = self.initial == Some(suggestion.value);
        option![
            attrs! {At::Value => suggestion.value, At::Selected => selected.as_at_value()},
            &suggestion.name
        ]
    }

    fn view_options(&self) -> Vec<Node<<Self as Field>::Msg>> {
        self.suggestions
            .borrow()
            .iter()
            .map(|x| self.view_option(x))
            .collect()
    }
}

impl Field for SelectField {
    type Msg = String;
    type Value = i64;

    fn set(&mut self, value: Option<Self::Value>) {
        self.initial = value;
    }

    fn value(&self, submit: bool) -> Option<Self::Value> {
        self.submitted.fetch_or(submit, Ordering::Relaxed);
        self.value
    }

    fn update(&mut self, msg: Self::Msg, _: &mut impl Orders<Self::Msg>) {
        self.initial = None;
        self.value = match msg.parse() {
            Ok(x) if x > 0 => Some(x),
            _ => None,
        }
    }

    fn view(&self) -> Vec<Node<Self::Msg>> {
        vec![div![
            C!["field"],
            label![C!["label"], self.label],
            div![
                C!["control"],
                div![
                    C![
                        "select",
                        IF!(self.value.is_none() && !self.optional && self.submitted.load(Ordering::Relaxed) => "is-danger"),
                        IF!(self.loading => "is-loading")
                    ],
                    select![
                        input_ev(Ev::Input, |str| str),
                        self.view_option(&Suggestion {
                            name: "Select option".to_owned(),
                            value: -1
                        }),
                        self.view_options()
                    ]
                ]
            ]
        ]]
    }
}

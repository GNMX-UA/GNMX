use seed::{prelude::*, *};
use std::str::FromStr;

use crate::fields::{Field, State};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Debug)]
pub enum Msg {
    Blur,
    Value(String)
}

#[derive(Default)]
pub struct InputField<U: Clone + ToString + FromStr + Default>
where
    <U as FromStr>::Err: ToString,
{
    label: &'static str,
    state: State<U>,

    initial: Option<U>,
    value: String,

    placeholder: &'static str,
    validator: Option<Box<dyn Fn(&U) -> Option<String>>>,

    optional: bool,
    submitted: AtomicBool,
}

impl<U: Clone + ToString + FromStr + Default> InputField<U>
where
    <U as FromStr>::Err: ToString,
{
    pub fn new(label: &'static str, optional: bool) -> Self {
        Self {
            label,
            optional,
            ..Default::default()
        }
    }

    pub fn with_initial(mut self, value: Option<U>) -> Self {
        self.set(value.clone());
        self.initial = value;
        self
    }

    pub fn with_placeholder(mut self, placeholder: &'static str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn with_validator(mut self, validator: impl Fn(&U) -> Option<String> + 'static) -> Self {
        self.validator = Some(Box::new(validator));
        self
    }
}

impl<U: Clone + ToString + FromStr + Default> Field for InputField<U>
where
    <U as FromStr>::Err: ToString,
{
    type Msg = Msg;
    type Value = U;

    fn set(&mut self, value: Option<Self::Value>) {
        self.value = value.as_ref().map(|x| x.to_string()).unwrap_or_default();

        self.state = match value {
            Some(x) => State::Value(x),
            None => State::Empty,
        };
    }

    fn value(&self, submit: bool) -> Option<Self::Value> {
        self.submitted.fetch_or(submit, Ordering::Relaxed);
        self.state.ok()
    }

    fn update(&mut self, msg: Self::Msg, _: &mut impl Orders<Self::Msg>) -> bool {
        let mapper = |value: &str| match U::from_str(value) {
            Ok(u) => State::Value(u),
            Err(err) => State::Error(err.to_string()),
        };

        self.state = match (&msg, &self.initial) {
            (Msg::Value(str), _) if str.is_empty() => State::Empty,
            (Msg::Value(str), Some(initial)) if str == &initial.to_string() => {
                State::Value(initial.clone())
            }
            (Msg::Value(str), _) => match (U::from_str(str), &self.validator) {
                (Ok(value), Some(validator)) => match validator(&value) {
                    Some(error) => State::Error(error),
                    None => State::Value(value)
                },
                (Ok(value), _) => State::Value(value),
                (Err(err), _) => State::Error(err.to_string()),
            },
            (Msg::Blur, _) => return true,
        };

        if let Msg::Value(str) = msg {
            self.value = str;
        }
        false
    }

    fn view(&self, readonly: bool) -> Vec<Node<Self::Msg>> {
        let (danger, error) = match &self.state {
            State::Error(error) => (true, error.as_str()),
            State::Empty if !self.optional && self.submitted.load(Ordering::Relaxed) => {
                (true, "This field is required")
            }
            _ => (false, ""),
        };

        vec![
            label![
                C!["label"],
                &self.label,
                IF!(self.optional => i![C!["has-text-grey"], " - Optional"])
            ],
            div![
                C![
                    "control",
                    IF!(danger => "has-icons-right"),
                    IF!(!danger => "mb-2") // use 5 for minimal error drift
                ],
                input![
                    C!["input", IF!(danger => "is-danger")],
                    input_ev(Ev::Input, |str| Msg::Value(str)),
                    ev(Ev::Blur, |_| Msg::Blur),
                    attrs! {At::Placeholder => &self.placeholder, At::Value => &self.value},
                    IF!(readonly => attrs! {At::Disabled => ""}),
                ],
                IF![danger => span![C!["icon is-small is-right"], i![C!["fas", "fa-exclamation-triangle"]]]]
            ],
            p![C!["help", "is-danger"], error],
        ]
    }
}

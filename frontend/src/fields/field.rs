use seed::prelude::{IntoNodes, Node, Orders};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum State<U: Clone + FromStr> {
    Empty,
    Value(U),
    Error(String),
}

impl<U: Clone + FromStr> Default for State<U> {
    fn default() -> Self {
        State::Empty
    }
}

impl<U: Clone + FromStr> State<U> {
    pub fn ok(&self) -> Option<U> {
        match self {
            State::Value(v) => Some(v.clone()),
            _ => None,
        }
    }
}

pub trait Field {
    type Msg: 'static;
    type Value: ToString;

    fn set(&mut self, value: Option<Self::Value>);
    fn value(&self, submit: bool) -> Option<Self::Value>;

    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>);
    fn view(&self, readonly: bool) -> Vec<Node<Self::Msg>>;
}

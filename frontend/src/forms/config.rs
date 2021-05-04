use seed::{prelude::*, *};

use crate::api::{Config, Initial};
use crate::components::Button;
use crate::fields::{Field, InputField};

#[derive(Clone, Debug)]
pub enum Msg {
    Ticks(<InputField<String> as Field>::Msg),
    Start,
    Update,
    Stop,
}

pub struct ConfigForm {
    ticks: InputField<u64>,
    start: Button<Msg>,
    update: Button<Msg>,
    stop: Button<Msg>,

    started: bool,
}

pub enum Action {
    Start(Initial, Config),
    Update(Config),
    Stop,
    None,
}

impl ConfigForm {
    pub fn new() -> Self {
        Self {
            ticks: InputField::new("Ticks", false)
                .with_placeholder("leave empty to run indefinitely"),
            start: Button::new("start simulation", "is-success", "fa-play", || Msg::Start),
            update: Button::new("update parameters", "is-link", "fa-wrench", || Msg::Update),
            stop: Button::new(
                "stop simulation",
                "is-danger is-outline",
                "fa-times",
                || Msg::Stop,
            ),
            started: false
        }
    }

    pub fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) -> Action {
        match msg {
            Msg::Start => match self.extract() {
                Some((a, b)) => {
                    self.started = true;
                    Action::Start(a, b)
                }
                None => Action::None,
            },
            Msg::Update => match self.extract_config() {
                Some(a) => Action::Update(a),
                None => Action::None,
            },
            Msg::Stop => {
                self.started = false;
                Action::Stop
            }
            Msg::Ticks(msg) => {
                self.ticks.update(msg, &mut orders.proxy(Msg::Ticks));
                Action::None
            }
        }
    }

    fn extract_initial(&self) -> Option<Initial> {
        let ticks = self.ticks.value(true);

        Some(Initial { ticks })
    }

    fn extract_config(&self) -> Option<Config> {
        Some(Config {
            param1: "".to_string(),
        })
    }

    fn extract(&self) -> Option<(Initial, Config)> {
        Some((self.extract_initial()?, self.extract_config()?))
    }

    pub fn view(&self) -> Node<Msg> {
        div![
            C!["box"],
            self.ticks.view(self.started).map_msg(Msg::Ticks),

            div![C!["buttons"],
                self.start.view(self.started),
                self.update.view(!self.started),
                self.stop.view(!self.started)
            ]
        ]
    }
}

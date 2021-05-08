mod api;
mod components;
mod fields;
mod forms;
mod graphs;

use seed::{prelude::*, *};

use crate::forms::{Action, ConfigForm};
use crate::graphs::first;
use std::future::Future;

#[derive(Debug)]
pub enum Msg {
    Config(crate::forms::config::Msg),
    Received(WebSocketMessage),
}

struct Model {
    config: ConfigForm,

    local: Vec<f32>,
}

// very dirty trick to make the order code look good
pub fn order<F: Future<Output = ()> + 'static>(f: impl FnOnce() -> F + 'static, orders: &mut impl Orders<Msg>)
{
    orders.skip().perform_cmd(async move {
        f().await;
        let t: Option<Msg> = None;
        t
    });
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    Model {
        config: ConfigForm::new(),
        local: vec![],
    }
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    log!(msg);
    match msg {
        Msg::Config(msg) => match model.config.update(msg, &mut orders.proxy(Msg::Config)) {
            Action::Start(initial, config) => order(move || api::start((initial, config)), orders),
            Action::Update(config) => order(move || api::update(config), orders),
            Action::Stop => order(move || api::stop(), orders),
            Action::None => (),
        },

        Msg::Received(message) => {
            let values: Vec<f32> = message.json().unwrap();
            model.local.extend(values);
            first::draw("canvas", &model.local).expect("could not draw");
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![
        C!["container"],
        model.config.view().map_msg(Msg::Config),
        canvas![attrs! {At::Id => "canvas", At::Width => "600", At::Height => "400"}]
    ]
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}

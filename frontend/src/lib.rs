mod api;
mod components;
mod fields;
mod forms;
mod graphs;

use seed::{prelude::*, *};

use crate::forms::{Action, ConfigForm};
use crate::graphs::simple;
use std::future::Future;
use crate::api::{State, query};

#[derive(Debug)]
pub enum Msg {
    Config(crate::forms::config::Msg),
    Query(State),
}

struct Model {
    config: ConfigForm,
    history: Vec<State>,
    handle: Option<CmdHandle>
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

async fn loop_query() {
    loop {
        cmds::timeout(5000, ||()).await;
        query().await.map(Msg::Query);
    }
}

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model {
        config: ConfigForm::new(),
        history: vec![],
        handle: None
    }
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Config(msg) => match model.config.update(msg, &mut orders.proxy(Msg::Config)) {
            Action::Start(initial, config) => {
                model.handle = Some(orders.perform_cmd_with_handle(loop_query()));
                order(move || api::start((initial, config)), orders)
            },
            Action::Update(config) => order(move || api::update(config), orders),
            Action::Stop => {
                model.handle = None;
                order(move || api::stop(), orders)
            },
            Action::None => (),
        },

        Msg::Query(state) => {
            log!(state);
            model.history.push(state);
            simple::draw("canvas", &model.history).expect("could not draw");
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![
        C!["columns"],
        div![C!["column is-8"], canvas![attrs! {At::Id => "canvas", At::Width => "600", At::Height => "400"}]],
        div![C!["column"], model.config.view().map_msg(Msg::Config)],
    ]
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}

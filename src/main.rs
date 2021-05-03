use futures::{FutureExt, StreamExt};
use log::{info, warn};
use warp::{Filter, Reply};

use simulation::{Config, init, step};
use warp::ws::Message;
use std::sync::mpsc;

#[derive(serde::Deserialize, Clone, Debug)]
pub enum Command {
    Pause,
    Update(Config),
}

#[derive(serde::Deserialize, Clone, Debug)]
pub enum Msg {
    Start{ticks: Option<u64>, config: Config},

    Command(Command),

    Notify(Vec<f32>)
}


fn simulate(ticks: Option<u64>, mut config: Config, receiver: mpsc::Receiver<Command>) {
    let mut state = init();
    let ticks = ticks.unwrap_or(u64::MAX); // i really don't care

    for _ in 0..ticks {
        match receiver.try_recv() {
            Ok(Command::Pause) => unimplemented!(),
            Ok(Command::Update(new)) => config = new,
            _ => (),
        }

        step(&mut state, &config)
    }
}

async fn upgrade(conn: warp::ws::WebSocket) {
    let (msg_sender, mut msg_receiver) = conn.split();
    let mut command_sender = None;

    // this whole thing is kinda messy
    while let Some(Ok(message)) = msg_receiver.next().await {
        match message.to_str() {
            Ok(msg) => match (serde_json::from_str(msg).unwrap(), &mut command_sender) {
                (Msg::Start{..}, Some(_)) => warn!("simulation already started, ignoring"),
                (Msg::Start{ticks, config}, old) => {
                    let (sender, receiver) = mpsc::channel();
                    *old = Some(sender);
                    tokio::task::spawn_blocking(move || simulate(ticks, config, receiver));
                }
                (Msg::Command(cmd), Some(sender)) => sender.send(cmd).unwrap(),
                (Msg::Command(_), None) => warn!("received command without starting, ignoring"),
                (Msg::Notify(_), _) => warn!("server should not receive a notify from the client, ignoring")
            },
            Err(_) => warn!("websocket encountered an error, ignoring")
        }
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let types = |reply: warp::filters::fs::File|
        {
            if reply.path().ends_with("wasm.js")
            {
                let reply = warp::reply::with_header(reply, "Cache-Control", "no-store");
                let reply = warp::reply::with_header(reply, "Content-Type", "text/javascript");
                reply.into_response()
            } else {
                reply.into_response()
            }
        };

    let ws = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(upgrade));

    let files = warp::fs::dir("static").map(types);
    let routes = ws.or(files);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await
}

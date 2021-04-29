use futures::{FutureExt, StreamExt};
use log::info;
use warp::{Filter, Reply};

use simulation::{Config, init, step};

pub enum Msg {
    Start(Config),
    Change(Config),
    Stop,
}

async fn simulate(config: Config) {
    let mut current = 0u32;
    let mut state = init();

    loop {
        match config.ticks {
            Some(ticks) if current > ticks => break,
            _ => ()
        }

        step(&mut state, &config)
    }
}

async fn upgrade(conn: warp::ws::WebSocket) {
    let (sender, mut receiver) = conn.split();

    while let Some(Ok(message)) = receiver.next().await {
        println!("{:?}", message);
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

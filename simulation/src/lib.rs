use std::time::Duration;

pub struct Config {
    pub ticks: Option<u32>,
    pub param1: String,
    pub param2: String,
}

pub struct State {
    population: Vec<f32>,
}

pub fn init() -> State {
    println!("initialized simulation");
    State { population: vec![] }
}

pub fn step(state: &mut State, config: &Config) {
    println!("step simulation");
    std::thread::sleep(Duration::from_secs(1))
}
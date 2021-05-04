use serde::{Serialize, Deserialize};

// copy pasta from backend
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub param1: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Initial {
    pub ticks: Option<u64>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Command {
    Pause,
    Update(Config),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Msg {
    Start(Initial),

    Command(Command),

    Notify(Vec<f32>),
}

#[derive(Clone, Debug, Deserialize)]
pub struct Suggestion
{
    pub name: String,
    pub value: i64
}

pub type Suggestions = Vec<Suggestion>;
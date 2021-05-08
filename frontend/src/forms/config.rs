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
    // initial values
    t_max: InputField<u64>,
    population: InputField<u64>,
    environment: InputField<u64>,

    // configurable values
    mutation_mu: InputField<f64>,
    mutation_sigma: InputField<f64>,
    mutation_step: InputField<f64>,
    rec: InputField<f64>,
    r_max: InputField<f64>,
    selection_sigma: InputField<f64>,
    gamma: InputField<f64>,
    diploid: InputField<bool>,
    m: InputField<f64>,

    // buttons
    start: Button<Msg>,
    update: Button<Msg>,
    stop: Button<Msg>,

    // state
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
            t_max: InputField::new("Ticks", false)
                .with_placeholder("leave empty to run indefinitely")
                .with_initial(Some(100_000)),
            population: InputField::new("Population size").with_initial(6_000),
            environment: InputField::new("Environment").with_initial(2),

            mutation_mu: InputField::new("Mutation Mu").with_initial(0.01),
            mutation_sigma: InputField::new("Mutation Sigma").with_initial(0.01),
            mutation_step: InputField::new("Mutation Step").with_initial(0.01),
            rec : InputField::new("Recombinational probability").with_initial(0.01),
            r_max: InputField::new("Max amount of offspring").with_initial(1000),
            selection_sigma: InputField::new("Selection Sigma"),
            gamma: InputField::new("Gamma").with_placeholder("generation overlap"),
            diploid: InputField::new("Diploid").with_initial(true),
            m: InputField::new("Dispersal parameter"),
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
            Msg::Start => return match self.extract() {
                Some((a, b)) => {
                    self.started = true;
                    Action::Start(a, b)
                }
                None => Action::None,
            },
            Msg::Update => return match self.extract_config() {
                Some(a) => Action::Update(a),
                None => Action::None,
            },
            Msg::Stop => {
                self.started = false;
                return Action::Stop
            },
            _ => None,
        }

        match msg {
            Msg::Ticks(msg) => self.t_max.update(&mut orders.proxy(Msg::Ticks)),
            _ => unreachable!("all other cases must be handled in previous match")
        }
        Action::None
    }

    fn extract_initial(&self) -> Option<InitConfig> {
        let t_max = self.t_max.value(true);
        let population = self.population.value(true);
        let environment = self.environment.value(true);

        Some(InitConfig { t_max, population, environment })
    }

    fn extract_config(&self) -> Option<Config> {
        let mutation_mu = self.mutation_mu.value(true);
        let mutation_sigma = self.mutation_sigma.value(true);
        let mutation_step = self.mutation_step.value(true);
        let rec = self.rec.value(true);
        let r_max = self.r_max.value(true);
        let selection_sigma = self.selection_sigma.value(true);
        let gamma = self.gamma.value(true);
        let diploid = self.diploid.value(true);
        let m = self.m.value(true);

        Some(Config {
            mutation_mu: mutation_mu?,
            mutation_sigma: mutation_sigma?,
            mutation_step: mutation_step?,
            rec: rec?,
            r_max: r_max?,
            selection_sigma: selection_sigma?,
            gamma: gamma?,
            diploid: diploid?,
            m: m?
        })
    }

    fn extract(&self) -> Option<(InitConfig, Config)> {
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

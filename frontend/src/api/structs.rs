use serde::{Deserialize, Serialize};

// copy pasta from backend
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitConfig {
    // max ticks, unlimited if None (=100000)
    pub t_max: Option<u64>,
    // initial population: columns are individuals, rows are loci
    // population size N = population.ncols() cannot change (=6000)
    // number of loci k = population.nrow() cannot change (=4)
    pub population: DMatrix<f64>,
    // initial environment
    // patch number n = environment.ncols() cannot change (must devide N)
    pub environment: DVector<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    // trait mutation probability (=0.01)
    pub mutation_mu: DVector<f64>,
    // expected mutational effect size (=0.01)
    pub mutation_sigma: f64,
    // bin size for mutational effects (=0.01)
    pub mutation_step: f64,
    // recombinational probality (=0.01)
    pub rec: f64,
    // maximum amount of offspring (=1000)
    pub r_max: f64,
    // selection strength (standard deviation)
    pub selection_sigma: f64,
    // generation overlap
    pub gamma: f64,
    // diploid or haploid
    pub diploid: bool,
    // dispersal parameter
    pub m: f64,
    // function to determine environment and selective optima
    pub environment_function: fn(&mut DVector<f64>, u64),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Command {
    Pause,
    Update(Config),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Msg {
    Start(InitConfig),

    Command(Command),

    Notify(Vec<f32>),
}

#[derive(Clone, Debug, Deserialize)]
pub struct Suggestion
{
    pub name: String,
    pub value: i64,
}

pub type Suggestions = Vec<Suggestion>;
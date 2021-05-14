use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InitialPopulation {
    // I: per individual, P: per patch, None: per population
    UniformI,
    UniformP,
    Uniform,
    ConstantI,
    ConstantP,
    Constant,
    NormalI,
    NormalP,
    Normal,
    AlternatingHalf,
    AlternatingThird,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitConfig {
    // max ticks, unlimited if None (=100000)
    pub t_max: Option<u64>,

    pub kind:        InitialPopulation,
    pub patches:     usize,
    pub individuals: usize,
    pub loci:        usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Environment {
    Random,
    AlternatingHalf,
    AlternatingThird,
    Sine,
    RandomWalk,
    Constant,
    ConstantWithJumps,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    // trait mutation probability (=0.01)
    pub mutation_mu:     f64,
    // expected mutational effect size (=0.01)
    pub mutation_sigma:  f64,
    // bin size for mutational effects (=0.01)
    pub mutation_step:   f64,
    // recombinational probality (=0.01)
    pub rec:             f64,
    // selection strength (standard deviation)
    pub selection_sigma: f64,
    // generation overlap
    pub gamma:           f64,
    // diploid or haploid
    pub diploid:         bool,
    // dispersal parameter
    pub m:               f64,
    // environment update function
    pub environment:     Environment,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Suggestion
{
    pub name: String,
    pub value: i64,
}

pub type Suggestions = Vec<Suggestion>;
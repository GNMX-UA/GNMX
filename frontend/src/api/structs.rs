use serde::{Deserialize, Serialize};
use tinyvec::TinyVec;

// copy pasta from backend
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Individual {
    loci: TinyVec<[f64; 10]>,
}

impl Individual {
    pub fn phenotype(&self) -> f64 { self.loci.iter().sum() }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Patch {
    pub individuals: Vec<Individual>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TempEnum {
    Default
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitConfig {
    // max ticks, unlimited if None (=100000)
    pub t_max: Option<u64>,

    pub kind: TempEnum
}

// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct InitConfig {
//     // max ticks, unlimited if None (=100000)
//     pub t_max: Option<u64>,
//
//     pub population_size: u64,
//     pub population_type: i64,
//
//     pub patch_amount: u64,
//     pub patch_type: i64,
// }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    // trait mutation probability (=0.01)
    pub mutation_mu: f64,
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub tick:    u64,
    pub patches: Vec<(Patch, f64)>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Suggestion
{
    pub name: String,
    pub value: i64,
}

pub type Suggestions = Vec<Suggestion>;
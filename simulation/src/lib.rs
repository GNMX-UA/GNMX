use std::f64::consts::PI;

use itertools::izip;
use rand::{
	distributions::{Distribution, WeightedIndex},
	thread_rng, Rng,
};

// possible extensions:
// no juvenile/adult carrying capacity (= 1/n)
// mutation_mu/sigma per trait
// measuring intervals, histint
// theta vector
// phenotype is not sum -> use inner product
// TODO don't scale by sqrt(2pi)

#[derive(Clone)]
pub struct Individual {
	genotype: Vec<f64>,
}

impl Individual {
	fn phenotype(&self) -> f64 { self.genotype.iter().sum() }
}

pub struct Patch {
	environment: f64,
	individuals: Vec<Individual>,
}

impl Patch {}

pub struct InitConfig {
	// max ticks, unlimited if None (=100000)
	pub t_max:   Option<u64>,
	// population size cannot change (=6000)
	// number of loci cannot change (=4)
	// patch number cannot change
	pub patches: Vec<Patch>,
}

#[derive(Clone)]
pub struct Config {
	// trait mutation probability (=0.01)
	pub mutation_mu:          f64,
	// expected mutational effect size (=0.01)
	pub mutation_sigma:       f64,
	// bin size for mutational effects (=0.01)
	pub mutation_step:        f64,
	// recombinational probality (=0.01)
	pub rec:                  f64,
	// maximum amount of offspring (=1000)
	pub r_max:                f64,
	// selection strength (standard deviation)
	pub selection_sigma:      f64,
	// generation overlap
	pub gamma:                f64,
	// diploid or haploid
	pub diploid:              bool,
	// dispersal parameter
	pub m:                    f64,
	// function to determine environment and selective optima
	pub environment_function: fn(&mut Vec<Patch>, u64),
}

pub struct State {
	tick:    u64,
	patches: Vec<Patch>,
}

impl State {
	pub fn reproduction(&self, r_max: f64, selection_sigma: f64) -> Vec<Vec<f64>> {
		let mut reproductive_success = Vec::with_capacity(self.patches.len());
		for patch in &self.patches {
			let mut patch_success = Vec::with_capacity(patch.individuals.len());
			for individual in &patch.individuals {
				let offspring = r_max / (selection_sigma * (2.0 * PI).sqrt())
					* (-(&patch.environment - individual.phenotype())
						/ (2.0 * selection_sigma.powi(2)))
					.exp();
				patch_success.push(offspring);
			}
			reproductive_success.push(patch_success);
		}
		reproductive_success
	}

	pub fn adult_death(&self, gamma: f64) -> Vec<Vec<&Individual>> {
		let mut rng = thread_rng();
		let mut death = Vec::with_capacity(self.patches.len());
		for patch in &self.patches {
			let mut patch_death = Vec::with_capacity(patch.individuals.len()); // upper bound
			for individual in &patch.individuals {
				if !rng.gen_bool(gamma) {
					patch_death.push(individual);
				}
			}
			death.push(patch_death);
		}
		death
	}

	pub fn density_regulation(
		&self,
		reproductive_success: &Vec<Vec<f64>>,
		death: &Vec<Vec<&Individual>>,
	) -> Vec<Vec<Individual>> {
		let mut new_generation = Vec::with_capacity(self.patches.len());
		for (patch, patch_success, patch_death) in izip!(&self.patches, reproductive_success, death)
		{
			new_generation.push(
				WeightedIndex::new(patch_success)
					.unwrap()
					.sample_iter(thread_rng())
					.take(patch_death.len())
					.map(|index| patch.individuals[index].clone())
					.collect(),
			);
		}
		new_generation
	}

	pub fn dispersal(&mut self) {}

	pub fn mutation(&mut self) {}
}

pub fn init(init_config: InitConfig) -> Result<State, &'static str> {
	Ok(State {
		tick:    0,
		patches: init_config.patches,
	})
}

pub fn step(state: &mut State, config: &Config) {
	(config.environment_function)(&mut state.patches, state.tick);
	let reproductive_success = state.reproduction(config.r_max, config.selection_sigma);
	let death = state.adult_death(config.gamma);
	let new_generation = state.density_regulation(&reproductive_success, &death);
	if config.diploid {
		todo!()
	} else {
	}
	state.dispersal();
}

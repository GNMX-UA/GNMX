use core::ptr;

use itertools::izip;
use rand::{prelude::SliceRandom, thread_rng, Rng};
use rand_distr::{Bernoulli, Binomial, Distribution, Normal, WeightedIndex};
use serde::{Deserialize, Serialize};

// TODO juvenile/adult
// TODO dispersal matrix
// TODO Vec<Individu>
// TODO init

// possible extensions:
// no juvenile/adult carrying capacity (= 1/n)
// dispersal chance not equal (no pool)
// mutation_mu/sigma per trait
// measuring intervals, histint
// theta vector
// phenotype is not sum -> use inner product

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Individual {
	loci: Vec<f64>,
}

impl Individual {
	fn phenotype(&self) -> f64 { self.loci.iter().sum() }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Patch {
	pub environment: f64,
	pub individuals: Vec<Individual>,
}

impl Patch {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TempEnum {
	Default,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitConfig {
	// max ticks, unlimited if None (=100000)
	pub t_max: Option<u64>,

	pub kind: TempEnum,
}

// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct InitConfig {
// 	// max ticks, unlimited if None (=100000)
// 	pub t_max:   Option<u64>,
// 	// population size can change (=6000)
// 	// number of loci (half if diploid) cannot change (=4)
// 	// patch number cannot change
// 	pub patches: Vec<Patch>,
// }

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
	// maximum amount of offspring (=1000)
	pub r_max:           f64,
	// selection strength (standard deviation)
	pub selection_sigma: f64,
	// generation overlap
	pub gamma:           f64,
	// diploid or haploid
	pub diploid:         bool,
	// dispersal parameter
	pub m:               f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
	pub tick:    u64,
	pub patches: Vec<Patch>,
}

impl State {
	pub fn reproduction(&self, r_max: f64, selection_sigma: f64) -> Vec<Vec<f64>> {
		let mut reproductive_success = Vec::with_capacity(self.patches.len());
		for patch in &self.patches {
			let mut patch_success = Vec::with_capacity(patch.individuals.len());
			for individual in &patch.individuals {
				// r(y, theta) = r_max*e^(-(theta - y)^2/(2*sigma^2)

				// use std::f64::consts::PI;
				// let offspring = ((r_max / (selection_sigma * (2.0*PI).sqrt())).ln()
				// 	- ((&patch.environment - individual.phenotype()).powi(2)
				// 		/ (2.0 * selection_sigma.powi(2)))).exp();
				let offspring = (r_max.ln()
					- ((&patch.environment - individual.phenotype()).powi(2)
						/ (2.0 * selection_sigma.powi(2))))
				.exp();
				patch_success.push(offspring);
			}
			reproductive_success.push(patch_success);
		}
		reproductive_success
	}

	pub fn adult_death(&mut self, gamma: f64) -> Vec<usize> {
		let mut rng = thread_rng();
		let mut death = Vec::with_capacity(self.patches.len());
		for patch in &mut self.patches {
			patch.individuals.shuffle(&mut rng);
			let patch_alive = Binomial::new(patch.individuals.len() as u64, gamma)
				.unwrap()
				.sample(&mut rng) as usize;
			death.push(patch.individuals.len() - patch_alive);
			patch
				.individuals
				.resize(patch_alive, Individual { loci: vec![] });
		}
		death
	}

	pub fn density_regulation(
		&self,
		reproductive_success: &Vec<Vec<f64>>,
		death: &Vec<usize>,
	) -> Vec<Vec<Individual>> {
		let mut new_generation = Vec::with_capacity(self.patches.len());
		for (patch, patch_success, patch_death) in izip!(&self.patches, reproductive_success, death)
		{
			new_generation.push(
				WeightedIndex::new(patch_success)
					.unwrap()
					.sample_iter(thread_rng())
					.take(2 * patch_death)
					.map(|index| patch.individuals[index].clone())
					.collect(),
			);
		}
		new_generation
	}

	pub fn recombination(
		&self,
		mut new_generation: Vec<Vec<Individual>>,
		rec: f64,
	) -> Vec<Vec<Individual>> {
		let k = self.patches[0].individuals[0].loci.len() / 2; //TODO proper
		// rec = 1-(1-locus_rec)^(k-1)
		let locus_rec = 1.0 - (1.0 / (k as f64 - 1.0) * (1.0 - rec).ln()).exp();
		let mut rng = thread_rng();
		let distr = Bernoulli::new(locus_rec).unwrap();
		let swapped = Bernoulli::new(0.5).unwrap();
		for patch in &mut new_generation {
			for individual in &mut *patch {
				let (loci1, loci2) = individual.loci.split_at_mut(k);
				let mut swapped = swapped.sample(&mut rng);
				for (locus1, locus2) in loci1.iter_mut().zip(&*loci2) {
					if distr.sample(&mut rng) {
						swapped = !swapped;
					}
					if swapped {
						*locus1 = *locus2;
					}
				}
			}
			for i in 0 .. patch.len() / 2 {
				unsafe {
					let individual = &mut *(patch.get_unchecked_mut(i) as *mut Individual);
					individual.loci[.. k].copy_from_slice(&patch[2 * i].loci[.. k]);
					individual.loci[k ..].copy_from_slice(&patch[(2 * i) + 1].loci[.. k]);
				}
			}
			patch.resize(patch.len() / 2, Individual { loci: vec![] })
		}
		new_generation
	}

	pub fn haploid_generation(mut new_generation: Vec<Vec<Individual>>) -> Vec<Vec<Individual>> {
		for patch in &mut new_generation {
			patch.resize(patch.len() / 2, Individual { loci: vec![] });
		}
		new_generation
	}

	pub fn dispersal(mut new_generation: Vec<Vec<Individual>>, m: f64) -> Vec<Vec<Individual>> {
		let mut rng = thread_rng();
		let distr = Bernoulli::new(m).unwrap();
		let mut pool: Vec<_> = new_generation
			.iter_mut()
			.flatten()
			.filter(|_| distr.sample(&mut rng))
			.collect();
		for i in (1 .. pool.len()).rev() {
			let pa = ptr::addr_of_mut!(*pool[i]);
			let pb = ptr::addr_of_mut!(*pool[gen_index(&mut thread_rng(), i + 1)]);
			unsafe {
				ptr::swap(pa, pb);
			}
		}
		new_generation
	}

	pub fn mutation(
		mut new_generation: Vec<Vec<Individual>>,
		mutation_mu: f64,
		mutation_sigma: f64,
		mutation_step: f64,
	) -> Vec<Vec<Individual>> {
		let mut rng = thread_rng();
		let distr = Bernoulli::new(mutation_mu).unwrap();
		// fixed
		// let up_down = Bernoulli::new(0.5).unwrap();
		// gausian
		let up_down = Normal::new(0.0, mutation_sigma).unwrap();
		for patch in &mut new_generation {
			for individual in patch {
				for locus in &mut individual.loci {
					// fixed
					// *locus += mutation_step
					// 	* 2.0 * (up_down.sample(&mut rng) as i32 as f64 - 0.5)
					// 	* distr.sample(&mut rng) as i32 as f64;
					// gaussian
					*locus += distr.sample(&mut rng) as i32 as f64
						* mutation_step * (up_down.sample(&mut rng) * mutation_sigma
						/ mutation_step)
						.round()
				}
			}
		}
		new_generation
	}

	fn update(&mut self, new_generation: Vec<Vec<Individual>>) {
		for (new_patch, patch) in new_generation.into_iter().zip(&mut self.patches) {
			patch.individuals.extend(new_patch);
		}
	}
}

// TODO: remove result probably
pub fn init(init_config: InitConfig) -> Result<State, &'static str> {
	Ok(State {
		tick:    0,
		patches: vec![Patch {
			environment: 0.5,
			individuals: vec![Individual {
				loci: vec![0.5, 0.7],
			}],
		}],
	})
}

pub fn step(state: &mut State, config: &Config) {
	// (config.environment_function)(&mut state.patches, state.tick);
	let reproductive_success = state.reproduction(config.r_max, config.selection_sigma);
	let death = state.adult_death(config.gamma);
	let mut new_generation = state.density_regulation(&reproductive_success, &death);
	if config.diploid {
		new_generation = state.recombination(new_generation, config.rec);
	} else {
		new_generation = State::haploid_generation(new_generation);
	}
	new_generation = State::dispersal(new_generation, config.m);
	new_generation = State::mutation(
		new_generation,
		config.mutation_mu,
		config.mutation_sigma,
		config.mutation_step,
	);
	state.update(new_generation);
}

#[inline]
fn gen_index<R: Rng + ?Sized>(rng: &mut R, ubound: usize) -> usize {
	if ubound <= (core::u32::MAX as usize) {
		rng.gen_range(0 .. ubound as u32) as usize
	} else {
		rng.gen_range(0 .. ubound)
	}
}

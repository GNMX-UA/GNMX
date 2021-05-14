use core::ptr;
use std::{
	f64::consts::PI,
	ops::{Deref, DerefMut},
};

use itertools::izip;
use rand::{prelude::SliceRandom, thread_rng, Rng};
use rand_distr::{Bernoulli, Binomial, Distribution, Normal, Uniform, WeightedAliasIndex};
use serde::{Deserialize, Serialize};
use tinyvec::{tiny_vec, TinyVec};

mod test;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Individual {
	loci: TinyVec<[f64; 10]>,
}

impl Individual {
	pub fn phenotype(&self) -> f64 { self.loci.iter().sum() }
}

impl Deref for Individual {
	type Target = TinyVec<[f64; 10]>;

	fn deref(&self) -> &Self::Target { &self.loci }
}

impl DerefMut for Individual {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.loci }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Patch {
	pub individuals: Vec<Individual>,
}

impl Patch {
	pub fn new(individuals: Vec<Individual>) -> Patch { Self { individuals } }

	pub fn extend(&mut self, other: Patch) { self.individuals.extend(other.individuals); }

	pub fn random(size: usize, patch_size: usize, loci: usize) -> Vec<Patch> {
		let mut rng = thread_rng();
		let distr = Uniform::new(-1.0 / loci as f64, 1.0 / loci as f64);
		(0 .. size)
			.map(|_| Patch {
				individuals: (0 .. patch_size)
					.map(|_| Individual {
						loci: distr.sample_iter(&mut rng).take(2 * loci).collect(),
					})
					.collect(),
			})
			.collect()
	}

	pub fn random_env(size: usize) -> Vec<f64> {
		let mut rng = thread_rng();
		let distr = Uniform::new(-1.0, 1.0);
		distr.sample_iter(&mut rng).take(size).collect()
	}
}

impl Deref for Patch {
	type Target = Vec<Individual>;

	fn deref(&self) -> &Self::Target { &self.individuals }
}

impl DerefMut for Patch {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.individuals }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TempEnum {
	Default,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitConfig {
	// max ticks, unlimited if None (=100000)
	pub t_max: Option<u64>,

	pub kind:        TempEnum,
	pub patches:     usize,
	pub individuals: usize,
	pub loci:        usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Environment {
	Default,
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
	/* environment update function
	 * pub environment:     Environment, */
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
	pub tick:    u64,
	pub patches: Vec<(Patch, f64)>,
}

impl State {
	// update the environment
	pub fn environment(&mut self, environment: &Environment, tick: u64) {
		match environment {
			Environment::Default => {},
		}
	}

	/// calculate amount of offspring per individual per patch
	pub fn reproduction(&self, r_max: f64, selection_sigma: f64) -> Vec<Vec<f64>> {
		let mut reproductive_success = Vec::with_capacity(self.patches.len());
		for (patch, env) in &self.patches {
			let mut patch_success = Vec::with_capacity(patch.len());
			for individual in &**patch {
				// r(y, theta) = r_max*e^(-(theta - y)^2/(2*sigma^2)

				let offspring = ((r_max / (selection_sigma * (2.0 * PI).sqrt())).ln()
					- ((env - individual.phenotype()).powi(2) / (2.0 * selection_sigma.powi(2))))
				.exp();

				// let offspring = (r_max.ln()
				// 	- ((env - individual.phenotype()).powi(2) / (2.0 * selection_sigma.powi(2))))
				// .exp();
				patch_success.push(offspring);
			}
			reproductive_success.push(patch_success);
		}
		reproductive_success
	}

	/// calculate amount of deaths per patch
	pub fn adult_death(&mut self, gamma: f64) -> Vec<usize> {
		let mut rng = thread_rng();
		let mut death = Vec::with_capacity(self.patches.len());
		for (patch, _) in &mut self.patches {
			let patch_alive = Binomial::new(patch.len() as u64, gamma)
				.unwrap()
				.sample(&mut rng) as usize;
			death.push(patch.len() - patch_alive);
		}
		death
	}

	/// create new generation by cloning as many individuals in a patch as there are deaths * 2
	pub fn density_regulation(
		&self,
		reproductive_success: Vec<Vec<f64>>,
		death: &Vec<usize>,
	) -> Vec<Patch> {
		let mut new_generation = Vec::with_capacity(self.patches.len());
		for ((patch, _), patch_success, patch_death) in
			izip!(&self.patches, reproductive_success, death)
		{
			let distr = if patch_success.iter().sum::<f64>() > 0.0 {
				WeightedAliasIndex::new(patch_success).unwrap()
			} else {
				WeightedAliasIndex::new(vec![1.0; patch_success.len()]).unwrap()
			};
			new_generation.push(Patch::new(
				distr
					.sample_iter(thread_rng())
					.take(2 * patch_death)
					.map(|index| patch[index].clone())
					.collect(),
			));
		}
		new_generation
	}

	/// produce gametes with recombination and then join every two gametes together for every patch
	/// results in new generation with as many individuals as deaths in the patch
	pub fn recombination(mut new_generation: Vec<Patch>, rec: f64) -> Vec<Patch> {
		let k = new_generation[0][0].len() / 2;
		let locus_rec = 1.0 - (1.0 / ((k - 1) as f64) * (1.0 - rec).ln()).exp();
		let mut rng = thread_rng();
		let distr = Bernoulli::new(locus_rec).unwrap();
		let swapped = Bernoulli::new(0.5).unwrap();
		for patch in &mut new_generation {
			for individual in &mut **patch {
				let (loci1, loci2) = individual.split_at_mut(k);
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
			let len = patch.len() / 2;
			for i in 0 .. len {
				unsafe {
					let individual = &mut *(patch.get_unchecked_mut(i) as *mut Individual);
					individual[.. k].copy_from_slice(&patch[2 * i][.. k]);
					individual[k ..].copy_from_slice(&patch[(2 * i) + 1][.. k]);
				}
			}
			patch.resize(
				len,
				Individual {
					loci: Default::default(),
				},
			)
		}
		new_generation
	}

	/// half the new generation in every patch
	pub fn haploid_generation(mut new_generation: Vec<Patch>) -> Vec<Patch> {
		for patch in &mut new_generation {
			let len = patch.len() / 2;
			patch.resize(
				len,
				Individual {
					loci: Default::default(),
				},
			);
		}
		new_generation
	}

	/// determine for every individual in the new generation if it will disperse
	/// then shuffle all the dispersing individuals around
	pub fn dispersal(mut new_generation: Vec<Patch>, m: f64) -> Vec<Patch> {
		let mut rng = thread_rng();
		let distr = Bernoulli::new(m).unwrap();
		let mut pool: Vec<_> = new_generation
			.iter_mut()
			.map(|patch| &mut **patch)
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

	/// mutate the value of every locus in every individual in every patch of the new generation
	pub fn mutation(
		mut new_generation: Vec<Patch>,
		mutation_mu: f64,
		mutation_sigma: f64,
		mutation_step: f64,
	) -> Vec<Patch> {
		let mut rng = thread_rng();

		let distr = Bernoulli::new(mutation_mu).unwrap();
		// fixed
		// let up_down = Bernoulli::new(0.5).unwrap();
		// normal
		let up_down = Normal::new(0.0, mutation_sigma).unwrap();

		for patch in &mut new_generation {
			for individual in &mut **patch {
				for locus in &mut **individual {
					if distr.sample(&mut rng) {
						// fixed
						// *locus +=
						// 	2.0 * mutation_step * (up_down.sample(&mut rng) as i32 as f64 - 0.5);
						// normal
						*locus += mutation_step * (up_down.sample(&mut rng) / mutation_step).round()
					}
				}
			}
		}
		new_generation
	}

	/// replace the old generation with the new one
	fn update(&mut self, new_generation: Vec<Patch>, death: Vec<usize>) {
		for ((patch, _), new, death) in izip!(&mut self.patches, new_generation, death) {
			patch.shuffle(&mut thread_rng());
			let len = patch.len() - death;
			patch.resize(len, Default::default());
			patch.extend(new);
		}
	}
}

pub fn init(init_config: InitConfig) -> Result<State, &'static str> {
	let patches = init_config.patches;
	let individuals = init_config.individuals;
	let loci = init_config.loci;

	let patch_size = individuals / patches;
	let p = Patch::random(patches, patch_size, loci).into_iter();
	let e = Patch::random_env(patches).into_iter();

	let state = State {
		tick:    0,
		patches: p.zip(e).collect(),
	};

	Ok(state)
}

pub fn step(state: &mut State, config: &Config) {
	// state.environment(&config.environment, state.tick);
	let reproductive_success = state.reproduction(config.r_max, config.selection_sigma);
	let death = state.adult_death(config.gamma);
	let mut new_generation = state.density_regulation(reproductive_success, &death);
	if config.diploid {
		new_generation = State::recombination(new_generation, config.rec);
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
	state.update(new_generation, death);
}

#[inline]
fn gen_index<R: Rng + ?Sized>(rng: &mut R, ubound: usize) -> usize {
	if ubound <= (core::u32::MAX as usize) {
		rng.gen_range(0 .. ubound as u32) as usize
	} else {
		rng.gen_range(0 .. ubound)
	}
}

use core::ptr;

use itertools::izip;
use patch::Patch;
use rand::{prelude::SliceRandom, thread_rng, Rng};
use rand_distr::{Bernoulli, Binomial, Distribution, Normal, WeightedAliasIndex};
use serde::{Deserialize, Serialize};
use tinyvec::TinyVec;

// TODO juvenile/adult
// TODO dispersal matrix
// TODO init

// possible extensions:
// no juvenile/adult carrying capacity (= 1/n)
// dispersal chance not equal (no pool)
// mutation_mu/sigma per trait
// measuring intervals, histint
// theta vector
// phenotype is not sum -> use inner product

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Individual {
	loci: TinyVec<[f64; 10]>,
}

impl Individual {
	pub fn phenotype(&self) -> f64 { self.loci.iter().sum() }
}

pub mod patch {
	use std::ops::{Deref, DerefMut};

	use rand_distr::Uniform;

	use super::*;

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
							loci: distr.sample_iter(&mut rng).take(loci).collect(),
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
	pub patches: Vec<(Patch, f64)>,
}

impl State {
	pub fn reproduction(&self, r_max: f64, selection_sigma: f64) -> Vec<Vec<f64>> {
		let mut reproductive_success = Vec::with_capacity(self.patches.len());
		for (patch, env) in &self.patches {
			let mut patch_success = Vec::with_capacity(patch.len());
			for individual in &**patch {
				// r(y, theta) = r_max*e^(-(theta - y)^2/(2*sigma^2)

				// use std::f64::consts::PI;
				// let offspring = ((r_max / (selection_sigma * (2.0*PI).sqrt())).ln()
				// 	- ((&patch.environment - individual.phenotype()).powi(2)
				// 		/ (2.0 * selection_sigma.powi(2)))).exp();

				let offspring = (r_max.ln()
					- ((env - individual.phenotype()).powi(2) / (2.0 * selection_sigma.powi(2))))
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
		for (patch, _) in &mut self.patches {
			patch.shuffle(&mut rng);
			let patch_alive = Binomial::new(patch.len() as u64, gamma)
				.unwrap()
				.sample(&mut rng) as usize;
			death.push(patch.len() - patch_alive);
			// TODO add back in
			// patch
			// 	.individuals
			// 	.resize(patch_alive, Individual { loci: vec![] });
		}
		death
	}

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
				// TODO vec empty
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

	pub fn recombination(&self, mut new_generation: Vec<Patch>, rec: f64) -> Vec<Patch> {
		let k = self.patches[0].0[0].loci.len() / 2; //TODO proper
		// rec = 1-(1-locus_rec)^(k-1)
		let locus_rec = 1.0 - (1.0 / (k as f64 - 1.0) * (1.0 - rec).ln()).exp();
		let mut rng = thread_rng();
		let distr = Bernoulli::new(locus_rec).unwrap();
		let swapped = Bernoulli::new(0.5).unwrap();
		for patch in &mut new_generation {
			for individual in &mut **patch {
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
			let len = patch.len() / 2;
			for i in 0 .. len {
				unsafe {
					let individual = &mut *(patch.get_unchecked_mut(i) as *mut Individual);
					individual.loci[.. k].copy_from_slice(&patch[2 * i].loci[.. k]);
					individual.loci[k ..].copy_from_slice(&patch[(2 * i) + 1].loci[.. k]);
				}
			}
			patch.resize(
				len / 2,
				Individual {
					loci: Default::default(),
				},
			)
		}
		new_generation
	}

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
				for locus in &mut individual.loci {
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

	fn update(&mut self, new_generation: Vec<Patch>) {
		for (new_patch, (patch, _)) in new_generation.into_iter().zip(&mut self.patches) {
			patch.extend(new_patch);
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

	// state.patches.push((Patch::new(vec![Individual { loci: tiny_vec!(0.5, 0.7) }]), 0.5));

	Ok(state)
}

pub fn step(state: &mut State, config: &Config) {
	// (config.environment_function)(&mut state.patches, state.tick);
	let reproductive_success = state.reproduction(config.r_max, config.selection_sigma);
	let death = state.adult_death(config.gamma);
	let mut new_generation = state.density_regulation(reproductive_success, &death);
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
	// TODO remove
	for ((patch, _), death) in state.patches.iter_mut().zip(death) {
		let len = patch.len() - death;
		patch.resize(len, Default::default());
	}
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

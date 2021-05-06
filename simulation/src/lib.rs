use std::f64::consts::PI;

use nalgebra::{DMatrix, DVector};
use rand::{thread_rng, Rng};

// possible extensions:
// no juvenile/adult carrying capacity (= 1/n)
// mutation_mu/sigma per trait
// measuring intervals, histint
// theta vector
// phenotype is not sum -> use inner product
// TODO don't scale by sqrt(2pi)

pub struct InitConfig {
	// initial population: columns are individuals, rows are loci
	// population size N = population.ncols() cannot change (=6000)
	// number of loci k = population.nrow() cannot change (=4)
	pub population:  DMatrix<f64>,
	// initial environment
	// patch number n = environment.ncols() cannot change (must devide N)
	pub environment: DVector<f64>,
}

#[derive(Clone)]
pub struct Config {
	// max ticks, unlimited if None (=100000)
	pub t_max:                Option<u64>,
	// trait mutation probability (=0.01)
	pub mutation_mu:          DVector<f64>,
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
	pub environment_function: fn(&mut DVector<f64>, u64),
}

#[derive(Debug)]
pub struct State {
	tick:        u64,
	population:  DMatrix<f64>,
	environment: DVector<f64>,
}

impl State {
	pub fn adult_death(&mut self, gamma: f64) -> DVector<usize> {
		let mut rng = thread_rng();
		DVector::from_iterator(
			self.population.len(),
			(0 .. self.population.len()).filter(|_| rng.gen_bool(gamma)),
		)
	}

	pub fn reproduction(&mut self, r_max: f64, selection_sigma: f64) -> DVector<f64> {
		let phenotype = self.population.row_sum_tr();
		let scale = r_max / (selection_sigma * (2.0 * PI).sqrt());
		let chance: DVector<f64> =
			(-(&self.environment - phenotype) / (2.0 * selection_sigma.powi(2))).map(|i| i.exp());
		scale * chance
	}

	pub fn density_regulation(&mut self, offspring: &DVector<f64>) {
		let patch_size = self.population.len() / self.environment.len();
	}

	pub fn dispersal(&mut self) {}

	pub fn mutation(&mut self) {}
}

pub fn init(init_config: InitConfig) -> Result<State, &'static str> {
	if init_config.population.ncols() % init_config.environment.ncols() != 0 {
		return Err("Population size must be divisible by number of patches");
	}

	Ok(State {
		tick:        0,
		population:  init_config.population,
		environment: init_config.environment,
	})
}

pub fn step(state: &mut State, config: &Config) {
	(config.environment_function)(&mut state.environment, state.tick);
	let death = state.adult_death(config.gamma);
	let offspring = state.reproduction(config.r_max, config.selection_sigma);
	state.density_regulation(&offspring);
	state.dispersal();
}

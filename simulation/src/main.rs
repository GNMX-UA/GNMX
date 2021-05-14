use simulation::{Environment::Constant, *};
use tinyvec::tiny_vec;

fn main() {
	let e = vec![0.5; 8].into_iter();
	let p = vec![
		Patch::new(vec![
			Individual {
				loci: tiny_vec![0.1, 0.1, 0.1, 0.1],
			};
			10000 / 8
		]);
		8
	];
	let mut state = State {
		tick:    0,
		patches: p.clone().into_iter().zip(e).collect(),
	};
	let mut config = Config {
		mutation_mu:     0.001,
		mutation_sigma:  0.01,
		mutation_step:   0.01,
		rec:             0.01,
		selection_sigma: 0.3,
		gamma:           0.0,
		diploid:         true,
		m:               1.0,
		environment:     Environment::Constant,
	};

	let x = State::mutation(
		p,
		config.mutation_mu,
		config.mutation_sigma,
		config.mutation_step,
	);
	for i in 0 .. 1000 {
		step(&mut state, &config);
	}
}

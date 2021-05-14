#[cfg(test)]
mod tests {
	use itertools::Itertools;

	use crate::*;

	#[test]
	fn step_test() {
		let e = vec![0.5; 8].into_iter();
		let p = vec![
			Patch::new(vec![
				Individual {
					loci: tiny_vec![0.1, 0.1, 0.1, 0.1],
				};
				4000 / 8
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
			r_max:           1000.0,
			selection_sigma: 0.3,
			gamma:           0.0,
			diploid:         false,
			m:               1.0,
		};

		let x = State::mutation(
			p,
			config.mutation_mu,
			config.mutation_sigma,
			config.mutation_step,
		);
		for i in 0 .. 100 {
			step(&mut state, &config);
		}
	}

	#[test]
	fn distr() {
		let distr = WeightedAliasIndex::new(vec![0.0, 200.0, 400.0]).unwrap();
		let x = distr.sample_iter(thread_rng()).take(1000).counts();
		dbg!(x);
	}

	#[test]
	fn reproduction() {
		let state = State {
			tick:    0,
			patches: vec![(
				Patch {
					individuals: vec![
						Individual {
							loci: tiny_vec!(0.0),
						};
						50
					],
				},
				1.0,
			)],
		};
		let x = state.reproduction(10.0, 1.0);
		dbg!(x);
	}

	#[test]
	fn adult_death() {
		let mut state = State {
			tick:    0,
			patches: vec![(
				Patch {
					individuals: vec![
						Individual {
							loci: tiny_vec!(0.0),
						};
						10
					],
				},
				1.0,
			)],
		};
		let x = state.adult_death(0.3);
		dbg!(x);
	}

	#[test]
	fn density_regulation() {
		let state = State {
			tick:    0,
			patches: vec![(
				Patch {
					individuals: vec![
						Individual {
							loci: tiny_vec!(0.0),
						},
						Individual {
							loci: tiny_vec!(1.0),
						},
					],
				},
				1.0,
			)],
		};
		let y = state.reproduction(10.0, 0.5);
		dbg!(&y);
		let death = vec![10];
		let x = state.density_regulation(y, &death);
		dbg!(x);
	}

	#[test]
	fn recombination() {
		let y = vec![Patch {
			individuals: vec![
				Individual {
					loci: tiny_vec!(0.0, 1.0, 2.0, 3.0),
				},
				Individual {
					loci: tiny_vec!(4.0, 5.0, 6.0, 7.0),
				},
			],
		}];
		let x = State::recombination(y, 0.5);
		dbg!(x);
	}

	#[test]
	fn hapoloid_generation() {
		let y = vec![Patch {
			individuals: vec![
				Individual {
					loci: tiny_vec!(0.0, 1.0, 2.0, 3.0),
				},
				Individual {
					loci: tiny_vec!(4.0, 5.0, 6.0, 7.0),
				},
			],
		}];
		let x = State::haploid_generation(y);
		dbg!(x);
	}

	#[test]
	fn dispersal() {
		let y = vec![
			Patch {
				individuals: vec![
					Individual {
						loci: tiny_vec!(0.0),
					},
					Individual {
						loci: tiny_vec!(1.0),
					},
				],
			},
			Patch {
				individuals: vec![
					Individual {
						loci: tiny_vec!(2.0),
					},
					Individual {
						loci: tiny_vec!(3.0),
					},
				],
			},
		];
		let x = State::dispersal(y, 0.75);
		dbg!(x);
	}

	#[test]
	fn mutation() {
		let y = vec![Patch {
			individuals: vec![Individual {
				loci: tiny_vec!(0.0, 0.0, 0.0, 0.0),
			}],
		}];
		let x = State::mutation(y, 0.5, 0.1, 0.1);
		dbg!(x);
	}
}

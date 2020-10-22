#![allow(dead_code, unused_variables)]

use std::collections::HashMap;
use std::convert::TryInto;

use bitvec::prelude::*;
use bitvec::slice::BitSlice;
use bitvec::vec::BitVec;
use itertools::Itertools;

const O: bool = false;
const I: bool = true;

fn bit_product(n: usize) -> Vec<Vec<bool>> {
    (0..n)
        .map(|_| vec![O, I])
        .multi_cartesian_product()
        .collect::<Vec<_>>()
}

#[derive(Debug)]
struct ModelScore {
    fit: f32,
    complexity: f32,
}

impl ModelScore {
    fn total(&self) -> f32 {
        self.fit + self.complexity
    }

    fn display(&self) -> String {
        format!(
            "fit = {}, complexity = {}, total = {}",
            self.fit,
            self.complexity,
            self.total()
        )
    }
}

#[derive(Debug)]
struct MarkovTheory {
    degree: usize,
    parameters: HashMap<(BitVec, bool), f32>,
}

impl MarkovTheory {
    fn sample(&self, n: usize) -> BitVec {
        // realistic midway start
        let mut result = BitVec::new();
        let roll: f32 = rand::random();
        let mut waterline = 0.;
        for (parameter, probability) in self.parameters.iter() {
            waterline += probability;
            if waterline > roll {
                let (prefix, _) = parameter;
                result.extend(prefix);
            }
        }
        // TODO: truncate if n was smaller than our degree
        // we're initialized, let's go
        while result.len() < n {
            let p = *self
                .parameters
                .get(&(result[result.len() - self.degree..].to_bitvec(), O))
                .unwrap();
            let roll: f32 = rand::random();
            if roll < p {
                result.push(O);
            } else {
                result.push(I);
            }
        }
        result
    }

    fn raw_likelihood(&self, data: &BitSlice) -> f64 {
        let mut total = 1.;
        for window in data.windows(self.degree + 1) {
            let (observation, tail) = window.split_at(self.degree);
            let next = tail[0];
            total *= *self
                .parameters
                .get(&(observation.to_bitvec(), next))
                .unwrap() as f64
        }
        total
    }

    fn log_loss(&self, data: &BitSlice) -> f32 {
        let mut total = 0.;
        for window in data.windows(self.degree + 1) {
            let (observation, tail) = window.split_at(self.degree);
            let next = tail[0];
            total += -self
                .parameters
                .get(&(observation.to_bitvec(), next))
                .unwrap()
                .log2();
        }
        total
    }

    fn uniform_random_theory(degree: usize) -> Self {
        let mut parameters = HashMap::new();
        let prefixes = if degree > 0 {
            bit_product(degree)
        } else {
            vec![Vec::new()]
        };
        for prefix_ in prefixes {
            let p: f32 = rand::random();
            let mut prefix = BitVec::new();
            prefix.extend(&prefix_);
            parameters.insert((prefix.clone(), O), p);
            parameters.insert((prefix.clone(), I), 1. - p);
        }
        MarkovTheory { degree, parameters }
    }

    fn maximum_likelihood_estimate(data: &BitSlice, degree: usize) -> Self {
        let k = degree.try_into().expect("degree should fit in a u32");
        let mut parameters = HashMap::with_capacity(2usize.pow(k));
        let prefix_size = degree;
        let prefixes = if prefix_size > 0 {
            bit_product(prefix_size)
        } else {
            vec![Vec::new()]
        };
        for prefix_ in prefixes {
            // surely there's a better way to initialize BitVecs?!
            let mut prefix: BitVec<LocalBits, usize> = BitVec::new();
            prefix.extend(&prefix_);
            assert_eq!(prefix.len(), prefix_size);
            let mut chances = 0;
            let mut zero_continuation = 0;
            let mut one_continuation = 0;
            for window in data.windows(prefix_size + 1) {
                let (observation, tail) = window.split_at(prefix_size);
                let next = tail[0];
                if prefix == observation {
                    chances += 1;
                    match next {
                        O => {
                            zero_continuation += 1;
                        }
                        I => {
                            one_continuation += 1;
                        }
                    }
                }
            }
            assert_eq!(chances, zero_continuation + one_continuation);
            parameters.insert(
                (prefix.clone(), O),
                zero_continuation as f32 / chances as f32,
            );
            parameters.insert(
                (prefix.clone(), I),
                one_continuation as f32 / chances as f32,
            );
        }
        Self { degree, parameters }
    }

    fn evaluate(&self, data: &BitSlice) -> ModelScore {
        let fit = self.log_loss(&data);
        let complexity = 2f32.powi(self.degree as i32) * 32.;
        ModelScore { fit, complexity }
    }
}

fn main() {
    println!("Hello information theory world!");

    let mut true_parameters = HashMap::new();
    true_parameters.insert((bitvec![0, 0, 0], O), 0.65);
    true_parameters.insert((bitvec![0, 0, 0], I), 0.35);
    true_parameters.insert((bitvec![0, 0, 1], O), 0.45);
    true_parameters.insert((bitvec![0, 0, 1], I), 0.55);

    true_parameters.insert((bitvec![0, 1, 0], O), 0.2);
    true_parameters.insert((bitvec![0, 1, 0], I), 0.8);
    true_parameters.insert((bitvec![0, 1, 1], O), 0.55);
    true_parameters.insert((bitvec![0, 1, 1], I), 0.45);

    true_parameters.insert((bitvec![1, 0, 0], O), 0.4);
    true_parameters.insert((bitvec![1, 0, 0], I), 0.6);
    true_parameters.insert((bitvec![1, 0, 1], O), 0.75);
    true_parameters.insert((bitvec![1, 0, 1], I), 0.25);

    true_parameters.insert((bitvec![1, 1, 0], O), 0.4);
    true_parameters.insert((bitvec![1, 1, 0], I), 0.6);
    true_parameters.insert((bitvec![1, 1, 1], O), 0.3);
    true_parameters.insert((bitvec![1, 1, 1], I), 0.7);
    let the_truth = MarkovTheory {
        degree: 3,
        parameters: true_parameters,
    };
    let data = the_truth.sample(10000);


    // let our_sample = vec![false, true, true, false, false, true, true, false, true, true, false, true, false, true, false, true, true, false, true, true, true, true, true, true, false, false, false, false, true, false, false, true, true, true, false, false, false, false, true, false, false, false, true, true, false, true, false, false, false, true, true, false, true, false, true, true, false, true, true, false, true, false, false, false, false, false, false, true, false, true, false, false, false, false, false, false, true, false, true, false, true, false, true, false, false, true, true, true, true, false, true, false, false, false, true, false, true, true, true, true, false, true, false, true, false, false, true, false, false, true, false, true, false, true, false, false, true, false, true, false, true, false, true, false, true, false, true, false, false, false, false, true, false, true, false, false, true, true, false, true, false, true, false, true, false, false, true, true, true, true, true, true, true, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, false, true, true, true, true, true, true, true, false, true, false, true, false, true, true, false, true, false, true, false, true, true, false, true, true, true, true, true, false, true, false, true, false, true, true, false, true, true, false, true, false, true, false, true, false, true, false, false, false, false, false, false, false, true, true, false, true, true, true, true, true, false, false, false, false, false, true, true, true, false, true, false, true, true, true, false, false, false, false, false, false, false, false, false, false, false, false, false, true, true, true, true, true, false, true, false, true, false, true, true, false, true, false, true, false, true, false, true, false, true, false, false, true, false, true, false, true, false, true, true, false, true, false, true, false, true, false, true, true, false, false, true, true, true, false, false, true, true, false, false, false, false, true, true, false, false, true, true, false, true, false, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, false, false, false, true, true, false, false, true, false, true, true, false, true, false, false, true, true, false, true, false, true, false, true, false, true, false, true, false, true, true, false, false, false, false, false, false, true, false, true, false, true, false, true, true, true, false, true, true, false, true, false, true, false, false, true, false, true, true, false, false, true, true, true, true, true, true, true, false, true, false, true, true, true, true, false, true, true, true, false, true, false, false, false, true, false, true, false, true, false, true, false, true, true, true, false, false, true, true, true, true, false, true, false, false, false, true, true, false, true, true, false, true, false, true, false, true, false, true, true, false, true, false, true, true, false, false, false, true, false, true, true, false, false, false, false, false, true, false, true, false, true, false, true, false, false, true, true, false, false, true, true, false, true, false, true, false, true, false, true, true, true, true];
    // let mut data = BitVec::new();
    // data.extend(&our_sample);

    println!("observed data is {:?}", data);

    for hypothesized_degree in 0..20 {
        let theory = MarkovTheory::maximum_likelihood_estimate(&data, hypothesized_degree);
        if hypothesized_degree == 0 {
            println!("{:?}", theory);
        }
        println!(
            "{}th-order theory: {}",
            hypothesized_degree,
            theory.evaluate(&data).display()
        );
        // println!("raw fit: {}", theory.raw_likelihood(&data));
    }
}

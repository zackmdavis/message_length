#![allow(dead_code, unused_variables)]

use std::collections::HashMap;
use std::convert::TryInto;

use bitvec::prelude::*;
use bitvec::slice::BitSlice;
use bitvec::vec::BitVec;
use itertools::Itertools;

const O: bool = false;
const I: bool = true;

#[derive(Debug)]
struct ModelScore {
    fit: f64,
    complexity: f64,
}

impl ModelScore {
    fn total(&self) -> f64 {
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
    parameters: HashMap<(BitVec, bool), f64>,
}

impl MarkovTheory {
    fn sample(&self, n: usize) -> BitVec {
        // realistic midway start
        let mut result = BitVec::new();
        let roll: f64 = rand::random();
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
            let roll: f64 = rand::random();
            if roll < p {
                result.push(O);
            } else {
                result.push(I);
            }
        }
        result
    }

    fn likelihood(&self, data: &BitSlice) -> f64 {
        let mut total = 1.;
        for window in data.windows(self.degree + 1) {
            let (observation, tail) = window.split_at(self.degree);
            let next = tail[0];
            total *= self
                .parameters
                .get(&(observation.to_bitvec(), next))
                .unwrap();
        }
        total
    }

    fn maximum_likelihood_estimate(data: &BitSlice, degree: usize) -> Self {
        let k = degree.try_into().expect("degree should fit in a u32");
        let mut parameters = HashMap::with_capacity(2usize.pow(k));
        let prefix_size = degree;
        let prefixes = if prefix_size > 0 {
            (0..prefix_size)
                .map(|_| vec![O, I])
                .multi_cartesian_product()
                .collect::<Vec<_>>()
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
                zero_continuation as f64 / chances as f64,
            );
            parameters.insert(
                (prefix.clone(), I),
                one_continuation as f64 / chances as f64,
            );
        }
        Self { degree, parameters }
    }

    fn evaluate(&self, data: &BitSlice) -> ModelScore {
        let fit = -self.likelihood(data).log2();
        let complexity = 2f64.powi(self.degree as i32);
        ModelScore { fit, complexity }
    }
}

fn main() {
    println!("Hello information theory world!");
    let mut true_parameters = HashMap::new();
    true_parameters.insert((bitvec![0, 0], O), 0.6);
    true_parameters.insert((bitvec![0, 0], I), 0.4);
    true_parameters.insert((bitvec![0, 1], O), 0.7);
    true_parameters.insert((bitvec![0, 1], I), 0.3);
    true_parameters.insert((bitvec![1, 0], O), 0.8);
    true_parameters.insert((bitvec![1, 0], I), 0.2);
    true_parameters.insert((bitvec![1, 1], O), 0.9);
    true_parameters.insert((bitvec![1, 1], I), 0.1);
    let the_truth = MarkovTheory {
        degree: 2,
        parameters: true_parameters,
    };
    let data = the_truth.sample(1000);
    println!("observed data is {:?}", data);
    for i in 0..8 {
        let theory = MarkovTheory::maximum_likelihood_estimate(&data, i);
        println!("{}th-order theory: {}", i, theory.evaluate(&data).display())
    }
}

#[cfg(test)]
mod tests {
    use super::{I, O};
    use itertools::Itertools;

    #[test]
    fn test_bit_product() {
        assert_eq!(
            (0..2)
                .map(|_| vec![O, I])
                .multi_cartesian_product()
                .collect::<Vec<_>>(),
            vec![vec![O, O], vec![O, I], vec![I, O], vec![I, I]]
        );
    }
}

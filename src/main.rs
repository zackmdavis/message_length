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
struct MarkovTheory {
    degree: usize,
    parameters: HashMap<(BitVec, bool), f32>,
}

impl MarkovTheory {
    fn maximum_likelihood(data: &BitSlice, degree: usize) -> Self {
        let k = degree.try_into().expect("degree should fit in a u32");
        let mut parameters = HashMap::with_capacity(2usize.pow(k));
        for prefix_size in 0..degree + 1 {
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
                    zero_continuation as f32 / chances as f32,
                );
                parameters.insert(
                    (prefix.clone(), I),
                    one_continuation as f32 / chances as f32,
                );
            }
        }
        Self { degree, parameters }
    }
}

fn main() {
    let mut data = BitVec::new();
    data.extend(&[
        O, I, O, I, O, I, O, O, O, O, I, I, I, O, I, O, I, O, I, O, I, O, I, O, I, O, I,
    ]);
    let zero_theory = MarkovTheory::maximum_likelihood(&data, 0);
    let one_theory = MarkovTheory::maximum_likelihood(&data, 1);
    let two_theory = MarkovTheory::maximum_likelihood(&data, 2);
    let three_theory = MarkovTheory::maximum_likelihood(&data, 3);
    println!("Hello information theory world!");
    println!("zeroth-order theory: {:?}", zero_theory);
    println!("first-order theory: {:?}", one_theory);
    println!("second-order theory: {:?}", two_theory);
    println!("third-order theory: {:?}", three_theory);
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

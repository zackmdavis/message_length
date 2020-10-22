#!/usr/bin/env run-cargo-script
// cargo-deps: rand="0.7"

// Use cargo-script (https://github.com/DanielKeep/cargo-script) to run as a
// standalone script.

extern crate rand;

use std::collections::HashMap;

type Bit = bool;
const ZERO: Bit = false;
const ONE: Bit = true;


fn sample(markov_chain: HashMap<(Vec<Bit>, Bit), f32>, count: usize) -> Vec<Bit> {
    let degree = log2(markov_chain.keys().count()) - 1;
    assert!(count > degree);
    let mut result = Vec::new();
    // set up
    for (parameter, probability) in markov_chain.iter() {
        let (prefix, _) = parameter;
        result.extend(prefix);
        break;
    }
    // we're initialized, let's go
    while result.len() < count {
        let p = *markov_chain
            .get(&(result[result.len() - degree..].to_vec(), ZERO))
            .unwrap();
        let roll: f32 = rand::random();
        if roll < p {
            result.push(ZERO);
        } else {
            result.push(ONE);
        }
    }
    result
}


fn bit_product(n: usize) -> Vec<Vec<Bit>> {
    // Thanks to
    // https://docs.python.org/3/library/itertools.html#itertools.product for
    // guidance
    let factors = (0..n).map(|_| vec![ZERO, ONE]).collect::<Vec<_>>();
    let mut result = vec![Vec::new()];
    for factor in &factors {
        let mut iteration = Vec::new();
        for subresult in &result {
            for &bit in factor {
                let mut item = Vec::new();
                item.extend(subresult);
                item.push(bit);
                iteration.push(item);
            }
        }
        result = iteration;
    }
    result
}

fn maximum_likelihood_estimate(data: &[Bit], degree: usize) -> HashMap<(Vec<Bit>, Bit), f32> {
    let mut theory = HashMap::with_capacity(2usize.pow(degree as u32));
    // Cartesian product—e.g., if degree 2, [00, 01, 10, 11]
    let patterns = bit_product(degree);
    for pattern in patterns {
        let mut zero_continuations = 0;
        let mut one_continuations = 0;
        for window in data.windows(degree + 1) {
            let (prefix, tail) = window.split_at(degree);
            let next = tail[0];
            if prefix == pattern {
                match next {
                    ZERO => {
                        zero_continuations += 1;
                    }
                    ONE => {
                        one_continuations += 1;
                    }
                }
            }
        }
        let continuations = zero_continuations + one_continuations;
        theory.insert(
            (pattern.clone(), ZERO),
            zero_continuations as f32 / continuations as f32,
        );
        theory.insert(
            (pattern.clone(), ONE),
            one_continuations as f32 / continuations as f32,
        );
    }
    theory
}

fn log2(x: usize) -> usize {
    // thanks to Michael Lamparski
    // (https://users.rust-lang.org/t/logarithm-of-integers/8506/5)
    ((std::mem::size_of::<usize>() * 8) as u32 - x.leading_zeros() - 1) as usize
}

fn log_loss(theory: &HashMap<(Vec<Bit>, Bit), f32>, data: &[Bit]) -> f32 {
    let mut total = 0.;
    let degree = log2(theory.keys().count()) - 1;
    for window in data.windows(degree + 1) {
        let (prefix, tail) = window.split_at(degree);
        let next = tail[0];
        total += -theory
            .get(&(prefix.to_vec(), next))
            .expect("theory should have param value for prefix-and-continuation")
            .log2();
    }
    total
}

fn main() {
    let mut true_theory = HashMap::new();
    true_theory.insert((vec![ZERO, ZERO, ZERO], ZERO), 0.65);
    true_theory.insert((vec![ZERO, ZERO, ZERO], ONE), 0.35);
    true_theory.insert((vec![ZERO, ZERO, ONE], ZERO), 0.45);
    true_theory.insert((vec![ZERO, ZERO, ONE], ONE), 0.55);
    true_theory.insert((vec![ZERO, ONE, ZERO], ZERO), 0.2);
    true_theory.insert((vec![ZERO, ONE, ZERO], ONE), 0.8);
    true_theory.insert((vec![ZERO, ONE, ONE], ZERO), 0.55);
    true_theory.insert((vec![ZERO, ONE, ONE], ONE), 0.45);
    true_theory.insert((vec![ONE, ZERO, ZERO], ZERO), 0.4);
    true_theory.insert((vec![ONE, ZERO, ZERO], ONE), 0.6);
    true_theory.insert((vec![ONE, ZERO, ONE], ZERO), 0.75);
    true_theory.insert((vec![ONE, ZERO, ONE], ONE), 0.25);
    true_theory.insert((vec![ONE, ONE, ZERO], ZERO), 0.4);
    true_theory.insert((vec![ONE, ONE, ZERO], ONE), 0.6);
    true_theory.insert((vec![ONE, ONE, ONE], ZERO), 0.3);
    true_theory.insert((vec![ONE, ONE, ONE], ONE), 0.7);
    let data = sample(true_theory, 10000);
    for hypothesized_degree in 0..10 {
        let theory = maximum_likelihood_estimate(&data, hypothesized_degree);
        let fit = log_loss(&theory, &data);
        let complexity = 2f32.powi(hypothesized_degree as i32) * 32.;
        println!(
            "{}th-order theory: fit = {}, complexity = {}, total = {}",
            hypothesized_degree, fit, complexity, fit + complexity
        );
    }
}

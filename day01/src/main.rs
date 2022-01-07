// in `day01/src/main.rs

// anyhow is a crate that helps with error handling; it comes with an error type that can contain any other error.
// So the definition of anyhow::Result is actually: `pub type Result<T, E = Error> = core::result::Result<T, E>;`
// And the Error here is anyhow::Error.
use anyhow::Result;
use itertools::Itertools;

fn main() -> anyhow::Result<()> {
    /*
    let pair = find_pair_whose_sum_is_2020(
        // include input.txt at compile-time
        // split by newlines, producing a stream of items
        // we parse Iterator<Item = &str> values to Iterator<Item = i64> values
        // unwrap all the items retrieved from the iterator
        // "?" after collect() acts like unwrap(); takes Result<T, E> and evaluates to a T
        include_str!("input.txt")
            .split('\n')
            .map(str::parse::<i64>)
            .collect::<Result<Vec<_>, _>>()?,
    );
    dbg!(pair);
    Ok(())
    */

    // Part 1: find the two entries that sum to 2020
    let (a, b) = include_str!("input.txt")
        .split('\n')
        .map(str::parse::<i64>)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .tuple_combinations()
        .filter(|(a, b)| a != b)
        .find(|(a, b)| a + b == 2020)
        .expect("no pair had a sum of 2020");

    println!("part 1:");
    println!("  a: {}  b: {}", a, b);
    println!("  a + b = {}", a + b);
    println!("  a * b = {}", a * b);

    // Part 2: find the three entries that sum to 2020
    let (a, b, c) = include_str!("input.txt")
        .split('\n')
        .map(str::parse::<i64>)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .tuple_combinations()
        .find(|(a, b, c)| a + b + c == 2020)
        .expect("no tuple of length 3 had a sum of 2020");

    println!("part 2:");
    println!("  a: {}  b: {}  c: {}", a, b, c);
    println!("  a + b + c = {}", a + b + c);
    println!("  a * b * c = {}", a * b * c);

    Ok(())
}

#[allow(dead_code)]
fn find_pair_whose_sum_is_2020(s: Vec<i64>) -> Option<(i64, i64)> {
    /*
    for i in 0..s.len() {
        for j in 0..s.len() {
            // require that solution pairs be made up of "different items"
            if i == j {
                continue;
            }
            if s[i] + s[j] == 2020 {
                return Some((s[i], s[j]));
            }
        }
    }
    None
    */

    /*
    for (a, b) in all_pairs(&s[..]) {
        if a == b {
            continue
        }
        if a + b == 2020 {
            return Some((a, b));
        }
    }
    None
    */

    all_pairs(&s[..])
        .into_iter()
        .filter(|(a, b)| a != b)
        .find(|(a, b)| a + b == 2020)
}

#[allow(dead_code)]
fn all_pairs(s: &[i64]) -> Vec<(i64, i64)> {
    let mut pairs: Vec<_> = Default::default();
    for i in 0..s.len() {
        for j in 0..s.len() {
            pairs.push((s[i], s[j]))
        }
    }
    pairs
}

/*
// Instead of returning a Vec<(i64, i64)> from all_pairs, we could return...
// an Iterator<Item = (i64, i64)> ... itertools crate helps avoid gnarly code like this
fn all_pairs(s: &[i64]) -> impl Iterator<Item = (i64, i64)> + '_ {
    s.iter()
        .copied()
        .enumerate()
        .map(move |(a_index, a)| {
            s.iter().copied().enumerate().filter_map(
                move |(b_index, b)| {
                    if a_index == b_index {
                        None
                    } else {
                        Some((a, b))
                    }
                },
            )
        })
        .flatten()
}
*/

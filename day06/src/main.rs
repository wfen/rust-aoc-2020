//use std::{fmt, collections::HashSet};
use std::fmt;
// im provides a set of immutable data structures, including HashSet; unions method
use im::HashSet;

// im docs: All of these data structures support in-place copy-on-write mutation which
// means that if you're the sole user of a data structure, you can update it in place
// without taking the performance hit of making a copy of the data structure before
// modifying it (this is about an order of magnitude faster than immutable operations,
// almost as fast as std::collection's mutable data structures.
//
// If you aren't the sole owner of an im data structure, you'll get an automatic copy
// of the node before modifying it. Cloning a data structure becomes a lazy operation.

// get reasonable readability for our output by supplying our own Debug
pub struct Answers(HashSet<u8>);

impl fmt::Debug for Answers {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &answer in &self.0 {
            write!(f, "{}", answer as char)?;
        }
        Ok(())
    }
}

// The standard library lets us compute the union of two HashSets and returns an iterator.
// We could chain all our iterators and collect the resulting iterator into a single HashSet

fn main() {
    /*
    // collect the answers for each person of a group - we don't need a parser to do that much
    let answers: Vec<_> = include_str!("input.txt")
        .split("\n\n")
        .map(|group| {
            group
                .lines()
                .map(|line| Answers(line.as_bytes().iter().copied().collect()))
                .collect::<Vec<_>>()
        })
        .collect();

    let group_answers: Vec<_> = answers
        .into_iter()
        .map(|group| Answers(HashSet::unions(group.into_iter().map(|x| x.0))))
        .collect();
    */

    // reworked to skip the intermediate step of collecting answers for each person of a group
    let _group_answers: Vec<_> = include_str!("input.txt")
        .split("\n\n")
        .map(|group| {
            Answers(HashSet::unions(
                group
                    .lines()
                    .map(|line| line.as_bytes().iter().copied().collect()),
            ))
        })
        .collect();

    //dbg!(&_group_answers[0..5]);

    // we skip additional intermediate steps to simplify the work to answer to part 1
    let answer: usize = include_str!("input.txt")
        .split("\n\n")
        .map(|group| {
            HashSet::<u8>::unions(
                group
                    .lines()
                    .map(|line| line.as_bytes().iter().copied().collect()),
            )
                .len()
        })
        .sum();

    println!("Part 1:");
    println!("  The sum of all questions answered by all the groups is {:?}", answer);

    // note: intersection of empty set with anything is the empty set (avoid fold using HashSet::<u8>::new())
    // our initial use of "fold" replaced by "reduce" For iterators with at least one element, same as fold() with
    // the first element of the iterator as the initial accumulator value, folding every subsequent element into it.
    //let init: HashSet<u8> = (b'a'..=b'z').collect();

    // keep only the answers to which everyone in the group said yes
    // unwrap_or_default needed for the situation where we are reduce-ing a collection of 0 items
    let answer2: usize = include_str!("input.txt")
        .split("\n\n")
        .map(|group| {
                group
                    .lines()
                    .map(|line| line.as_bytes().iter().copied().collect())
                    .reduce(|acc: HashSet<u8>, x| acc.intersection(x))
                    .unwrap_or_default()
                    .len()
        })
        .sum();

    println!("Part 2:");
    println!("  The sum of all questions answered by all individuals in each group is {:?}", answer2);

}

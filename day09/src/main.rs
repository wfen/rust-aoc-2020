use itertools::Itertools;

fn main() {
    let numbers = include_str!("input.txt")
        .lines()
        .map(|x| x.parse::<usize>().unwrap())
        .collect::<Vec<_>>();

    let n = 25;
    let answer = numbers.windows(n + 1).find_map(|s| {
        if (&s[..n])
            .iter()
            .tuple_combinations()
            .any(|(a, b)| a + b == s[n])
        {
            None
        } else {
            Some(s[n])
        }
    });
    println!("Part 1:");
    println!("  answer = {:?}", answer);

    let answer = answer.unwrap();

    /*
    // simply try to find a contiguous set of numbers whose sum is the same as the answer we found in part1
    // we don't know afterwards where or how large the set was
    let _answer = (2..numbers.len())
        .into_iter().flat_map(|n| numbers.windows(n).map(|s| s.iter().sum::<usize>()))
        .find(|&n| n == answer);
    println!("  answer2 = {:?}", _answer);
    */

    let answer2 = (2..numbers.len())
        .into_iter().flat_map(|n| {
            numbers
                .windows(n)
                .enumerate()
                .map(move |(i, s)| (n, i, s.iter().sum::<usize>()))
        })
        .find(|&(_, _, sum)| sum == answer);

    let (n, i, _) = answer2.unwrap();
    let set = &numbers[i..][..n];

    println!("Part 2:");
    println!("  sum({:?}) = {}", set, answer);
    let answer3 = set.iter().max().unwrap() + set.iter().min().unwrap();
    println!("  sum of min() and max() for this contiguous range = {}", answer3);
}

use std::collections::HashMap;

#[derive(Default, Clone, Copy, Debug)]
struct Results {
    ones: usize,
    threes: usize,
}

fn main() {

    // we have this list of numbers...
    let mut numbers: Vec<_> = std::iter::once(0)
        .chain(
            include_str!("input.txt")
                .lines()
                .map(|x| x.parse::<usize>().unwrap()),
        )
        .collect();
    // clippy told me to use `sort_unstable`
    numbers.sort_unstable();

    // to which we need to add 0 and whatever the maximum was plus three
    if let Some(&max) = numbers.iter().max() {
        // numbers is still sorted after this
        numbers.push(max + 3);
    }

    // from there on, if we take them in order, we'll have gaps of 1 and gaps of 3
    // we need to multiply the amount of 1-gaps with the amount of 3-gaps
    // recent rust versions allow use of the method array_windows [usize; 2]
    let results = numbers.windows(2).fold(Results::default(), |acc, s| {
        if let [x, y] = s {
            match y - x {
                1 => Results {
                    ones: acc.ones + 1,
                    ..acc
                },
                3 => Results {
                    threes: acc.threes + 1,
                    ..acc
                },
                gap => panic!("invalid input (found {} gap)", gap),
            }
        } else {
            unreachable!()
        }
    });
    dbg!(results, results.ones * results.threes);

    // part2 wants all the possible ways in which we can connect our adapters
    // given 1, 2, 3, 5, 6 ... [1 2 3 5 6], [1 2 3 6], [1 2 5 6], [1 3 5 6], or [1 3 6] = 5 ways
    // ways to 6
    // node_6 = 1
    // node_5 = node_6 = 1
    // node3 = node_5 + node_6 = 1 + 1 = 2
    // node 2 = node_3 + node_5 = 2 + 1 = 3
    // node_1 = node_2 + node_3 = 3 + 2 = 5
    // rules stipulate an initial node of 0 and a final node of max+3

    let mut numbers: Vec<_> = std::iter::once(0)
        .chain(
            // sample0.txt file contains 1, 2, 3, 5, 6
            include_str!("input.txt")
                .lines()
                .map(|x| x.parse::<usize>().unwrap()),
        )
        .collect();
    numbers.sort_unstable();

    // numbers is still sorted after this
    numbers.push(numbers.iter().max().unwrap() + 3);

    let mut num_paths = HashMap::new();

    let n = numbers.len();
    num_paths.insert(numbers.last().copied().unwrap(), 1);
    for i in (0..(numbers.len() - 1)).into_iter().rev() {
        let i_val = numbers[i];
        let range = (i + 1)..=std::cmp::min(i + 3, n - 1);

        let num_neighbors: usize = range
            .filter_map(|j| {
                let j_val = numbers[j];
                let gap = j_val - i_val;
                if (1..=3).contains(&gap) {
                    Some(num_paths.get(&j_val).unwrap())
                } else {
                    None
                }
            })
            .sum();
        num_paths.insert(i_val, num_neighbors);
    }

    for &n in numbers.iter().rev() {
        let &m = num_paths.get(&n).unwrap();
        println!(
            "from {}, there's {} {}",
            n,
            m,
            if m == 1 { "path" } else { "paths" }
        );
    }

}

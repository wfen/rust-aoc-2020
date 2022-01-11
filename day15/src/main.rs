use std::collections::HashMap;

type Turn = usize;
type Number = i64;

struct NumberGame {
    last_turns: HashMap<Number, Turn>,
    prev_turns: HashMap<Number, Turn>,
    starting_numbers: Vec<Number>,
    next_turn: Turn,
    last_spoken: Number
}

impl NumberGame {
    fn new(starting_numbers: &[Number]) -> Self {
        NumberGame {
            last_turns: HashMap::new(),
            prev_turns: HashMap::new(),
            starting_numbers: starting_numbers.iter().cloned().collect(),
            next_turn: 0,
            last_spoken: 0
        }
    }
}

impl Iterator for NumberGame {
    type Item = Number;

    fn next(&mut self) -> Option<Number> {
        let next_number = if self.next_turn < self.starting_numbers.len() {
            self.starting_numbers[self.next_turn]
        } else {
            let last = self.last_turns.get(&self.last_spoken).unwrap();
            match self.prev_turns.get(&self.last_spoken) {
                None => 0,
                Some(prev) => (last - prev) as Number
            }
        };

        if let Some(prev) = self.last_turns.get(&next_number) {
            self.prev_turns.insert(next_number, *prev);
        }
        self.last_turns.insert(next_number, self.next_turn);
        self.last_spoken = next_number;
        self.next_turn += 1;

        Some(next_number)
    }
}


fn number_spoken_at_index(starting_numbers: &[Number], target_index: Turn) -> Number {
    NumberGame::new(starting_numbers)
        .nth(target_index - 1)
        .unwrap()
}

fn part1(starting_numbers: &[Number]) -> Number {
    number_spoken_at_index(starting_numbers, 2020)
}

fn part2(starting_numbers: &[Number]) -> Number {
    number_spoken_big(starting_numbers, 30000000)
}

// number_spoken_big() uses a dynamic programming implementation
fn number_spoken_big(starting_numbers: &[Number], last: usize) -> Number {
    let mut turns_spoken: HashMap<Number, usize> = starting_numbers
        .iter()
        .take(starting_numbers.len() - 1)
        .enumerate()
        .map(|(i, x)| (*x, i))
        .collect();
    let mut last_spoken = *starting_numbers.last().unwrap();
    for i in starting_numbers.len()..last {
        let newly_spoken = match turns_spoken.get(&last_spoken) {
            Some(last_time) => i - *last_time - 1,
            None => 0,
        };
        turns_spoken.insert(last_spoken, i - 1);
        last_spoken = newly_spoken as Number;
    }
    last_spoken
}

fn main() {
    let input = [0,5,4,1,10,14,7];
    println!("part 1 {}", part1(&input));
    println!("part 2 {}", part2(&input));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_spoken_at_index() {
        assert_eq!(number_spoken_at_index(&[0,3,6], 10), 0);
        assert_eq!(number_spoken_big(&[0,3,6], 30000000), 175594);
    }
}

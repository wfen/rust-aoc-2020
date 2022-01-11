type Cup = usize;

#[derive(Debug)]
struct Cups {
    data: Vec<Cup>,
    next: Vec<Cup>,
    current_cup: usize,
    current_move: usize,
}

impl Cups {
    fn new(input: &str, size: usize) -> Self {
        let mut data: Vec<Cup> = Vec::with_capacity(size);

        for ch in input.trim().chars() {
            let cup = ch.to_digit(10).unwrap() as Cup;
            data.push(cup);
        }

        for cup in (*data.iter().max().unwrap() + 1)..(size + 1) {
            data.push(cup);
        }

        let mut next: Vec<Cup> = vec![0; data.len() + 1];
        for index in 0..(data.len() - 1) {
            next[data[index]] = data[index + 1];
        }
        next[data[data.len() - 1]] = data[0];

        let current_cup = data[0];

        Cups {
            data,
            next,
            current_cup,
            current_move: 0,
        }
    }

    fn stepn(&mut self, count: usize) {
        for _index in 0..count {
            self.step();
        }
    }

    fn step(&mut self) {
        self.current_move += 1;

        let mut moving_cups: Vec<Cup> = Vec::with_capacity(3);
        let mut cup: Cup = self.current_cup;
        for _ in 0..3 {
            cup = self.next[cup];
            moving_cups.push(cup);
        }

        let destination_cup: Cup = self.destination(&moving_cups);

        //        self.print_step(&moving_cups, destination_cup);
        self.move_cups(&moving_cups, destination_cup);

        self.current_cup = self.next[self.current_cup];
    }

    fn destination(&self, moving_cups: &[Cup]) -> Cup {
        let mut destination_cup: Cup = self.previous_cup(self.current_cup);

        loop {
            if moving_cups.contains(&destination_cup) {
                destination_cup = self.previous_cup(destination_cup)
            } else {
                break;
            }
        }

        destination_cup
    }

    fn previous_cup(&self, cup: Cup) -> Cup {
        let mut previous_cup: Cup = cup - 1;

        if previous_cup <= 0 {
            previous_cup += self.data.len() as Cup;
        }

        previous_cup
    }

    fn move_cups(&mut self, cups: &[Cup], destination_cup: Cup) {
        self.next.swap(destination_cup, self.current_cup);
        self.next.swap(self.current_cup, cups[2]);
    }

    fn output(&mut self) -> String {
        let mut cup: Cup = self.next[1];
        let mut cups: Vec<Cup> = vec![cup];

        for _ in 0..7 {
            cup = self.next[cup];
            cups.push(cup);
        }

        cups.iter().map(|&cup| (48 + (cup as u8)) as char).collect()
    }

    #[allow(dead_code)]
    fn print_step(&self, moving_cups: &[Cup], destination_cup: Cup) {
        println!("-- move {} --", self.current_move);
        print!("cups: ");
        let mut cups = vec![0; 9];
        let mut cup = 7;

        for index in 0..9 {
            cup = self.next[cup];
            cups[index] = cup;
        }

        for cup in cups {
            if cup == self.current_cup {
                print!("({}) ", cup);
            } else {
                print!("{} ", cup);
            }
        }
        println!();
        println!("pick up: {:?}", moving_cups);
        println!("destination: {}\n", destination_cup);
    }
}

#[must_use]
pub fn part1(input: &str) -> String {
    let mut cups: Cups = Cups::new(input, 9);

    cups.stepn(100);

    cups.output()
}

#[must_use]
pub fn part2(input: &str) -> usize {
    let mut cups: Cups = Cups::new(input, 1_000_000);

    cups.stepn(10_000_000);

    let one = cups.next[1];
    let two = cups.next[one];

    one * two
}

fn main() {
    let input = "157623984";
    println!("part 1 {}", part1(&input));
    println!("part 2 {}", part2(&input));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_short_example() {
        let mut cups: Cups = Cups::new("389125467\n", 9);

        cups.stepn(10);

        assert_eq!(cups.output(), "92658374");
    }

    #[test]
    fn part1_full_example() {
        assert_eq!(part1("389125467\n"), "67384529");
    }

    #[test]
    fn part2_example() {
        assert_eq!(part2("389125467\n"), 149245887792)
    }
}

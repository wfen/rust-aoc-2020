use std::collections::HashSet;
use std::collections::VecDeque;
use std::str::FromStr;

#[derive(Debug)]
struct Player {
    id: u8,
    deck: VecDeque<u8>,
}

impl FromStr for Player {
    type Err = ();
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut lines = input.trim().lines();
        let id: u8 = lines
            .next()
            .and_then(|line| line.strip_prefix("Player "))
            .and_then(|line| line.strip_suffix(":"))
            .and_then(|line| line.parse().ok())
            .unwrap();
        let deck: VecDeque<u8> = lines.map(|line| line.parse().unwrap()).collect();

        Ok(Self { id, deck })
    }
}

fn play_round(deck1: &mut VecDeque<u8>, deck2: &mut VecDeque<u8>) {
    //    println!("Player 1's deck: {:?}", &deck1);
    //    println!("Player 2's deck: {:?}", &deck2);

    let card1 = deck1.pop_front().unwrap();
    let card2 = deck2.pop_front().unwrap();

    //    println!("Player 1 plays: {}", &card1);
    //    println!("Player 2 plays: {}", &card2);

    if card1 > card2 {
        //        println!("Player 1 wins the round!");
        deck1.push_back(card1);
        deck1.push_back(card2);
    } else {
        //        println!("Player 2 wins the round!");
        deck2.push_back(card2);
        deck2.push_back(card1);
    }
}

fn play_recursive_round(
    deck1: &mut VecDeque<u8>,
    deck2: &mut VecDeque<u8>,
    game: usize,
    _round: usize,
) {
    //    println!("Player 1's deck: {:?}", &deck1);
    //    println!("Player 2's deck: {:?}", &deck2);

    let card1 = deck1.pop_front().unwrap();
    let card2 = deck2.pop_front().unwrap();

    //    println!("Player 1 plays: {}", &card1);
    //    println!("Player 2 plays: {}", &card2);

    if deck1.len() >= card1 as usize && deck2.len() >= card2 as usize {
        //        println!("Playing a sub-game to determine the winner...");

        let mut deck1_copy = deck1.iter().take(card1 as usize).copied().collect();
        let mut deck2_copy = deck2.iter().take(card2 as usize).copied().collect();

        play_game(&mut deck1_copy, &mut deck2_copy, true, game + 1);

        if deck1_copy.len() == 0 {
            //            println!("Player 2 wins game {} and therefore game {}, round {}!", game, game - 1, round);
            deck2.push_back(card2);
            deck2.push_back(card1);
        } else {
            //            println!("Player 1 wins game {} and therefore game {}, round {}!", game, game - 1, round);
            deck1.push_back(card1);
            deck1.push_back(card2);
        }
    } else {
        if card1 > card2 {
            //            println!("Player 1 wins the round!");
            deck1.push_back(card1);
            deck1.push_back(card2);
        } else {
            //            println!("Player 2 wins the round!");
            deck2.push_back(card2);
            deck2.push_back(card1);
        }
    }
}

fn play_game(
    mut deck1: &mut VecDeque<u8>,
    mut deck2: &mut VecDeque<u8>,
    recursive: bool,
    game: usize,
) {
    //    println!("\n=== Game {} ===", game);

    let mut states_seen: HashSet<Vec<u8>> = HashSet::new();
    let mut round: usize = 1;

    loop {
        let state: Vec<u8> = deck1
            .iter()
            .chain(vec![0].iter())
            .chain(deck2.iter())
            .copied()
            .collect();

        if deck1.len() == 0 || deck2.len() == 0 {
            break;
        }
        if states_seen.contains(&state) {
            deck2.clear();
            break;
        }

        states_seen.insert(state);

        //        println!("\n-- Round {} (Game {}) --", round, game);

        if recursive {
            play_recursive_round(&mut deck1, &mut deck2, game, round);
        } else {
            play_round(&mut deck1, &mut deck2);
        }

        round += 1;
    }
}

fn compute_score(deck: &VecDeque<u8>) -> usize {
    let count = deck.len();
    deck.iter().enumerate().fold(0, |sum, (index, &card)| {
        sum + (card as usize) * (count - index)
    })
}

#[must_use]
pub fn part1(input: &str) -> usize {
    let player_inputs: Vec<&str> = input.split("\n\n").collect();
    let mut player1: Player = player_inputs.get(0).unwrap().parse().unwrap();
    let mut player2: Player = player_inputs.get(1).unwrap().parse().unwrap();

    play_game(&mut player1.deck, &mut player2.deck, false, 1);

    if player1.deck.len() == 0 {
        compute_score(&player2.deck)
    } else {
        compute_score(&player1.deck)
    }
}

#[must_use]
pub fn part2(input: &str) -> usize {
    let player_inputs: Vec<&str> = input.split("\n\n").collect();
    let mut player1: Player = player_inputs.get(0).unwrap().parse().unwrap();
    let mut player2: Player = player_inputs.get(1).unwrap().parse().unwrap();

    play_game(&mut player1.deck, &mut player2.deck, true, 1);

    if player1.deck.len() == 0 {
        compute_score(&player2.deck)
    } else {
        compute_score(&player1.deck)
    }
}

fn main() {
    let input = include_str!("input.txt");
    println!("part 1 {}", part1(&input));
    println!("part 2 {}", part2(&input));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> &'static str {
        "\
Player 1:
9
2
6
3
1

Player 2:
5
8
4
7
10
"
    }

    #[test]
    fn part1_example() {
        assert_eq!(part1(input()), 306)
    }

    #[test]
    fn part2_example() {
        assert_eq!(part2(input()), 291)
    }

    #[test]
    fn part2_infinite_recursion_test() {
        assert_eq!(
            part2(
                "\
Player 1:
43
19

Player 2:
2
29
14"
            ),
            105
        );
    }
}

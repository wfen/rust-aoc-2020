
type Subject = u64;
type Key = u64;
type LoopSize = usize;

fn transform(subject: Subject, loop_size: LoopSize) -> Key {
    (0..loop_size).fold(1, |value, _| (value * subject) % 20201227)
}

fn determine_loop_size(key: Key, subject: Subject) -> Option<LoopSize> {
    let mut value= 1;
    for loop_size in 1..99999999 {
        value = (value * subject) % 20201227;
        if value == key {
            return Some(loop_size);
        }
    }
    None
}

fn part1(door_public_key: Key, card_public_key: Key) -> Option<Key> {
    let door_loop_size = determine_loop_size(door_public_key, 7).unwrap();
    let card_loop_size = determine_loop_size(card_public_key, 7).unwrap();
    let encryption_key = transform(door_public_key, card_loop_size);
    assert_eq!(encryption_key, transform(card_public_key, door_loop_size));
    Some(encryption_key)
}

fn main() {
    let door_public_key = 17773298;
    let card_public_key = 15530095;
    println!("part 1 {:?}", part1(door_public_key, card_public_key));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_loop_size() {
        assert_eq!(determine_loop_size(5764801, 7), Some(8));
        assert_eq!(determine_loop_size(17807724, 7), Some(11));
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(5764801, 17807724), Some(14897079));
    }
}
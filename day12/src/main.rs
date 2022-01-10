use derive_more::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Add, Sub)]
struct Vec2 {
    x: isize,
    y: isize,
}

impl Vec2 {
    // Vec2 is copy, so it's fine to take `self`
    fn manhattan(self) -> usize {
        (self.x.abs() + self.y.abs()) as _
    }
}

// we often move several units in some direction... so it'd be neat to multiply a Vec2 by an isize
impl std::ops::Mul<isize> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: isize) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

// Variant order chosen because trigonometry uses 0° as "east", facing east right turn ends south (clockwise)
// simplify Direction "adding" by explicitly defining our enum's representation, working with 0..=3
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Direction {
    East = 0,
    South = 1,
    West = 2,
    North = 3,
}

// We can easily convert a Direction to an isize, because _any_ Direction is always a valid isize
impl Into<isize> for Direction {
    fn into(self) -> isize {
        self as _
    }
}

// from isize to Direction is a fallible conversion (need to TryFrom trait)
impl std::convert::TryFrom<isize> for Direction {
    type Error = &'static str;

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        if (0..=3).contains(&value) {
            Ok(unsafe { std::mem::transmute(value as u8) })
        } else {
            Err("direction out of bounds!")
        }
    }
}


impl Direction {
    fn vec(self) -> Vec2 {
        match self {
            Direction::East => Vec2 { x: 1, y: 0 },
            Direction::South => Vec2 { x: 0, y: -1 },
            Direction::West => Vec2 { x: -1, y: 0 },
            Direction::North => Vec2 { x: 0, y: 1 },
        }
    }
}

/// Represents an angle, in multiples of 90°
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct AngleDelta(isize);

// if the angle is 90, then
// * If facing East, now facing South  * If facing South, now facing West
// * If facing West, now facing North  * If facing North, now facing East
// but the angle could also be 180, 270, 360, -90... lots of cases to deal with
impl std::ops::Add<AngleDelta> for Direction {
    type Output = Self;

    fn add(self, rhs: AngleDelta) -> Self::Output {
        use std::convert::TryInto;

        let angle: isize = self.into();
        (angle + rhs.0).rem_euclid(4).try_into().unwrap()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct ShipState {
    pos: Vec2,
    dir: Direction,
}

// really nice impl to leverage with fold... imagine we start with initial state,
// and keep applying modifications to it, from each instruction yielded by an iterator
impl std::ops::Add<Instruction> for ShipState {
    type Output = Self;

    fn add(self, rhs: Instruction) -> Self::Output {
        match rhs {
            Instruction::Move(dir, units) => Self {
                pos: self.pos + dir.vec() * units,
                ..self
            },
            Instruction::Rotate(delta) => Self {
                dir: self.dir + delta,
                ..self
            },
            Instruction::Advance(units) => Self {
                pos: self.pos + self.dir.vec() * units,
                ..self
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Instruction {
    /// Moves in given direction
    Move(Direction, isize),
    /// Turns
    Rotate(AngleDelta),
    /// Moves forward
    Advance(isize),
}

fn parse_instructions(input: &str) -> impl Iterator<Item = Instruction> + '_ {
    input.lines().map(|line| {
        let command = line.as_bytes()[0];
        // Safety: this will panic if `line` starts with multibyte character
        let number: isize = (&line[1..]).parse().unwrap();

        match command {
            b'N' => Instruction::Move(Direction::North, number),
            b'S' => Instruction::Move(Direction::South, number),
            b'E' => Instruction::Move(Direction::East, number),
            b'W' => Instruction::Move(Direction::West, number),
            b'L' => Instruction::Rotate(AngleDelta(-number / 90)),
            b'R' => Instruction::Rotate(AngleDelta(number / 90)),
            b'F' => Instruction::Advance(number),
            c => panic!("unknown instruction {}", c as char),
        }
    })
}

fn main() {
    for ins in parse_instructions(include_str!("input.txt")) {
        println!("{:?}", ins);
    }

    let start = ShipState {
        dir: Direction::East,
        pos: Vec2 { x: 0, y: 0 },
    };
    let end = parse_instructions(include_str!("input.txt")).fold(start, |state, ins| state + ins);

    dbg!(start, end, (end.pos - start.pos).manhattan());
}


#[test]
fn vec2_add() {
    let a = Vec2 { x: 3, y: 8 };
    let b = Vec2 { x: 2, y: 10 };
    assert_eq!(a + b, Vec2 { x: 5, y: 18 });
}

#[test]
fn manhattan_example() {
    let start = Vec2 { x: 0, y: 0 };
    let end = Vec2 { x: 17, y: -8 };
    assert_eq!((end - start).manhattan(), 25);
}

#[test]
fn direction_try_from() {
    use std::convert::TryFrom;

    assert_eq!(
        <Direction as TryFrom<isize>>::try_from(0).unwrap(),
        Direction::East
    );
    assert_eq!(
        <Direction as TryFrom<isize>>::try_from(2).unwrap(),
        Direction::West
    );
    assert!(<Direction as TryFrom<isize>>::try_from(-1).is_err(),);
    assert!(<Direction as TryFrom<isize>>::try_from(4).is_err(),);
}

#[test]
fn test_direction_add() {
    // From example
    assert_eq!(Direction::East + AngleDelta(1), Direction::South);
    // Turning "left" (counter-clockwise)
    assert_eq!(Direction::East + AngleDelta(-1), Direction::North);
    // Doing a 360°
    assert_eq!(Direction::East + AngleDelta(4), Direction::East);
}

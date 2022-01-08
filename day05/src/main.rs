use bitvec::prelude::*; // treat anything as a vector of... bits! exactly what we want to do here

#[derive(Default, Debug, PartialEq)]
struct Seat {
    row: u8,
    col: u8,
}

impl Seat {
    const ROW_BITS: usize = 7;
    const COL_BITS: usize = 3;

    fn id(&self) -> u64 {
        // bit shifting to multiply by the row number by 8
        ((self.row as u64) << Self::COL_BITS) + (self.col as u64)
    }

    fn parse(input: &str) -> Self {
        let bytes = input.as_bytes();
        let mut res: Seat = Default::default();

        {
            // treat `res.row` as a collection of bits...
            let row = BitSlice::<Msb0, _>::from_element_mut(&mut res.row);
            // for each `F` or `B` element...
            for (i, &b) in bytes[0..Self::ROW_BITS].iter().enumerate() {
                // set the corresponding bit, in positions 1 through 7 (0-indexed)
                row.set(
                    (8 - Self::ROW_BITS) + i,
                    match b {
                        b'F' => false,
                        b'B' => true,
                        _ => panic!("unexpected row letter: {}", b as char),
                    },
                );
            }
        }

        {
            let col = BitSlice::<Msb0, _>::from_element_mut(&mut res.col);
            for (i, &b) in bytes[Self::ROW_BITS..][..Self::COL_BITS].iter().enumerate() {
                col.set(
                    (8 - Self::COL_BITS) + i,
                    match b {
                        b'L' => false,
                        b'R' => true,
                        _ => panic!("unexpected col letter: {}", b as char),
                    },
                );
            }
        }

        res
    }
}

// derive Ord to indicate that our type (more or less still a u16) has total ordering
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Seat2(u16);

impl Seat2 {
    // simplify Seat type to a u16 (its u10, 7bits row 3bits column) parse all ten bits in one go
    // reverse the iterator and use Lsb0 (least-significant bit first) order, no need for arithmetic
    fn parse(input: &str) -> Self {
        let mut res: Seat2 = Default::default();

        let bits = BitSlice::<Lsb0, _>::from_element_mut(&mut res.0);
        for (i, &b) in input.as_bytes().iter().rev().enumerate() {
            bits.set(
                i,
                match b {
                    b'F' | b'L' => false,
                    b'B' | b'R' => true,
                    _ => panic!("unexpected letter: {}", b as char),
                },
            )
        }

        res
    }
}

fn main() {
    let max_id = itertools::max(
        include_str!("input.txt")
            .lines()
            .map(Seat::parse)
            .map(|seat| seat.id()),
    );
    println!("Part 1:");
    println!("  The maximum seat ID is {:?}", max_id);

    let max_id = itertools::max(
        include_str!("input.txt")
            .lines()
            .map(Seat2::parse)
            .map(|seat| seat.0),
    );
    println!("  The maximum seat ID is {:?}", max_id);

    // part 2 wants missing seat
    // collect all the IDs, sort them (from smallest to largest), then iterate, keeping track
    // of the last one, and whenever the gap is more than 1 - that's it! We've found our seat.
    // for our first iteration, we won't have a "last id", so we'll just use an Option
    let mut ids: Vec<_> = include_str!("input.txt").lines().map(Seat2::parse).collect();
    ids.sort();

    let mut last_id: Option<Seat2> = None;
    for id in ids {
        if let Some(last_id) = last_id {
            let gap = id.0 - last_id.0;
            if gap > 1 {
                println!("Our seat ID is {}", last_id.0 + 1);
                return;
            }
        }
        last_id = Some(id);
    }
}


#[test]
fn test_parse() {
    let input = "FBFBBFFRLR";
    let seat = Seat::parse(input);
    assert_eq!(seat, Seat { row: 44, col: 5 });
}

#[test]
fn test_seat_id() {
    macro_rules! validate {
        ($input: expr, $row: expr, $col: expr, $id: expr) => {
            let seat = Seat::parse($input);
            assert_eq!(
                seat,
                Seat {
                    row: $row,
                    col: $col
                }
            );
            assert_eq!(seat.id(), $id);
        };
    }

    validate!("BFFFBBFRRR", 70, 7, 567);
    validate!("FFFBBBFRRR", 14, 7, 119);
    validate!("BBFFBBFRLL", 102, 4, 820);
}

#[test]
fn test_seat2_id() {
    assert_eq!(Seat2::parse("BFFFBBFRRR"), Seat2(567));
    assert_eq!(Seat2::parse("FFFBBBFRRR"), Seat2(119));
    assert_eq!(Seat2::parse("BBFFBBFRLL"), Seat2(820));
}

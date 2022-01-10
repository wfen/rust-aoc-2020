use im::Vector;
use itertools::Itertools;
use std::fmt;
use std::iter::Extend;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 {
    x: i64,
    y: i64,
}

#[derive(Clone, Copy, PartialEq)]
enum Tile {
    Floor,
    EmptySeat,
    OccupiedSeat,
}

impl Default for Tile {
    fn default() -> Self {
        Self::Floor
    }
}

impl fmt::Debug for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Tile::Floor => '.',
            Tile::EmptySeat => 'L',
            Tile::OccupiedSeat => '#',
        };
        write!(f, "{}", c)
    }
}

impl Tile {
    fn next1<I>(self, neighbors: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        match self {
            Self::Floor => Self::Floor,
            Self::EmptySeat => match neighbors
                .filter(|t| matches!(t, Self::OccupiedSeat))
                .count() {
                // no one around? we can sit here!
                0 => Self::OccupiedSeat,
                // social distancing please
                _ => Self::EmptySeat,
            },
            Self::OccupiedSeat => {
                match neighbors
                    .filter(|t| matches!(t, Self::OccupiedSeat))
                    .count() {
                    // up to 3 neighbors: still ok for now
                    0..=3 => Self::OccupiedSeat,
                    // that's too many folks!
                    _ => Self::EmptySeat,
                }
            }
        }
    }

    fn next2<I>(self, neighbors: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        match self {
            Self::Floor => Self::Floor,
            Self::EmptySeat => match neighbors
                .filter(|t| matches!(t, Self::OccupiedSeat))
                .count() {
                // no one around? we can sit here!
                0 => Self::OccupiedSeat,
                // social distancing please
                _ => Self::EmptySeat,
            },
            Self::OccupiedSeat => {
                match neighbors
                    .filter(|t| matches!(t, Self::OccupiedSeat))
                    .count() {
                    // 👇 new!
                    // up to 4 neighbors: still okay for now
                    0..=4 => Self::OccupiedSeat,
                    // that's too many folks!
                    _ => Self::EmptySeat,
                }
            }
        }
    }
}

#[derive(Debug)]
struct Positioned<T>(Vec2, T);

// Note: Vec2 already derives PartialEq. As for T, it might or it might not.
// Map<T> will only implement PartialEq if T itself implements PartialEq.

#[derive(PartialEq, Clone)]
struct Map<T>
where
    T: Clone,
{
    size: Vec2,
    tiles: Vector<T>,
}

impl<T> fmt::Debug for Map<T>
where
    T: fmt::Debug + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.size.y {
            for x in 0..self.size.x {
                write!(f, "{:?}", self.get(Vec2 { x, y }).unwrap())?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl<A> Extend<Positioned<A>> for Map<A>
where
    A: Clone,
{
    fn extend<T: IntoIterator<Item = Positioned<A>>>(&mut self, iter: T) {
        for Positioned(pos, tile) in iter {
            self.set(pos, tile)
        }
    }
}

impl<T> Map<T>
where
    T: Default + Clone,
{
    fn new(size: Vec2) -> Self {
        let num_tiles = size.x * size.y;
        Self {
            size,
            tiles: (0..num_tiles)
                .into_iter()
                .map(|_| Default::default())
                .collect(),
        }
    }
}
impl<T> Map<T>
where
    T: Clone,
{
    fn index(&self, pos: Vec2) -> Option<usize> {
        if (0..self.size.x).contains(&pos.x) && (0..self.size.y).contains(&pos.y) {
            Some((pos.x + pos.y * self.size.x) as _)
        } else {
            None
        }
    }

    fn set(&mut self, pos: Vec2, tile: T) {
        if let Some(index) = self.index(pos) {
            self.tiles[index] = tile;
        }
    }

    fn neighbor_positions(&self, pos: Vec2) -> impl Iterator<Item = Vec2> {
        (-1..=1)
            .flat_map(|dx| (-1..=1).map(move |dy| (dx, dy)))
            .filter(|&(dx, dy)| !(dx == 0 && dy == 0))
            .map(move |(dx, dy)| Vec2 {
                x: pos.x + dx,
                y: pos.y + dy,
            })
    }
}

impl<T> Map<T>
where
    T: Copy,
{
    fn get(&self, pos: Vec2) -> Option<T> {
        self.index(pos).map(|index| self.tiles[index])
    }

    // regarding '_: this iterator is only valid as long as &self is borrowed, because it's reading
    // from it. Default lifetime for impl Iterator<Item = T> of 'static is only true for owned types.
    fn neighbor_tiles(&self, pos: Vec2) -> impl Iterator<Item = T> + '_ {
        self.neighbor_positions(pos)
            .filter_map(move |pos| self.get(pos))
    }

    fn iter(&self) -> impl Iterator<Item = Positioned<T>> + '_ {
        (0..self.size.y).flat_map(move |y| {
            (0..self.size.x).map(move |x| {
                let pos = Vec2 { x, y };
                Positioned(pos, self.get(pos).unwrap())
            })
        })
    }
}

impl Map<Tile>
where
    Tile: Clone,
{
    fn parse(input: &[u8]) -> Self {
        let mut columns = 0;
        let mut rows = 1;
        for &c in input.iter() {
            if c == b'\n' {
                rows += 1;
                columns = 0;
            } else {
                columns += 1;
            }
        }

        let mut iter = input.iter().copied();
        let mut map = Self::new(Vec2 { x: columns, y: rows });
        for row in 0..map.size.y {
            for col in 0..map.size.x {
                let tile = match iter.next() {
                    Some(b'.') => Tile::Floor,
                    Some(b'L') => Tile::EmptySeat,
                    Some(b'#') => Tile::OccupiedSeat,
                    c => panic!("Expected '.', 'L' or '#', but got: {:?}", c),
                };
                map.set(Vec2 { x: col, y: row }, tile);
            }
            iter.next();
        }
        map
    }

    fn next1(&self) -> Self {
        let mut res = Self::new(self.size);
        res.extend(
            self.iter()
                .map(|Positioned(pos, tile)| Positioned(pos, tile.next1(self.neighbor_tiles(pos)))),
        );
        res
    }

    fn last1(self) -> Self {
        itertools::iterate(self, Map::next1)
            .tuple_windows()
            .find_map(|(prev, next)| if prev == next { Some(next) } else { None })
            .unwrap()
    }

    fn next2(&self) -> Self {
        let mut res = Self::new(self.size);
        res.extend(
            self.iter()
                //                                                                                       👇👇👇
                .map(|Positioned(pos, tile)| Positioned(pos, tile.next2(self.visible_seats(pos)))),
        );
        res
    }

    fn last2(self) -> Self {
        itertools::iterate(self, Map::next2)
            .tuple_windows()
            .find_map(|(prev, next)| if prev == next { Some(next) } else { None })
            .unwrap()
    }

    fn visible_seats(&self, pos: Vec2) -> impl Iterator<Item = Tile> + '_ {
        (-1..=1)
            .flat_map(|dx| (-1..=1).map(move |dy| (dx, dy)))
            .filter(|&(dx, dy)| !(dx == 0 && dy == 0))
            .flat_map(move |(dx, dy)| {
                // keep moving in set direction
                itertools::iterate(pos, move |v| Vec2 {
                    x: v.x + dx,
                    y: v.y + dy,
                })
                // required to get the initial value right for our call to itertools::iterate
                .skip(1)
                // as long as we're on the map
                .map(move |pos| self.index(pos))
                .while_some()
                // and until we reach a seat
                .filter_map(move |index| match self.tiles[index] {
                    Tile::Floor => None,
                    seat => Some(seat),
                })
                .take(1)
            })
    }
}

fn main() {
    /*
    let mut m = Map::new(Vec2 { x: 3, y: 3 });
    m.set(Vec2 { x: 1, y: 1 }, Tile::OccupiedSeat);

    for tile in m.iter() {
        println!("{:?}", tile);
    }
    */

    /*
    let m = Map::<Tile>::parse(include_bytes!("input.txt"));
    dbg!(&m.size);
    println!("{:?}", m);
    */

    /*
    let maps = itertools::iterate(Map::<Tile>::parse(include_bytes!("input.txt")), Map::next);
    for map in maps.take(5) {
        println!("{:?}", map);
    }
    */

    /*
    let last = Map::<Tile>::parse(include_bytes!("input.txt")).last1();
    println!("{:?}", last);
    */

    let last = Map::<Tile>::parse(include_bytes!("input.txt")).last1();
    //println!("{:?}", last);
    println!("Part1:");
    println!(
        "  there are {} occupied seats",
        last.iter()
            //      👇  this is a Positioned<Tile>
            .filter(|p| matches!(p.1, Tile::OccupiedSeat))
            .count()
    );


    let last2 = Map::<Tile>::parse(include_bytes!("input.txt")).last2();
    //println!("{:?}", last2);
    println!("Part2:");
    println!(
        "  there are {} occupied seats",
        last2
            .iter()
            //      👇  this is a Positioned<Tile>
            .filter(|p| matches!(p.1, Tile::OccupiedSeat))
            .count()
    );
}

#[test]
fn test_neighbor_positions() {
    use std::collections::HashSet;

    let map = Map::<()>::new(Vec2 { x: 3, y: 3 });
    let positions: HashSet<_> = map
        .neighbor_positions(Vec2 { x: 1, y: 1 })
        .map(|v| (v.x, v.y))
        .collect();
    for p in &[(0, 0), (0, 1), (0, 2), (1, 0), (2, 0), (1, 2), (2, 2), (2, 1)] {
        assert!(positions.contains(p));
    }
}

#[test]
fn test_visible_seats() {
    let map = Map::<Tile>::parse(
        indoc::indoc!(
            "
            .......#.
            ...#.....
            .#.......
            .........
            ..#L....#
            ....#....
            .........
            #........
            ...#.....
            "
        )
        .trim()
        .as_bytes(),
    );
    println!("{:?}", map);
    assert_eq!(map.visible_seats(Vec2 { x: 3, y: 4 }).count(), 8);
    assert_eq!(map.visible_seats(Vec2 { x: 8, y: 0 }).count(), 2);
}

#[test]
fn test_visible_seats2() {
    let map = Map::<Tile>::parse(
        indoc::indoc!(
            "
            .##.##.
            #.#.#.#
            ##...##
            ...L...
            ##...##
            #.#.#.#
            .##.##.
            "
        )
        .trim()
        .as_bytes(),
    );

    assert_eq!(map.visible_seats(Vec2 { x: 3, y: 3 }).count(), 0);
}

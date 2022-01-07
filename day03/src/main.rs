use std::fmt;
use std::ops::AddAssign;

// Vec2 will be used to represent positions on the map
#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 {
    x: i64,
    y: i64,
}

impl From<(i64, i64)> for Vec2 {
    fn from((x, y): (i64, i64)) -> Self {
        Self { x, y }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

// Tile will represent what's _in_ a tile.
#[derive(Clone, Copy, PartialEq)]
enum Tile {
    Open,
    Tree,
}

impl Default for Tile {
    fn default() -> Self {
        Self::Open
    }
}

impl fmt::Debug for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Tile::Open => '.',
            Tile::Tree => '#',
        };
        write!(f, "{}", c)
    }
}

struct Map {
    size: Vec2,
    tiles: Vec<Tile>,
}

// We're storing all tiles in a flat array, in row-major order, which means we're
// storing all tiles from the top row first, then we move on to the second row, etc.
impl Map {
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

    /// normalize_pos() wraps the x coordinate so the map extends infinitely to the left and right.
    /// Map has finite height. Returns `None` for coordinates above 0 or below `self.size.y
    fn normalize_pos(&self, pos: Vec2) -> Option<Vec2> {
        if pos.y < 0 || pos.y >= self.size.y {
            None
        } else {
            let x = pos.x % self.size.x;
            // wrap around for left side (negative X coordinates)
            let x = if x < 0 { self.size.x + x } else { x };
            Some((x, pos.y).into())
        }
    }

    // index() returns the index of a tile in our flat storage
    // None is returned for positions that do not exist on the map (above or below it)
    fn index(&self, pos: Vec2) -> Option<usize> {
        self.normalize_pos(pos)
            .map(|pos| (pos.x + pos.y * self.size.x) as _)
    }

    // get() gives back the Tile for a given pos. We simplify get() by returning a Tile
    // instead of Option<Tile>. Tiles outside the map are open.
    fn get(&self, pos: Vec2) -> Tile {
        self.index(pos).map(|i| self.tiles[i]).unwrap_or_default()
    }

    // set() allows us to assign a Tile value to a particular pos. We simplify set() by
    // assuming that every tile outside the map is immutable.
    fn set(&mut self, pos: Vec2, tile: Tile) {
        // you can think of if let as syntax sugar for a match that runs code
        // when the value matches one pattern and then ignores other values
        if let Some(index) = self.index(pos) {
            self.tiles[index] = tile
        }
    }

    // input comes from include_bytes! working with input.txt
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
        let mut map = Self::new((columns, rows).into());
        for row in 0..map.size.y {
            for col in 0..map.size.x {
                let tile = match iter.next() {
                    Some(b'.') => Tile::Open,
                    Some(b'#') => Tile::Tree,
                    c => panic!("Expected '.' or '#', but got: {:?}", c),
                };
                map.set((col, row).into(), tile);
            }
            iter.next();
        }
        map
    }
}

impl fmt::Debug for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..self.size.y {
            for col in 0..self.size.x {
                write!(f, "{:?}", self.get((col, row).into()))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}


fn main() -> anyhow::Result<()> {
    /*
    // let's build a simple map and check our Debug implementation
    let map = {
        let mut m = Map::new((6, 6).into());
        let points = [(1, 1), (4, 1), (1, 3), (4, 3), (2, 4), (3, 4)];
        for p in (&points).iter().copied() {
            m.set(p.into(), Tile::Tree);
        }
        m
    };
    println!("{:?}", map);
    */

    let map = Map::parse(include_bytes!("input.txt"));
    let itinerary = (0..map.size.y).into_iter().map(|y| Vec2::from((y * 3, y)));
    let num_trees = itinerary.filter(|&pos| map.get(pos) == Tile::Tree).count();
    println!("Part 1:");
    println!("  We encountered {} trees", num_trees);

    // Part 2 asks that we explore a collection of different moving patterns.
    // Right 1, down 1    Right 3, down 1   Right 5, down 1    Right 7, down 1    Right 1, down 2
    let deltas: &[Vec2] = &[
        (1, 1).into(),
        (3, 1).into(),
        (5, 1).into(),
        (7, 1).into(),
        (1, 2).into(),
    ];
    let answer = deltas
        .iter()
        .copied()
        // generate all itineraries
        .map(|delta| generate_itinerary(&map, delta))
        // count trees
        .map(|itin| {
            itin.into_iter()
                .filter(|&pos| map.get(pos) == Tile::Tree)
                .count()
        })
        // multiply everything together
        .product::<usize>();

    println!("Part 2:");
    println!("  The answer is {}", answer);

    Ok(())
}

/// generate_itinerary() produces a list of positions from a given moving pattern.
/// A borrowed &Map allows us to stop once we've exceeded the map's bounds
fn generate_itinerary(map: &Map, delta: Vec2) -> Vec<Vec2> {
    let mut pos = Vec2::from((0,0));
    let mut res: Vec<_> = Default::default();

    while map.normalize_pos(pos).is_some() {
        res.push(pos);
        pos += delta;
    }
    res
}

#[test]
fn test_tuple() {
    let v: Vec2 = (5, 8).into();
    assert_eq!(v.x, 5);
    assert_eq!(v.y, 8);
}

#[test]
fn test_normalize_pos() {
    let m = Map::new((2, 2).into());
    assert_eq!(m.normalize_pos((0, 0).into()), Some((0, 0).into()));
    assert_eq!(m.normalize_pos((1, 0).into()), Some((1, 0).into()));
    assert_eq!(m.normalize_pos((2, 0).into()), Some((0, 0).into()));
    assert_eq!(m.normalize_pos((-1, 0).into()), Some((1, 0).into()));
    assert_eq!(m.normalize_pos((-2, 0).into()), Some((0, 0).into()));
    assert_eq!(m.normalize_pos((0, -1).into()), None);
    assert_eq!(m.normalize_pos((0, 2).into()), None);
}

#[test]
fn test_index() {
    let m = Map::new((3, 5).into());
    assert_eq!(m.index((0, 0).into()), Some(0));
    assert_eq!(m.index((2, 0).into()), Some(2));
    assert_eq!(m.index((0, 1).into()), Some(3));
    assert_eq!(m.index((2, 1).into()), Some(5));
}

#[test]
fn test_generate_itinerary() {
    assert_eq!(
        &generate_itinerary(&Map::new((5, 5).into()), (1, 1).into()),
        &[
            (0, 0).into(),
            (1, 1).into(),
            (2, 2).into(),
            (3, 3).into(),
            (4, 4).into(),
        ],
        "right 1 down 1, 5x5 map"
    );

    assert_eq!(
        &generate_itinerary(&Map::new((5, 5).into()), (3, 1).into()),
        &[
            (0, 0).into(),
            (3, 1).into(),
            (6, 2).into(),
            (9, 3).into(),
            (12, 4).into(),
        ],
        "right 3 down 1, 5x5 map"
    );

    assert_eq!(
        &generate_itinerary(&Map::new((5, 5).into()), (2, 2).into()),
        &[(0, 0).into(), (2, 2).into(), (4, 4).into(),],
        "right 2 down 2, 5x5 map"
    );

    assert_eq!(
        &generate_itinerary(&Map::new((9, 9).into()), (2, 5).into()),
        &[(0, 0).into(), (2, 5).into(),],
        "right 2 down 5, 9x9 map"
    );
}

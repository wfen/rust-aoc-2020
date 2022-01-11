use std::collections::{HashMap, HashSet};
use std::ops::Add;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use parser::*;

// -- model

#[derive(Debug, PartialEq, Copy, Clone, EnumIter)]
enum Direction {
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest
}

type Path = Vec<Direction>;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct HexTile {
    x: i64,
    y: i64,
    z: i64
}

impl HexTile {
    fn new(x: i64, y: i64, z: i64) -> Self {
        assert!(x + y + z == 0);
        HexTile { x, y, z }
    }

    fn from_path(path: &Path) -> Self {
        path.iter().fold(HexTile::new(0, 0, 0), |hex, step| hex + step)
    }

    fn neighbours(&self) -> impl Iterator<Item = HexTile> + '_ {
        Direction::iter().map(move |dir| *self + &dir)
    }
}

impl Add<&Direction> for HexTile {
    type Output = HexTile;

    fn add(self, dir: &Direction) -> Self {
        use Direction::*;
        match dir {
            East      => HexTile::new(self.x + 1, self.y - 1, self.z),
            West      => HexTile::new(self.x - 1, self.y + 1, self.z),
            NorthEast => HexTile::new(self.x + 1, self.y,     self.z - 1),
            NorthWest => HexTile::new(self.x,     self.y + 1, self.z - 1),
            SouthEast => HexTile::new(self.x,     self.y - 1, self.z + 1),
            SouthWest => HexTile::new(self.x - 1, self.y,     self.z + 1)
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Color {
    White, Black
}

impl Color {
    fn flip(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White
        }
    }
}

#[derive(Debug, Clone)]
struct Grid {
    tiles: HashMap<HexTile, Color>
}

impl Grid {
    fn new() -> Self {
        Grid { tiles: HashMap::new() }
    }

    fn at(&self, tile: &HexTile) -> Color {
        match self.tiles.get(tile) {
            None => Color::White,
            Some(c) => *c
        }
    }

    fn flip(&mut self, tile: &HexTile) {
        match self.tiles.get_mut(tile) {
            None => {
                self.tiles.insert(*tile, Color::Black);
            }
            Some(c) => {
                *c = c.flip();
            }
        }
    }

    fn count(&self, c: Color) -> usize {
        self.tiles.values().filter(|t| *t == &c).count()
    }

    fn all_tiles_with_margin(&self) -> HashSet<HexTile> {
        let mut all = HashSet::new();
        for tile in self.tiles.keys() {
            all.insert(*tile);
            for n in tile.neighbours() {
                all.insert(n);
            }
        }
        all
    }

    fn next_generation(&self) -> Grid {
        let mut next = Grid::new();
        for tile in self.all_tiles_with_margin().iter() {
            let black = tile.neighbours().filter(|t| self.at(&t) == Color::Black).count();
            let next_c = match self.at(&tile) {
                Color::Black =>
                    if black == 0 || black > 2 { Color::White } else { Color::Black },
                Color::White =>
                    if black == 2 { Color::Black } else { Color::White }
            };
            next.tiles.insert(*tile, next_c);
        }
        next
    }

    fn run_n_generations(&self, n: usize) -> Grid {
        (0..n).fold(self.clone(), |grid, _| grid.next_generation())
    }

}

// -- parser

fn parse_paths(input: &str) -> ParseResult<Vec<Path>> {
    let east = match_literal("e").means(Direction::East);
    let west = match_literal("w").means(Direction::West);
    let north_east = match_literal("ne").means(Direction::NorthEast);
    let north_west = match_literal("nw").means(Direction::NorthWest);
    let south_east = match_literal("se").means(Direction::SouthEast);
    let south_west = match_literal("sw").means(Direction::SouthWest);
    let step = east.or(west).or(north_east).or(north_west).or(south_east).or(south_west);
    let path = one_or_more(step);
    let paths = one_or_more(whitespace_wrap(path));
    paths.parse(input)
}

// -- problems

fn grid_from_paths(paths: &Vec<Path>) -> Grid {
    let mut grid = Grid::new();
    for path in paths {
        grid.flip(&HexTile::from_path(&path));
    }
    grid
}

fn part1(grid: &Grid) -> usize {
    grid.count(Color::Black)
}

fn part2(grid: &Grid) -> usize {
    let final_grid = grid.run_n_generations(100);
    final_grid.count(Color::Black)
}


fn main() {
    // hexagonal grids https://www.redblobgames.com/grids/hexagons/

    let input = include_str!("input.txt");
    let paths = parse_paths(&input).unwrap().1;
    let grid = grid_from_paths(&paths);
    println!("part 1 {}", part1(&grid));
    println!("part 2 {}", part2(&grid));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_paths() -> Vec<Path> {
        parse_paths(
            "sesenwnenenewseeswwswswwnenewsewsw
            neeenesenwnwwswnenewnwwsewnenwseswesw
            seswneswswsenwwnwse
            nwnwneseeswswnenewneswwnewseswneseene
            swweswneswnenwsewnwneneseenw
            eesenwseswswnenwswnwnwsewwnwsene
            sewnenenenesenwsewnenwwwse
            wenwwweseeeweswwwnwwe
            wsweesenenewnwwnwsenewsenwwsesesenwne
            neeswseenwwswnwswswnw
            nenwswwsewswnenenewsenwsenwnesesenew
            enewnwewneswsewnwswenweswnenwsenwsw
            sweneswneswneneenwnewenewwneswswnese
            swwesenesewenwneswnwwneseswwne
            enesenwswwswneneswsenwnewswseenwsese
            wnwnesenesenenwwnenwsewesewsesesew
            nenewswnwewswnenesenwnesewesw
            eneswnwswnwsenenwnwnwwseeswneewsenese
            neswnwewnwnwseenwseesewsenwsweewe
            wseweeenwnesenwwwswnew"
        ).unwrap().1
    }

    fn test_grid() -> Grid {
        grid_from_paths(&test_paths())
    }

    #[test]
    fn test_parser() {
        use Direction::*;
        let paths = parse_paths("esew\nnwwswee");
        assert_eq!(paths, Ok(("", vec![
            vec![East, SouthEast, West],
            vec![NorthWest, West, SouthWest, East, East]
        ])));
    }

    #[test]
    fn test_hextile_from_path() {
        use Direction::*;
        assert_eq!(HexTile::from_path(&vec![East, SouthWest, West]), HexTile::new(-1, 0, 1));
        assert_eq!(HexTile::from_path(&vec![NorthWest, West, SouthWest, East, East]), HexTile::new(0, 0, 0));
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(&test_grid()), 10);
    }

    #[test]
    fn test_part2() {
        use Color::Black;
        let grid = test_grid();
        assert_eq!(grid.run_n_generations(1).count(Black), 15);
        assert_eq!(grid.run_n_generations(2).count(Black), 12);
        assert_eq!(grid.run_n_generations(3).count(Black), 25);
        assert_eq!(grid.run_n_generations(4).count(Black), 14);
        assert_eq!(grid.run_n_generations(20).count(Black), 132);
        assert_eq!(grid.run_n_generations(30).count(Black), 259);
        assert_eq!(grid.run_n_generations(100).count(Black), 2208);
    }
}

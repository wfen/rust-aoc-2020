#[macro_use]
extern crate lazy_static;

use log::{debug, info};
use std::collections::{HashMap, HashSet};
use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use parser::*;

// --- model

type TileID = usize;
type EdgePattern = u64;

trait Reversible {
    fn reversed(self) -> Self;
}

lazy_static! {
    static ref REVERSE_EDGE_PATTERNS: Vec<EdgePattern> = {
        let mut reversed = Vec::with_capacity(1 << 10);
        for i in 0..(1<<10) {
            let mut rev = 0;
            for bit in 0..10 {
                if i & (1 << bit) != 0 {
                    rev |= 0x200 >> bit;
                }
            }
            reversed.push(rev);
        }
        reversed
    };
}

impl Reversible for EdgePattern {
    fn reversed(self) -> Self {
        REVERSE_EDGE_PATTERNS[self as usize]
    }
}

#[derive(Debug)]
struct Tile {
    id: TileID,
    top: EdgePattern,
    left: EdgePattern,
    right: EdgePattern,
    bottom: EdgePattern,
    content: Vec<Vec<char>>
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, EnumIter, Hash)]
enum Orientation {
    R0,
    R90,
    R180,
    R270,
    R0FlipH,
    R0FlipV,
    R90FlipH,
    R90FlipV
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
struct OrientedTile {
    tile_id: TileID,
    orientation: Orientation
}

impl Eq for Tile {}

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Tile {
    fn top_edge_in_orientation(&self, orientation: Orientation) -> EdgePattern {
        match orientation {
            Orientation::R0 => self.top,
            Orientation::R90 => self.right,
            Orientation::R180 => self.bottom.reversed(),
            Orientation::R270  => self.left.reversed(),
            Orientation::R0FlipH => self.top.reversed(),
            Orientation::R0FlipV => self.bottom,
            Orientation::R90FlipH => self.right.reversed(),
            Orientation::R90FlipV => self.left
        }
    }

    fn bottom_edge_in_orientation(&self, orientation: Orientation) -> EdgePattern {
        match orientation {
            Orientation::R0 => self.bottom,
            Orientation::R90 => self.left,
            Orientation::R180 => self.top.reversed(),
            Orientation::R270  => self.right.reversed(),
            Orientation::R0FlipH => self.bottom.reversed(),
            Orientation::R0FlipV => self.top,
            Orientation::R90FlipH => self.left.reversed(),
            Orientation::R90FlipV => self.right
        }
    }

    fn left_edge_in_orientation(&self, orientation: Orientation) -> EdgePattern {
        match orientation {
            Orientation::R0 => self.left,
            Orientation::R90 => self.top.reversed(),
            Orientation::R180 => self.right.reversed(),
            Orientation::R270  => self.bottom,
            Orientation::R0FlipH => self.right,
            Orientation::R0FlipV => self.left.reversed(),
            Orientation::R90FlipH => self.bottom.reversed(),
            Orientation::R90FlipV =>self.top
        }
    }

    fn right_edge_in_orientation(&self, orientation: Orientation) -> EdgePattern {
        match orientation {
            Orientation::R0 => self.right,
            Orientation::R90 => self.bottom.reversed(),
            Orientation::R180 => self.left.reversed(),
            Orientation::R270  => self.top,
            Orientation::R0FlipH => self.left,
            Orientation::R0FlipV => self.right.reversed(),
            Orientation::R90FlipH => self.top.reversed(),
            Orientation::R90FlipV => self.bottom
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
enum Relationship {
    Above,
    Below,
    LeftOf,
    RightOf
}

#[derive(Debug)]
struct AllowedOrientedTiles {
    neighbours: HashMap<(TileID, Orientation, Relationship), HashSet<OrientedTile>>,
    empty: HashSet<OrientedTile>
}

impl AllowedOrientedTiles {
    fn new(tiles: &Vec<&Tile>) -> Self {
        let mut allowed = HashMap::new();
        for tile in tiles.iter() {
            for orientation in Orientation::iter() {
                let mut above = HashSet::new();
                let mut below = HashSet::new();
                let mut left_of = HashSet::new();
                let mut right_of = HashSet::new();

                for candidate in tiles.iter().filter(|t| t.id != tile.id) {
                    for candidate_orientation in Orientation::iter() {
                        if candidate.bottom_edge_in_orientation(candidate_orientation) == tile.top_edge_in_orientation(orientation) {
                            above.insert(OrientedTile { tile_id: candidate.id, orientation: candidate_orientation });
                        }
                        if candidate.top_edge_in_orientation(candidate_orientation) == tile.bottom_edge_in_orientation(orientation) {
                            below.insert(OrientedTile { tile_id: candidate.id, orientation: candidate_orientation });
                        }
                        if candidate.left_edge_in_orientation(candidate_orientation) == tile.right_edge_in_orientation(orientation) {
                            right_of.insert(OrientedTile { tile_id: candidate.id, orientation: candidate_orientation });
                        }
                        if candidate.right_edge_in_orientation(candidate_orientation) == tile.left_edge_in_orientation(orientation) {
                            left_of.insert(OrientedTile { tile_id: candidate.id, orientation: candidate_orientation });
                        }
                    }
                }

                allowed.insert((tile.id, orientation, Relationship::Above), above);
                allowed.insert((tile.id, orientation, Relationship::Below), below);
                allowed.insert((tile.id, orientation, Relationship::LeftOf), left_of);
                allowed.insert((tile.id, orientation, Relationship::RightOf), right_of);
            }
        }

        AllowedOrientedTiles {
            neighbours: allowed,
            empty: HashSet::new()
        }
    }

    fn get(&self, tile_id: TileID, orientation: Orientation, relationship: Relationship) -> &'_ HashSet<OrientedTile> {
        self.neighbours.get(&(tile_id, orientation, relationship)).unwrap_or(&self.empty)
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
struct Pos {
    x: i64,
    y: i64
}

impl Pos {
    fn up(&self) -> Pos {
        Pos { x: self.x, y: self.y - 1 }
    }

    fn down(&self) -> Pos {
        Pos { x: self.x, y: self. y + 1 }
    }

    fn left(&self) -> Pos {
        Pos { x: self.x - 1, y: self.y }
    }

    fn right(&self) -> Pos {
        Pos { x: self.x + 1, y: self.y }
    }

    fn neighbours(&self) -> impl Iterator<Item=Pos> + '_ {
        let mut i = 0;
        std::iter::from_fn(move || {
            let n = i;
            i += 1;
            match n {
                0 => Some(self.up()),
                1 => Some(self.down()),
                2 => Some(self.left()),
                3 => Some(self.right()),
                _ => None
            }
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
enum TilePlacement<'a> {
    None,
    Placed {
        orientation: Orientation,
        tile: &'a Tile
    }
}

impl<'a> Default for TilePlacement<'a> {
    fn default() -> Self { TilePlacement::None }
}

struct OrientedTileSet {
    unrestricted: bool,
    oriented_tiles: HashSet<OrientedTile>
}

impl OrientedTileSet {
    fn new() -> Self {
        OrientedTileSet {
            unrestricted: true,
            oriented_tiles: HashSet::new()
        }
    }

    fn restrict_to(&mut self, neighbours: &HashSet<OrientedTile>) {
        if !self.unrestricted {
            self.oriented_tiles = self.oriented_tiles.intersection(neighbours).cloned().collect();
        } else {
            self.oriented_tiles = neighbours.clone();
            self.unrestricted = false;
        }
    }

    fn is_empty(&self) -> bool {
        !self.unrestricted && self.oriented_tiles.is_empty()
    }
}

struct Arrangement<'a> {
    width: i64,
    height: i64,
    fixed_tiles: [[TilePlacement<'a>; 12]; 12],
    available_tiles: HashMap<TileID, &'a Tile>,
    next_positions: HashSet<Pos>
}


impl<'a> Arrangement<'a> {
    fn new(width: i64, height: i64, tiles: &[&'a Tile]) -> Self {
        Arrangement {
            width,
            height,
            fixed_tiles: Default::default(),
            available_tiles: tiles.iter().map(|tile| (tile.id, *tile)).collect(),
            next_positions: HashSet::new()
        }
    }

    fn place(&mut self, pos: &Pos, orientation: Orientation, tile_id: TileID) {
        if let Some(tile) = self.available_tiles.remove(&tile_id) {
            self.fixed_tiles[pos.y as usize][pos.x as usize] = TilePlacement::Placed { orientation, tile };
            self.next_positions.remove(pos);
            for n in pos.neighbours() {
                if self.valid(&n) && self.tile_at(&n) == &TilePlacement::None {
                    self.next_positions.insert(n);
                }
            }
            debug!("place {} {:?} at {:?}", tile_id, orientation, pos);
            debug!("{:?}", self);
        } else {
            panic!("trying to place unavailable tile");
        }
    }

    fn remove(&mut self, pos: &Pos) {
        match self.fixed_tiles[pos.y as usize][pos.x as usize] {
            TilePlacement::None => {},
            TilePlacement::Placed { orientation: _, tile } => {
                self.available_tiles.insert(tile.id, tile);
                self.fixed_tiles[pos.y as usize][pos.x as usize] = TilePlacement::None;
                self.next_positions.insert(*pos);
                debug!("remove {} from {:?}", tile.id, pos);
                debug!("{:?}", self);
            }
        }
    }

    fn valid(&self, pos: &Pos) -> bool {
        0 <= pos.x && 0 <= pos.y && pos.x < self.width && pos.y < self.height
    }

    fn tile_at(&self, pos: &Pos) -> &'a TilePlacement {
        if self.valid(pos) {
            &self.fixed_tiles[pos.y as usize][pos.x as usize]
        } else {
            &TilePlacement::None
        }
    }

    fn tile_id_at(&self, pos: &Pos) -> Option<TileID> {
        match self.tile_at(pos) {
            TilePlacement::None => None,
            TilePlacement::Placed { orientation: _, tile } => Some(tile.id)
        }
    }

    fn possible_orientations(&self, pos: &Pos, allowed_neighbours: &AllowedOrientedTiles) -> Result<HashSet<OrientedTile>, TileID> {
        let mut possible = OrientedTileSet::new();

        if let TilePlacement::Placed { tile, orientation } = self.tile_at(&pos.left()) {
            possible.restrict_to(allowed_neighbours.get(tile.id, *orientation, Relationship::RightOf));
        }

        if let TilePlacement::Placed { tile, orientation } = self.tile_at(&pos.up()) {
            possible.restrict_to(allowed_neighbours.get(tile.id, *orientation, Relationship::Below));
            if possible.is_empty() {
                return Err(tile.id);
            }
        }

        if let TilePlacement::Placed { tile, orientation } = self.tile_at(&pos.right()) {
            possible.restrict_to(allowed_neighbours.get(tile.id, *orientation, Relationship::LeftOf));
            if possible.is_empty() {
                return Err(tile.id);
            }
        }

        if let TilePlacement::Placed { tile, orientation } = self.tile_at(&pos.down()) {
            possible.restrict_to(allowed_neighbours.get(tile.id, *orientation, Relationship::Above));
            if possible.is_empty() {
                return Err(tile.id);
            }
        }

        if possible.unrestricted {
            panic!("tried to place a tile with no neighbours at {:?}", pos);
        }

        Ok(possible.oriented_tiles)
    }

    fn try_arrange(&mut self, allowed_neighbours: &AllowedOrientedTiles) -> Result<(), TileID> {
        match self.next_positions.iter().cloned().next() {
            None =>
                Ok(()),

            Some(pos) =>
                match self.possible_orientations(&pos, allowed_neighbours) {
                    Err(tile_id) => Err(tile_id),

                    Ok(oriented_tiles) => {
                        for tile in oriented_tiles.iter() {
                            self.place(&pos, tile.orientation, tile.tile_id);
                            match self.try_arrange(allowed_neighbours) {
                                Err(tile_id)  => {
                                    self.remove(&pos);
                                    if tile_id != tile.tile_id {
                                        // cut search to point where the offending tile was placed
                                        return Err(tile_id);
                                    }
                                }
                                Ok(_) => {
                                    return Ok(());
                                }
                            }
                        }
                        Err(0)
                    }
                }
        }
    }

    fn image(&self) -> Image {
        let mut image = vec![];
        for tiley in 0..self.height as usize {
            for y in 0..8 {
                let mut row = vec![];
                for tilex in 0..self.width as usize {
                    match self.fixed_tiles[tiley][tilex] {
                        TilePlacement::Placed { tile, orientation } => {
                            let mut tile_image = Image::new(&tile.content);
                            tile_image.orientation = orientation;
                            for x in 0..8 {
                                row.push(*tile_image.get(Pos { x, y }));
                            }
                        }
                        _ => {
                            panic!("can't generate image until tiles are arranged");
                        }
                    }
                }
                image.push(row);
            }
        }
        Image::new(&image)
    }
}

impl<'a> fmt::Debug for Arrangement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            write!(f, "\n")?;
            for x in 0..self.width {
                match self.tile_at(&Pos { x, y }) {
                    TilePlacement::None => write!(f, "---- ")?,
                    TilePlacement::Placed { orientation: _, tile } => write!(f, "{:4} ", tile.id)?
                }
            }
        };
        Ok(())
    }
}

fn arrange_tiles<'a>(width: i64, height: i64, tiles: &Vec<&'a Tile>) -> Option<Arrangement<'a>> {
    let allowed_neighbours = AllowedOrientedTiles::new(tiles);

    tiles.iter().filter_map(|tile|
        Orientation::iter().filter_map(|orientation| {
            info!("trying {} {:?} in start position", tile.id, orientation);
            let mut arrangement = Arrangement::new(width, height, tiles);
            arrangement.place(&Pos { x: 0, y: 0 }, orientation, tile.id);
            match arrangement.try_arrange(&allowed_neighbours) {
                Ok(_) => Some(arrangement),
                Err(_) => None
            }
        }).next()
    ).next()
}

impl std::ops::Add<&Pos> for Pos {
    type Output = Pos;
    fn add(self, other: &Pos) -> Self::Output {
        Pos {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }
}

struct Image {
    image: Vec<Vec<char>>,
    orientation: Orientation,
    width: usize,
    height: usize
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "orientation {:?} size {:?}x{:?}", self.orientation, self.width(), self.height())?;
        for y in 0..self.height() {
            for x in 0..self.width() {
                write!(f, "{}", self.get(Pos { x: x as i64, y: y as i64 }))?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl Image {
    fn new(image: &Vec<Vec<char>>) -> Self {
        Image {
            image: image.clone(),
            orientation: Orientation::R0,
            height: image.len(),
            width: image[0].len()
        }
    }

    fn from_str(image: &str) -> Self {
        Image::new(&image.lines().map(|row| row.chars().collect()).collect())
    }

    fn width(&self) -> usize {
        match self.orientation {
            Orientation::R0 | Orientation::R180 | Orientation::R0FlipH | Orientation::R0FlipV => self.width,
            Orientation::R90 | Orientation::R270 | Orientation::R90FlipH | Orientation::R90FlipV => self.height
        }
    }

    fn height(&self) -> usize {
        match self.orientation {
            Orientation::R0 | Orientation::R180 | Orientation::R0FlipH | Orientation::R0FlipV => self.height,
            Orientation::R90 | Orientation::R270 | Orientation::R90FlipH | Orientation::R90FlipV => self.width
        }
    }

    fn transform(&self, pos: Pos) -> (usize, usize) {
        let x = pos.x as usize;
        let y = pos.y as usize;
        let rx = self.width() - 1 - x;
        let ry = self.height() - 1 - y;

        match self.orientation {
            Orientation::R0 => (x, y),
            Orientation::R90 => (ry, x),
            Orientation::R180 => (rx, ry),
            Orientation::R270 => (y, rx),
            Orientation::R0FlipH => (rx, y),
            Orientation::R0FlipV => (x, ry),
            Orientation::R90FlipH => (ry, rx),
            Orientation::R90FlipV => (y, x)
        }
    }

    fn get(&self, pos: Pos) -> &char {
        let (x, y) = self.transform(pos);
        &self.image[y][x]
    }

    fn get_mut(&mut self, pos: Pos) -> &mut char {
        let (x, y) = self.transform(pos);
        &mut self.image[y][x]
    }

    fn iter(&self) -> impl Iterator<Item = Pos> + '_ {
        (0..self.height()).flat_map(move |y|
            (0..self.width()).map(move |x|
                Pos { x: x as i64, y: y as i64 }
            )
        )
    }

    fn has_monster_at(&self, origin: &Pos, monster: &Image) -> bool {
        monster.iter().all(|pos|
            monster.get(pos) == &' ' || self.get(pos + origin) == &'#'
        )
    }

    fn overwrite_monster(&mut self, origin: &Pos, monster: &Image) {
        for pos in monster.iter() {
            if monster.get(pos) == &'#' {
                *self.get_mut(pos + origin) = 'O';
            }
        }
    }

    fn find_monsters(&mut self, monster: &Image) -> usize {
        let mut count = 0;
        for y in 0..(self.height() - monster.height()) {
            for x in 0..(self.width() - monster.width()) {
                let p = Pos { x: x as i64, y: y as i64 };
                if self.has_monster_at(&p, monster) {
                    self.overwrite_monster(&p, monster);
                    count += 1;
                }
            }
        }
        count
    }
}

fn find_monsters(image: &mut Image) -> usize {
    let monster = Image::from_str("                  # \n#    ##    ##    ###\n #  #  #  #  #  #   ");

    Orientation::iter().filter_map(|orientation| {
        image.orientation = orientation;
        let count = image.find_monsters(&monster);
        if count > 0 {
            Some(count)
        } else {
            None
        }
    }).next();

    image.iter().filter(|pos| image.get(*pos) == &'#').count()
}

// -- parser

fn decode_cell(c: char) -> EdgePattern {
    if c == '#' { 1 } else { 0 }
}

fn decode_row(cells: &Vec<char>) -> EdgePattern {
    cells.iter().fold(0, |pattern, cell|
        (pattern << 1) | decode_cell(*cell)
    )
}

fn decode_column(cells: &Vec<Vec<char>>, column: usize) -> EdgePattern {
    cells.iter().fold(0, |pattern, row|
        (pattern << 1) | decode_cell(row[column])
    )
}

fn trim_edges(cells: &Vec<Vec<char>>) -> Vec<Vec<char>> {
    cells[1..cells.len()-1]
        .iter()
        .map(|row| row[1..row.len()-1].iter().copied().collect())
        .collect()
}

fn parse_input(input: &str) -> ParseResult<Vec<Tile>> {
    let tile_id = integer
        .between(match_literal("Tile "), match_literal(":\n"))
        .map(|i| i as TileID);

    let tile_char = any_char.pred(|c| *c == '#' || *c == '.');
    let tile_row = whitespace_wrap(one_or_more(tile_char));
    let tile = pair(tile_id, one_or_more(tile_row), |id, cells|
        Tile {
            id,
            top: decode_row(&cells[0]),
            bottom: decode_row(&cells[cells.len()-1]),
            left: decode_column(&cells, 0),
            right: decode_column(&cells, cells[0].len()-1),
            content: trim_edges(&cells)
        }
    );

    one_or_more(tile).parse(input)
}

// -- problems

fn part1(tiles: &Vec<&Tile>) -> Option<usize> {
    let corners = vec![
        Pos { x:  0, y:  0 },
        Pos { x:  0, y: 11 },
        Pos { x: 11, y:  0 },
        Pos { x: 11, y: 11 }
    ];

    arrange_tiles(12, 12, tiles).map(|arrangement|
        corners.iter().filter_map(|c| arrangement.tile_id_at(c)).product()
    )
}

fn part2(tiles: &Vec<&Tile>) -> usize {
    let mut image = arrange_tiles(12, 12, tiles).unwrap().image();
    find_monsters(&mut image)
}

fn main() {
    env_logger::init();
    let input = include_str!("input.txt");
    let tiles = parse_input(&input).unwrap().1;
    let tiles_by_ref: Vec<&Tile> = tiles.iter().collect();
    println!("part 1 {:?}", part1(&tiles_by_ref).unwrap());
    println!("part 2 {:?}", part2(&tiles_by_ref));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_logging() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn example_input() -> String {
        include_str!("example.txt").to_string()
    }

    fn example_tiles() -> Vec<Tile> {
        let input = example_input();
        let tiles = parse_input(input.as_str());
        assert!(tiles.is_ok());
        tiles.unwrap().1
    }

    #[test]
    fn test_parser() {
        assert_eq!(example_tiles()[0], Tile {
            id: 2311,
            top: 0x0d2,
            bottom: 0x0e7,
            left: 0x1f2,
            right: 0x059,
            content: vec![]
        });
    }

    #[test]
    fn test_orientations_iter() {
        let ors: Vec<Orientation> = Orientation::iter().collect();
        assert_eq!(ors.len(), 8);
    }

    #[test]
    fn test_orientations() {
        use Orientation::*;
        let tile = Tile {
            id: 1,
            top: 0x2F9,
            bottom: 0x077,
            left: 0x325,
            right: 0x16D,
            content: vec![]
        };

        assert_eq!(tile.top_edge_in_orientation(R0), 0x2F9);
        assert_eq!(tile.bottom_edge_in_orientation(R0), 0x077);
        assert_eq!(tile.left_edge_in_orientation(R0), 0x325);
        assert_eq!(tile.right_edge_in_orientation(R0), 0x16D);

        assert_eq!(tile.top_edge_in_orientation(R90), 0x16D);
        assert_eq!(tile.bottom_edge_in_orientation(R90), 0x325);
        assert_eq!(tile.left_edge_in_orientation(R90), 0x27D);
        assert_eq!(tile.right_edge_in_orientation(R90), 0x3B8);

        assert_eq!(tile.top_edge_in_orientation(R180), 0x3B8);
        assert_eq!(tile.bottom_edge_in_orientation(R180), 0x27D);
        assert_eq!(tile.left_edge_in_orientation(R180), 0x2DA);
        assert_eq!(tile.right_edge_in_orientation(R180), 0x293);

        assert_eq!(tile.top_edge_in_orientation(R270), 0x293);
        assert_eq!(tile.bottom_edge_in_orientation(R270), 0x2DA);
        assert_eq!(tile.left_edge_in_orientation(R270), 0x077);
        assert_eq!(tile.right_edge_in_orientation(R270), 0x2F9);
    }

    #[test]
    fn test_allowed_neighbours() {
        use Orientation::*;
        use Relationship::*;

        let tiles = example_tiles();
        let tiles_by_ref: Vec<&Tile> = tiles.iter().collect();
        let allowed_neighbours = AllowedOrientedTiles::new(&tiles_by_ref);

        assert!(allowed_neighbours.get(1951, R0FlipV, Below).contains(&OrientedTile { tile_id: 2729, orientation: R0FlipV }));
        assert!(allowed_neighbours.get(1951, R0FlipV, RightOf).contains(&OrientedTile { tile_id: 2311, orientation: R0FlipV }));
        assert!(allowed_neighbours.get(2729, R0FlipV, Below).contains(&OrientedTile { tile_id: 2971, orientation: R0FlipV }));
        assert!(allowed_neighbours.get(2311, R0FlipV, RightOf).contains(&OrientedTile { tile_id: 3079, orientation: R0 }));
    }

    #[test]
    fn test_arrangement() {
        init_logging();

        let tiles = example_tiles();
        let tiles_by_ref: Vec<&Tile> = tiles.iter().collect();
        let arrangement = arrange_tiles(3, 3, &tiles_by_ref);
        assert!(arrangement.is_some());
        let arrangement = arrangement.unwrap();
        println!("{:?}", arrangement);
        let corners = vec![
            arrangement.tile_id_at(&Pos { x: 0, y: 0 }),
            arrangement.tile_id_at(&Pos { x: 2, y: 0 }),
            arrangement.tile_id_at(&Pos { x: 0, y: 2 }),
            arrangement.tile_id_at(&Pos { x: 2, y: 2 })
        ];
        assert!(corners.contains(&Some(1951)));
        assert!(corners.contains(&Some(3079)));
        assert!(corners.contains(&Some(2971)));
        assert!(corners.contains(&Some(1171)));
    }

    #[test]
    fn test_find_monsters() {
        let tiles = example_tiles();
        let tiles_by_ref: Vec<&Tile> = tiles.iter().collect();
        let mut image = arrange_tiles(3, 3, &tiles_by_ref).unwrap().image();

        assert_eq!(find_monsters(&mut image), 273);
    }
}

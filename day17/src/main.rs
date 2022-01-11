use std::cmp::{max, min};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::ops::{AddAssign, RangeInclusive};

// --- model

#[derive(Eq, PartialEq, Copy, Clone)]
enum Cube {
    Inactive,
    Active
}

impl From<char> for Cube {
    fn from(c: char) -> Self {
        match c {
            '#' => Cube::Active,
            _ => Cube::Inactive
        }
    }
}

trait Position: Eq + Hash {
    fn neighbours(&self) -> Box<dyn Iterator<Item = Self> + '_>;
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
struct Pos3(i64, i64, i64);

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
struct Pos4(i64, i64, i64, i64);

impl Position for Pos3 {
    fn neighbours(&self) -> Box<dyn Iterator<Item = Self> + '_> {
        let it = (-1..=1).flat_map(
            move |z| (-1..=1).flat_map(
                move |y| (-1..=1).map(
                    move |x| Pos3(self.0+x, self.1+y, self.2+z)
                )
            )
        ).filter(move |p| p != self);

        Box::new(it)
    }
}

impl Position for Pos4 {
    fn neighbours(&self) -> Box<dyn Iterator<Item = Self> + '_> {
        let it = (-1..=1).flat_map(
            move |w| (-1..=1).flat_map(
                move |z| (-1..=1).flat_map(
                    move |y| (-1..=1).map(
                        move |x| Pos4(self.0+x, self.1+y, self.2+z, self.3+w)
                    )
                )
            )
        ).filter(move |p| p != self);

        Box::new(it)
    }
}

struct Bounds3 {
    x: RangeInclusive<i64>,
    y: RangeInclusive<i64>,
    z: RangeInclusive<i64>
}

struct Bounds4 {
    x: RangeInclusive<i64>,
    y: RangeInclusive<i64>,
    z: RangeInclusive<i64>,
    w: RangeInclusive<i64>
}

impl Default for Bounds3 {
    fn default() -> Self {
        Bounds3 {
            x: 0..=0,
            y: 0..=0,
            z: 0..=0
        }
    }
}

impl Default for Bounds4 {
    fn default() -> Self {
        Bounds4 {
            x: 0..=0,
            y: 0..=0,
            z: 0..=0,
            w: 0..=0
        }
    }
}

impl AddAssign<Pos3> for Bounds3 {
    fn add_assign(&mut self, pos: Pos3) {
        self.x = min(*self.x.start(), pos.0) ..= max(*self.x.end(), pos.0);
        self.y = min(*self.y.start(), pos.1) ..= max(*self.y.end(), pos.1);
        self.z = min(*self.z.start(), pos.2) ..= max(*self.z.end(), pos.2);
    }
}

impl AddAssign<Pos4> for Bounds4 {
    fn add_assign(&mut self, pos: Pos4) {
        self.x = min(*self.x.start(), pos.0) ..= max(*self.x.end(), pos.0);
        self.y = min(*self.y.start(), pos.1) ..= max(*self.y.end(), pos.1);
        self.z = min(*self.z.start(), pos.2) ..= max(*self.z.end(), pos.2);
        self.w = min(*self.w.start(), pos.3) ..= max(*self.w.end(), pos.3);
    }
}

trait Dimension<Pos: Position + Copy> where Self: Sized {
    fn grid(&self) -> &HashMap<Pos, Cube>;

    fn iter(&self) -> Box<dyn Iterator<Item = Pos> + '_>;

    fn at(&self, p: &Pos) -> &Cube;

    fn next_generation(&self) -> Self;

    fn occupied_neighbours(&self, p: &Pos) -> usize {
        p.neighbours()
            .filter(|p|
                self.at(p) == &Cube::Active
            ).count()
    }

    fn bounds<Bounds: Default + AddAssign<Pos>>(&self) -> Bounds {
        let mut bounds = Bounds::default();
        for pos in self.grid().keys() {
            bounds += *pos;
        }
        bounds
    }

    fn active_cubes(&self) -> usize {
        self.grid().values().filter(|c| *c == &Cube::Active).count()
    }

    fn next_generation_grid(&self) -> HashMap<Pos, Cube> {
        self.iter().map(|pos| {
            let occupied = self.occupied_neighbours(&pos);
            let new_state = match self.at(&pos) {
                Cube::Active =>
                    if occupied == 2 || occupied == 3 {
                        Cube::Active
                    } else {
                        Cube::Inactive
                    }

                Cube::Inactive =>
                    if occupied == 3 {
                        Cube::Active
                    } else {
                        Cube::Inactive
                    }
            };
            (pos, new_state)
        }).collect()
    }
}

#[derive(Clone)]
struct PocketDimension<Pos: Position> {
    grid: HashMap<Pos, Cube>
}


impl PartialEq for PocketDimension<Pos3> {
    fn eq(&self, other: &Self) -> bool {
        let mut bounds: Bounds3 = self.bounds();
        for pos in other.iter() {
            bounds += pos;
        }
        for z in bounds.z {
            for y in (&bounds.y).clone() {
                for x in (&bounds.x).clone() {
                    let pos = Pos3(x, y, z);
                    if self.at(&pos) != other.at(&pos) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

impl PartialEq for PocketDimension<Pos4> {
    fn eq(&self, other: &Self) -> bool {
        let mut bounds: Bounds4 = self.bounds();
        for pos in other.iter() {
            bounds += pos;
        }
        for w in bounds.w {
            for z in bounds.z.clone() {
                for y in (&bounds.y).clone() {
                    for x in (&bounds.x).clone() {
                        let pos = Pos4(x, y, z, w);
                        if self.at(&pos) != other.at(&pos) {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
}

impl PocketDimension<Pos3> {
    fn new3(origin: &Pos3, s: &str) -> Self {
        let mut grid = HashMap::new();

        for (z, zs) in s.split("\n\n").enumerate() {
            for (y, ys) in zs.lines().enumerate() {
                for (x, xs) in ys.trim().chars().enumerate() {
                    grid.insert(Pos3(origin.0 + x as i64, origin.1 + y as i64, origin.2 + z as i64), Cube::from(xs));
                }
            }
        }

        PocketDimension { grid }
    }
}

impl Dimension<Pos3> for PocketDimension<Pos3> {
    fn grid(&self) -> &HashMap<Pos3, Cube> {
        &self.grid
    }

    fn at(&self, p: &Pos3) -> &Cube {
        self.grid.get(p).unwrap_or(&Cube::Inactive)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Pos3> + '_> {
        let bounds: Bounds3 = self.bounds();
        let (xmin, xmax) = (*bounds.x.start() - 1, *bounds.x.end() + 1);
        let (ymin, ymax) = (*bounds.y.start() - 1, *bounds.y.end() + 1);
        let (zmin, zmax) = (*bounds.z.start() - 1, *bounds.z.end() + 1);

        let it = (zmin..=zmax).flat_map(move |z|
            (ymin..=ymax).flat_map(move |y|
                (xmin..=xmax).map(move |x| Pos3(x, y, z) )
            )
        );

        Box::new(it)
    }

    fn next_generation(&self) -> Self {
        PocketDimension { grid: self.next_generation_grid() }
    }
}

impl PocketDimension<Pos4> {
    fn new4(origin: &Pos4, s: &str) -> Self {
        let mut grid = HashMap::new();

        for (z, zs) in s.split("\n\n").enumerate() {
            for (y, ys) in zs.lines().enumerate() {
                for (x, xs) in ys.trim().chars().enumerate() {
                    grid.insert(Pos4(origin.0 + x as i64, origin.1 + y as i64, origin.2 + z as i64, 0), Cube::from(xs));
                }
            }
        }

        PocketDimension { grid }
    }
}

impl Dimension<Pos4> for PocketDimension<Pos4> {
    fn grid(&self) -> &HashMap<Pos4, Cube> {
        &self.grid
    }

    fn at(&self, p: &Pos4) -> &Cube {
        self.grid.get(p).unwrap_or(&Cube::Inactive)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Pos4> + '_> {
        let bounds: Bounds4 = self.bounds();
        let (xmin, xmax) = (*bounds.x.start() - 1, *bounds.x.end() + 1);
        let (ymin, ymax) = (*bounds.y.start() - 1, *bounds.y.end() + 1);
        let (zmin, zmax) = (*bounds.z.start() - 1, *bounds.z.end() + 1);
        let (wmin, wmax) = (*bounds.w.start() - 1, *bounds.w.end() + 1);

        let it = (wmin..=wmax).flat_map(move |w|
            (zmin..=zmax).flat_map(move |z|
                (ymin..=ymax).flat_map(move |y|
                    (xmin..=xmax).map(move |x| Pos4(x, y, z, w) )
                )
            )
        );

        Box::new(it)
    }

    fn next_generation(&self) -> Self {
        PocketDimension { grid: self.next_generation_grid() }
    }

}

impl fmt::Debug for Cube {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cube::Inactive => write!(f, "."),
            Cube::Active => write!(f, "#")
        }
    }
}

impl fmt::Debug for PocketDimension<Pos3> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bounds: Bounds3 = self.bounds();
        write!(f, "zs={:?} ys={:?} xs={:?}\n", bounds.z, bounds.y, bounds.x)?;
        for z in bounds.z {
            write!(f, "z={:?}\n", z)?;
            for y in (&bounds.y).clone() {
                for x in (&bounds.x).clone() {
                    write!(f, "{:?}", self.at(&Pos3(x,y,z)))?;
                }
                write!(f, " {}\n", y)?;
            }
        }
        Ok(())
    }
}

// --- problems

fn part1(input: &str) -> usize {
    let mut p = PocketDimension::new3(&Pos3(0,0,0), input);
    for _ in 0..6 {
        p = p.next_generation();
    }
    p.active_cubes()
}

fn part2(input: &str) -> usize {
    let mut p = PocketDimension::new4(&Pos4(0,0,0,0), input);
    for _ in 0..6 {
        p = p.next_generation();
    }
    p.active_cubes()
}


fn main() {
    let input = include_str!("input.txt");
    println!("part1 {:?}", part1(&input));
    println!("part2 {:?}", part2(&input));
}


#[cfg(test)]
mod tests {
    use super::*;

    fn test_grid() -> &'static str {
        ".#.
         ..#
         ###"
    }

    #[test]
    fn test_init() {
        let pd = PocketDimension::new3(&Pos3(0,0,0), test_grid());
        assert_eq!(pd.at(&Pos3(0,0,0)), &Cube::Inactive);
        assert_eq!(pd.at(&Pos3(1,0,0)), &Cube::Active);
        assert_eq!(pd.at(&Pos3(3,6,9)), &Cube::Inactive);
        assert_eq!(pd.at(&Pos3(2,1,0)), &Cube::Active);
    }

    #[test]
    fn test_neighbours_3d() {
        assert_eq!(Pos3(0,0,0).neighbours().count(), 26);
    }

    #[test]
    fn test_neighbours_4d() {
        assert_eq!(Pos4(0,0,0,0).neighbours().count(), 80);
    }

    #[test]
    fn test_occupied_neighbours() {
        let pd = PocketDimension::new3(&Pos3(0,0,0), test_grid());
        assert_eq!(pd.occupied_neighbours(&Pos3(0,0,0)), 1);
        assert_eq!(pd.occupied_neighbours(&Pos3(1,2,0)), 3);
    }

    #[test]
    fn test_generations() {
        let pd = PocketDimension::new3(&Pos3(0,0,0), test_grid());

        let gen1 = pd.next_generation();
        assert_eq!(gen1, PocketDimension::new3(&Pos3(0,1,-1),
                                               "#..
             ..#
             .#.

             #.#
             .##
             .#.

             #..
             ..#
             .#."
        ));

        let gen2 = gen1.next_generation();
        assert_eq!(gen2, PocketDimension::new3(&Pos3(-1,0,-2),
                                               ".....
             .....
             ..#..
             .....
             .....

             ..#..
             .#..#
             ....#
             .#...
             .....

             ##...
             ##...
             #....
             ....#
             .###.

             ..#..
             .#..#
             ....#
             .#...
             .....

             .....
             .....
             ..#..
             .....
             ....."
        ));
    }

    #[test]
    fn test_six_generations_v1() {
        let mut p = PocketDimension::new3(&Pos3(0,0,0), test_grid());
        for _ in 0..6 {
            p = p.next_generation();
        }
        assert_eq!(p.active_cubes(), 112);
    }
}
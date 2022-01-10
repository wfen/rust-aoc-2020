use itertools::Itertools;
use std::fmt;

#[derive(Debug)]
struct ProblemStatement1 {
    departure_time: usize,
    buses: Vec<usize>,
}

impl ProblemStatement1 {
    fn parse(input: &str) -> Self {
        let mut lines = input.lines();
        ProblemStatement1 {
            departure_time: lines.next().unwrap().parse().unwrap(),
            buses: lines
                .next()
                .unwrap()
                .split(',')
                .filter(|&s| s != "x")
                .map(|x| x.parse().unwrap())
                .collect(),
        }
    }
}

#[derive(Debug)]
struct ProblemStatement {
    #[allow(dead_code)]
    departure_time: usize,
    buses: Vec<Bus>,
}

#[derive(Debug)]
struct Bus {
    id: usize,
    time_offset: usize,
}

impl ProblemStatement {
    fn parse(input: &str) -> Self {
        let mut lines = input.lines();
        ProblemStatement {
            departure_time: lines.next().unwrap().parse().unwrap(),
            buses: lines
                .next()
                .unwrap()
                .split(',')
                .enumerate()
                .filter_map(|(index, input)| {
                    input.parse().ok().map(|id| Bus {
                        id,
                        time_offset: index,
                    })
                })
                .collect(),
        }
    }

    #[allow(dead_code)]
    fn check_solution(&self, solution: usize) -> Result<(), WrongGap<'_>> {
        self.buses
            .iter()
            .tuple_windows()
            // 👇 here's our `try_fold` used to "short-circuit" a fold
            .try_fold(solution, |acc, (earlier, later)| {
                // 👇 that debug print is still here for now
                //    (note that `acc` is now a `usize`, not a `Result<usize, WrongGap>`)
                //dbg!(&acc);

                let earlier_timestamp = acc;
                let later_timestamp = earlier_timestamp + later.id - (earlier_timestamp % later.id);

                let offset_gap = later.time_offset - earlier.time_offset;
                let actual_gap = later_timestamp - earlier_timestamp;

                // 👇 we still return a `Result` though!
                if offset_gap == actual_gap {
                    Ok(later_timestamp)
                } else {
                    Err(WrongGap {
                        earlier,
                        later,
                        earlier_timestamp,
                        offset_gap,
                        actual_gap,
                    })
                }
            })
            .map(|_| ())
    }

    /*fn solve(&self) -> usize {
        let first_bus = self.buses.first().unwrap();
        itertools::iterate(0, |&i| i + first_bus.id)
            .find(|&timestamp| self.check_solution(timestamp).is_ok())
            .unwrap()
    }
    */
    fn solve(&self) -> i64 {
        solve_lincon_system(self.buses.iter().map(|bus| LinearCongruence {
            lhs: Expr::Var('x'),
            // 👇👇👇
            rhs: Expr::Literal((bus.id as i64 - bus.time_offset as i64).rem_euclid(bus.id as _)),
            //rhs: Expr::Literal(bus.time_offset as _),
            modulo: bus.id as _,
        }))
    }
}

#[derive(Debug)]
struct WaitTime {
    bus_id: usize,
    /// in minutes
    wait: usize,
}

struct WrongGap<'a> {
    earlier: &'a Bus,
    later: &'a Bus,
    #[allow(dead_code)]
    earlier_timestamp: usize,
    offset_gap: usize,
    actual_gap: usize,
}

impl fmt::Debug for WrongGap<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "expected Bus {} to leave {} minutes after Bus {}, but it left {} minutes after",
            self.later.id, self.earlier.id, self.offset_gap, self.actual_gap
        )
    }
}

impl fmt::Display for WrongGap<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for WrongGap<'_> {}

#[derive(Clone, PartialEq, Eq)]
enum Expr {
    Literal(i64),
    Var(char),
    Add(Vec<Expr>),
    Mul(Vec<Expr>),
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &Expr::Literal(lit) => write!(f, "{}", lit),
            //  👇
            Expr::Var(c) => write!(f, "{}", c),
            Expr::Add(terms) => {
                write!(f, "(")?;
                for (i, term) in terms.iter().enumerate() {
                    if i == 0 {
                        write!(f, "{:?}", term)?;
                    } else {
                        write!(f, " + {:?}", term)?;
                    }
                }
                write!(f, ")")?;
                Ok(())
            }
            Expr::Mul(terms) => {
                write!(f, "(")?;
                for (i, term) in terms.iter().enumerate() {
                    if i == 0 {
                        write!(f, "{:?}", term)?;
                    } else {
                        write!(f, " * {:?}", term)?;
                    }
                }
                write!(f, ")")?;
                Ok(())
            }
        }
    }
}

impl Expr {
    /// Multiply `self` by `expr`
    fn mul(&self, expr: Expr) -> Self {
        match self {
            Self::Mul(items) => {
                Self::Mul(std::iter::once(expr).chain(items.iter().cloned()).collect())
            }
            _ => Self::Mul(vec![expr, self.clone()]),
        }
    }

    /// Add `self` by `expr`
    fn add(&self, expr: Expr) -> Self {
        match self {
            Self::Add(items) => {
                Self::Add(std::iter::once(expr).chain(items.iter().cloned()).collect())
            }
            _ => Self::Add(vec![expr, self.clone()]),
        }
    }

    fn modulo(&self, modulo: u32) -> Self {
        match self {
            &Self::Literal(lit) => Expr::Literal(lit.rem_euclid(modulo as _)),
            Self::Var(c) => Expr::Var(*c),
            Self::Add(_) => self.clone(),
            Self::Mul(items) => Self::Mul(items.iter().map(|x| x.modulo(modulo)).collect()),
        }
    }

    // Replaces `Expr::Var` with `expr` everywhere in that expression
    fn replace(&self, expr: Expr) -> Self {
        match self {
            &Expr::Literal(lit) => Expr::Literal(lit),
            Expr::Var(_) => expr,
            Expr::Add(items) => Expr::Add(
                items
                    .iter()
                    .cloned()
                    .map(|ex| ex.replace(expr.clone()))
                    .collect(),
            ),
            Expr::Mul(items) => Expr::Mul(
                items
                    .iter()
                    .cloned()
                    .map(|ex| ex.replace(expr.clone()))
                    .collect(),
            ),
        }
    }

    fn distribute(&self) -> Self {
        if let Self::Mul(items) = self {
            if let [Self::Literal(lit), Self::Add(add_terms)] = &items[..] {
                return Self::Add(
                    add_terms
                        .iter()
                        .map(|ex| ex.mul(Self::Literal(*lit)))
                        .collect(),
                );
            }
        }

        // 👇 new!
        if let Self::Add(items) = self {
            return Self::Add(items.iter().map(|ex| ex.distribute()).collect());
        }

        self.clone()
    }

    fn reduce(&self) -> Expr {
        match self {
            &Expr::Literal(lit) => Expr::Literal(lit),
            Expr::Var(c) => Expr::Var(*c),
            Expr::Add(items) => {
                // 👇 new!
                if let Some((index, nested_items)) =
                items
                    .iter()
                    .enumerate()
                    .find_map(|(index, item)| match item {
                        Expr::Add(terms) => Some((index, terms)),
                        _ => None,
                    })
                {
                    return Expr::Add(
                        items
                            .iter()
                            .enumerate()
                            .filter(|&(i, _)| i != index)
                            .map(|(_, item)| item)
                            .chain(nested_items)
                            .cloned()
                            .collect(),
                    )
                        .reduce();
                }
                let (literals, others): (Vec<_>, Vec<_>) = items
                    .iter()
                    .map(Self::reduce)
                    .partition(|x| matches!(x, Self::Literal(_)));

                if literals.is_empty() && others.is_empty() {
                    Expr::Literal(0)
                } else {
                    let mut terms = others;
                    let sum = literals
                        .into_iter()
                        .map(|x| {
                            if let Expr::Literal(x) = x {
                                x
                            } else {
                                unreachable!()
                            }
                        })
                        .sum();
                    if sum != 0 {
                        if terms.is_empty() {
                            return Self::Literal(sum);
                        } else {
                            terms.insert(0, Self::Literal(sum));
                        }
                    }
                    if terms.len() == 1 {
                        terms.pop().unwrap()
                    } else {
                        Expr::Add(terms)
                    }
                }
            }
            Expr::Mul(items) => {
                let (literals, others): (Vec<_>, Vec<_>) = items
                    .iter()
                    .map(Self::reduce)
                    .partition(|x| matches!(x, Self::Literal(_)));

                if literals.is_empty() && others.is_empty() {
                    Expr::Literal(1)
                } else {
                    let mut terms = others;
                    let product = literals
                        .into_iter()
                        .map(|x| {
                            if let Expr::Literal(x) = x {
                                x
                            } else {
                                unreachable!()
                            }
                        })
                        .product();
                    if product != 1 {
                        if terms.is_empty() {
                            return Self::Literal(product);
                        } else {
                            terms.insert(0, Self::Literal(product));
                        }
                    }
                    if terms.len() == 1 {
                        terms.pop().unwrap()
                    } else {
                        Expr::Mul(terms)
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct LinearCongruence {
    lhs: Expr,
    rhs: Expr,
    modulo: u32,
}

impl fmt::Debug for LinearCongruence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} ≡ {:?} (mod {})", self.lhs, self.rhs, self.modulo)
    }
}

#[derive(Debug)]
struct CantSolve(LinearCongruence);

impl fmt::Display for CantSolve {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for CantSolve {}

impl LinearCongruence {
    /// Multiply both sides of congruence by `expr`
    fn mul(&self, expr: Expr) -> Self {
        Self {
            lhs: self.lhs.mul(expr.clone()).reduce().modulo(self.modulo),
            rhs: self.rhs.mul(expr).reduce().modulo(self.modulo),
            modulo: self.modulo,
        }
    }

    /// Add both sides of congruence by `expr`
    fn add(&self, expr: Expr) -> Self {
        Self {
            lhs: self.lhs.add(expr.clone()).reduce().modulo(self.modulo),
            rhs: self.rhs.add(expr).reduce().modulo(self.modulo),
            modulo: self.modulo,
        }
    }

    fn solve(&self) -> Result<Self, CantSolve> {
        eprintln!("should solve {:?}", self);
        if let Expr::Mul(items) = &self.lhs {
            if let [Expr::Literal(lit), Expr::Var(_)] = items[..] {
                let mmi = modular_multiplicative_inverse(lit, self.modulo);
                eprintln!("multiplying by mmi: {}", mmi);
                return self.mul(Expr::Literal(mmi)).solve();
            }
        }

        if let Expr::Add(items) = &self.lhs {
            if let Some(lit) = items.iter().find_map(|expr| match *expr {
                Expr::Literal(lit) => Some(lit),
                _ => None,
            }) {
                eprintln!("adding {} on both sides", -lit);
                return self.add(Expr::Literal(-lit)).solve();
            }
        }

        if let Expr::Var(_) = &self.lhs {
            // already solved!
            return Ok(self.clone());
        }

        Err(CantSolve(self.clone()))
    }

    /// Turns this linear congruence into an expression,
    /// for example `x ≡ 7 (mod 13)` would give `13*var + 7`.
    /// Panics if linear congruence is not solved yet.
    //               👇
    fn expr(&self, name: char) -> Expr {
        match (&self.lhs, &self.rhs) {
            (Expr::Var(_), &Expr::Literal(remainder)) => Expr::Add(vec![
                //                                                         👇
                Expr::Mul(vec![Expr::Literal(self.modulo as _), Expr::Var(name)]),
                Expr::Literal(remainder),
            ]),
            _ => {
                panic!(
                    "Expected solved congruence (of form `var ≡ literal (mod m)`), but got `{:?}`",
                    self
                )
            }
        }
    }

    // Replaces `Expr::Var` with `expr` everywhere in that expression
    fn replace(&self, expr: Expr) -> Self {
        Self {
            lhs: self.lhs.replace(expr.clone()),
            rhs: self.rhs.replace(expr),
            modulo: self.modulo,
        }
    }
}

/// Finds the modular multiplicative inverse of `a` modulo `m`
/// Returns the wrong result if `m` isn't prime.
fn modular_multiplicative_inverse(a: i64, m: u32) -> i64 {
    modular_pow(a, m - 2, m as _)
}

fn modular_pow(x: i64, exp: u32, modulo: i64) -> i64 {
    (match x.checked_pow(exp) {
        Some(x) => x,
        None => {
            let exp_a = exp / 2;
            let exp_b = exp - exp_a;
            modular_pow(x, exp_a, modulo) * modular_pow(x, exp_b, modulo)
        }
    }) % modulo
}

fn solve_lincon_system<I>(mut cons: I) -> i64
    where
        I: Iterator<Item = LinearCongruence> {
    //println!("Solving system of {} linear congruences", cons.len()); // len() bad for iterator

    // Variable naming
    let mut curr_var = b'a';
    let mut next_var = || -> char {
        let res = curr_var as char;
        curr_var += 1;
        res
    };

    //let mut cons = cons.iter(); // now part of function signature
    let con = cons.next().unwrap();
    println!("👉 {:?}", con);
    let mut x = con.expr(next_var()).reduce();
    println!("x = {:?}", x);

    for con in cons {
        println!("👉 {:?}", con);
        x = x
            .replace(con.replace(x.clone()).solve().unwrap().expr(next_var()))
            .distribute()
            .reduce();
        println!("x = {:?}", x);
    }

    let x = x.replace(Expr::Literal(0)).reduce();
    if let Expr::Literal(lit) = x {
        lit
    } else {
        panic!("expected `x` to be a literal but got {:?}", x)
    }
}

#[allow(dead_code, non_snake_case)]
fn solve_lincong_system_direct<I>(congs: I) -> i64
    where
        I: Iterator<Item = LinearCongruence>,
{
    // This time, we need to be able to index our linear congruences
    let congs: Vec<_> = congs.collect();

    fn remainder(lc: &LinearCongruence) -> i64 {
        match &lc.rhs {
            Expr::Literal(lit) => *lit,
            _ => panic!(),
        }
    }

    (0..congs.len())
        .map(|i| {
            let a_i = remainder(&congs[i]);
            let N_i = congs
                .iter()
                .enumerate()
                .filter(|&(j, _)| j != i)
                .map(|(_, con)| con.modulo as i64)
                .product();

            let M_i = modular_multiplicative_inverse(N_i, congs[i].modulo);

            a_i * N_i * M_i
        })
        .sum()
}

fn main() {
    let stat = ProblemStatement1::parse(include_str!("input.txt"));
    //dbg!(stat);

    /*
    let times: Vec<_> = stat
        .buses
        .iter()
        .map(|&bus_id| WaitTime {
            bus_id,
            wait: bus_id - stat.departure_time % bus_id,
        })
        .collect();
    dbg!(times);
    */

    // we need to find the bus that leaves at the earliest time following our earliest departure time
    // i.e. the minimum WaitTime::wait
    let answer = stat
        .buses
        .iter()
        .map(|&bus_id| WaitTime {
            bus_id,
            wait: bus_id - stat.departure_time % bus_id,
        })
        .min_by_key(|wt| wt.wait);

    println!("Part 1:");
    match answer {
        Some(wt) => {
            println!("  bus_id({}) * wait({}) = {}", wt.bus_id, wt.wait, wt.bus_id * wt.wait);
        }
        None => {
            panic!("No answer found!");
        }
    }
    println!();

    // part2 7,13,x,x,59,x,31,19... x values matter because we take into account the position of a bus ID in the list
    // 7 => 0   13 => 1   59 => 4   31 => 6   19 => 7      we should find a timestamp t such that: bus 7 departs at t,
    // bus 13 departs at t + 1,   bus 59 departs at t + 4,   bus 31 departs at t + 6,   bus 19 departs at t + 7

    //let stat = ProblemStatement::parse(include_str!("input.txt"));
    //dbg!(stat);

    // iterate over all our buses, and use tuple_windows to consider them pair-wise
    /*
    stat.buses
        .iter()
        .tuple_windows()
        .for_each(|(earlier, later)| {
            let offset_gap = later.time_offset - earlier.time_offset;
            dbg!("-----------", earlier.id, later.id, offset_gap);
        });
    */

    // imagine we already know a potential solution (i.e. departure time for the last bus, Bus 19)
    // can we check it?
    // dbg!(&stat.check_solution(1068781_usize)); // check a known to be good solution
    // dbg!(&stat.check_solution(256)); // check a known to be bad solution

    //dbg!(&stat.solve());  // takes too long... maybe never finishes

    /*
    let lc = LinearCongruence {
        lhs: Expr::Mul(vec![Expr::Literal(17), Expr::Var('x')]),
        rhs: Expr::Literal(2),
        modulo: 13,
    };
    let lc = lc.solve('x').unwrap();
    dbg!(&lc);
    let expr = lc.expr('x');
    dbg!(&expr);

    let expr = LinearCongruence {
        lhs: Expr::Var('x'),
        rhs: Expr::Literal(0),
        modulo: 17,
    }
        .expr('x')
        .replace(expr);
    dbg!(&expr);
    */

    /*
    let con1 = LinearCongruence {
        lhs: Expr::Var('x'),
        rhs: Expr::Literal(0),
        modulo: 17,
    };
    let con2 = LinearCongruence {
        lhs: Expr::Var('x'),
        rhs: Expr::Literal(2),
        modulo: 13,
    };
    let con3 = LinearCongruence {
        lhs: Expr::Var('x'),
        rhs: Expr::Literal(3),
        modulo: 19,
    };

    println!(
        "✅ Solution: {}",
        solve_lincon_system(vec![con1, con2, con3].into_iter())
    );

    println!(
        "✅ Solution: {}",
        ProblemStatement::parse("0\n17,x,13,19").solve()
    );
    */

    println!("Part 2:");
    println!(
        "✅ Solution: {}",
        ProblemStatement::parse(include_str!("input.txt")).solve()
    );
}

#[test]
fn test_solutions() {
    // using a macro to allow us to group all the test's data neatly in one place,
    // and leave the logic elsewhere
    macro_rules! test {
        ($list: literal, $solution: expr) => {
            assert_eq!(
                ProblemStatement::parse(concat!("0\n", $list, "\n")).solve(),
                $solution
            )
        };
    }

    test!("17,x,13,19", 3417);
    test!("67,7,59,61", 754018);
    test!("67,x,7,59,61", 779210);
    test!("67,7,x,59,61", 1261476);
    test!("1789,37,47,1889", 1202161486);
}

#[test]
fn test_reduce() {
    assert_eq!(Expr::Add(vec![]).reduce(), Expr::Literal(0).reduce());

    assert_eq!(
        Expr::Add(vec![Expr::Literal(2), Expr::Literal(3)]).reduce(),
        Expr::Add(vec![Expr::Literal(5)]).reduce(),
    );

    assert_eq!(
        Expr::Add(vec![Expr::Literal(2), Expr::Literal(3), Expr::Literal(5)]).reduce(),
        Expr::Add(vec![Expr::Literal(10)]).reduce(),
    );

    assert_eq!(
        Expr::Add(vec![Expr::Literal(2), Expr::Literal(3), Expr::Var('x')]).reduce(),
        Expr::Add(vec![Expr::Literal(5), Expr::Var('x')]).reduce(),
    );

    assert_eq!(
        Expr::Mul(vec![Expr::Literal(2), Expr::Literal(3), Expr::Var('x')]).reduce(),
        Expr::Mul(vec![Expr::Literal(6), Expr::Var('x')]).reduce(),
    );

    assert_eq!(
        Expr::Mul(vec![
            Expr::Add(vec![Expr::Literal(2), Expr::Literal(3)]),
            Expr::Literal(10),
            Expr::Var('x')
        ])
            .reduce(),
        Expr::Mul(vec![Expr::Literal(50), Expr::Var('x')]).reduce(),
    );
}

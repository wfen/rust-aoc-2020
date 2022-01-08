// using multimap to store multiple elements in a thinly wrapped HashMap
use multimap::MultiMap;
use std::fmt;
use itertools::Itertools;

/// K can contain V.0 of V.1
type Rules<'a> = MultiMap<BagSpec<'a>, (usize, BagSpec<'a>)>;

/// (adjective, color), i.e. ("dark", "orange")
type BagSpec<'a> = (&'a str, &'a str);

fn parse_rules(input: &str) -> Rules<'_> {
    let mut rules: Rules = Default::default();

    peg::parser! {
        pub(crate) grammar parser() for str {
            pub(crate) rule root(r: &mut Rules<'input>)
                = (line(r) "." whitespace()*)* ![_]

            rule line(r: &mut Rules<'input>)
                = spec:bag_spec() " contain " rules:rules() {
                if let Some(rules) = rules {
                    for rule in rules {
                        r.insert(spec, rule)
                    }
                }
            }

            rule bag_spec() -> BagSpec<'input>
                = adjective:name() " " color:name() " bag" "s"? { (adjective, color) }

            rule rules() -> Option<Vec<(usize, BagSpec<'input>)>>
                = rules:rule1()+ { Some(rules) }
                / "no other bags" { None }

            /// Rule followed by an optional comma and space
            rule rule1() -> (usize, BagSpec<'input>)
                = r:rule0() ", "? { r }

            /// A single rule
            rule rule0() -> (usize, BagSpec<'input>)
                = quantity:number() " " spec:bag_spec() { (quantity, spec) }

            rule number() -> usize
                = e:$(['0'..='9']+) { e.parse().unwrap() }

            /// A sequence of non-whitespace characters
            rule name() -> &'input str
                = $((!whitespace()[_])*)

            /// Spaces, tabs, CR and LF
            rule whitespace()
                = [' ' | '\t' | '\r' | '\n']
        }
    }

    parser::root(input, &mut rules).unwrap();
    rules
}

// replicate the formatting of the input, for inspection
struct FormattedRules<'a>(Rules<'a>);

impl fmt::Display for FormattedRules<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (k, vv) in &self.0 {
            write!(f, "{} {} bags contain ", k.0, k.1)?;
            if vv.is_empty() {
                write!(f, "no other bags")?;
            } else {
                for (i, v) in vv.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(
                        f,
                        "{} {} {} {}",
                        v.0,
                        v.1 .0,
                        v.1 .1,
                        if v.0 == 1 { "bag" } else { "bags" }
                    )?;
                }
            }
            writeln!(f, ".")?;
        }
        Ok(())
    }
}

// subgraph_contains walks the graph starting from _all the nodes_, it walks the same subgraph multiple times
fn subgraph_contains(graph: &Rules<'_>, root: &(&str, &str), needle: &(&str, &str)) -> bool {
    graph
        .get_vec(root)
        .unwrap_or(&Default::default())
        .iter()
        .any(|(_, neighbor)| neighbor == needle || subgraph_contains(graph, neighbor, needle))
    /*
    if let Some(neighbors) = graph.get_vec(root) {
        for (_, neighbor) in neighbors {
            if neighbor == needle || subgraph_contains(graph, neighbor, needle) {
                return true;
            }
        }
    }
    false

    graph
        .get_vec(root)
        .map(|v| {
            v.iter().any(|(_, neighbor)| {
                neighbor == needle || subgraph_contains(graph, neighbor, needle)
            })
        })
        .unwrap_or_default()
    */
}

// as an optimization, we can make the arrows go up
fn reverse_graph<'a>(graph: &Rules<'a>) -> Rules<'a> {
    graph
        .iter_all()
        .flat_map(|(&node, neighbors)| {
            neighbors
                .iter()
                .map(move |&(quantity, neighbor)| (neighbor, (quantity, node)))
        })
        .collect()
    /*
    let mut reverse: Rules = Default::default();
    for (&node, neighbors) in graph.iter_all() {
        for &(quantity, neighbor) in neighbors {
            reverse.insert(neighbor, (quantity, node));
        }
    }
    reverse
    */
}

#[allow(dead_code)]
// walk_subgraph() naively walks the graph, visiting some nodes more than once; it returns duplicates
fn walk_subgraph<'a>(graph: &Rules<'a>, root: &(&str, &str)) -> Vec<(&'a str, &'a str)> {
    let mut res: Vec<_> = Default::default();
    if let Some(neighbors) = graph.get_vec(root) {
        for &(_quantity, neighbor) in neighbors {
            res.push(neighbor);
            res.extend(walk_subgraph(graph, &neighbor));
        }
    }
    res
}

#[allow(dead_code)]
// walk_subgraph1() strives to not allocate a Vec every time we walk a subgraph; takes a &mut Vec
fn walk_subgraph1<'a>(graph: &Rules<'a>, root: &(&str, &str), res: &mut Vec<(&'a str, &'a str)>) {
    if let Some(neighbors) = graph.get_vec(root) {
        for &(_quantity, neighbor) in neighbors {
            res.push(neighbor);
            walk_subgraph1(graph, &neighbor, res);
        }
    }
}

// walk_subgraph2() returns an iterator; leverages Box
fn walk_subgraph2<'iter, 'elems: 'iter>(
    graph: &'iter Rules<'elems>,
    root: &(&'iter str, &'iter str),
) -> Box<dyn Iterator<Item = (&'elems str, &'elems str)> + 'iter> {
    Box::new(
        graph
            .get_vec(root)
            .into_iter()
            .flatten()
            .flat_map(move |&(_, neighbor)| {
                std::iter::once(neighbor).chain(walk_subgraph2(graph, &neighbor))
            }),
    )
}

#[allow(dead_code)]
// walk_subgraph3() includes quantities
fn walk_subgraph3<'iter, 'elems: 'iter>(
    graph: &'iter Rules<'elems>,
    root: &(&'iter str, &'iter str),
    //                       👇 we're now returning the quantity as well
) -> Box<dyn Iterator<Item = (usize, (&'elems str, &'elems str))> + 'iter> {
    Box::new(
        graph
            .get_vec(root)
            .into_iter()
            .flatten()
            .flat_map(move |&n| std::iter::once(n).chain(walk_subgraph3(graph, &n.1))),
    )
}

// bag_quantities() reworks the ideas of walk_subgraph3 while multiplying appropriately.
// We need to multiply stuff together... if every "shiny gold" bag has two "dark red" bags,
// and those have three "light magenta" bags, then we have 2*3 = 6 "light magenta" bags.
fn bag_quantities<'iter, 'elems: 'iter>(
    graph: &'iter Rules<'elems>,
    root: &(&'iter str, &'iter str),
) -> Box<dyn Iterator<Item = usize> + 'iter> {
    Box::new(
        graph
            .get_vec(root)
            .into_iter()
            .flatten()
            .flat_map(move |&(qt, n)| {
                std::iter::once(qt).chain(bag_quantities(graph, &n).map(move |x| x * qt))
            }),
    )
}

fn main() {
    let rules = parse_rules(include_str!("input.txt"));
    //print!("{}", FormattedRules(rules));

    let needle = &("shiny", "gold");
    let colors_that_contain_shiny_gold: Vec<_> = rules
        .keys()
        // shiny gold bags are already shiny gold, we're not interested
        // in what they can contain (as per the example)
        .filter(|&k| k != needle)
        .filter(|&k| subgraph_contains(&rules, k, needle))
        .collect();
    println!("{:?}", colors_that_contain_shiny_gold);
    println!();

    let rev_rules = reverse_graph(&rules);
    /*
    let colors_that_contain_shiny_gold2 = walk_subgraph(&rev_rules, &("shiny", "gold"));
    println!("  {:?}", colors_that_contain_shiny_gold2);
    let mut colors_that_contain_shiny_gold3 = Default::default();
    walk_subgraph1(
        &rev_rules,
        &("shiny", "gold"),
        &mut colors_that_contain_shiny_gold3,
    );
    println!("  {:?}", colors_that_contain_shiny_gold3);
    */
    let answer1 = walk_subgraph2(&rev_rules, &needle).unique().count();
    println!("Part 1:");
    println!("  {} colors can contain {:?} bags", answer1, needle);

    let root = ("shiny", "gold");
    let answer2: usize = bag_quantities(&rules, &root).sum();
    println!("Part 2:");
    println!("  you must buy {} bags to fill a  {:?} bag", answer2, root);
}

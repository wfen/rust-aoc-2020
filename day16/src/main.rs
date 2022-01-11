use std::collections::{HashMap, HashSet};
use std::ops::RangeInclusive;
use parser::*;

// --- model

#[derive(Debug, Eq, PartialEq)]
struct Ranges(Vec<RangeInclusive<i64>>);

type FieldRanges = HashMap<String, Ranges>;
type Ticket = Vec<i64>;

#[derive(Debug, Eq, PartialEq)]
struct TicketData {
    field_ranges: FieldRanges,
    your_ticket: Ticket,
    nearby_tickets: Vec<Ticket>
}

impl Ranges {
    fn contains(&self, value: &i64) -> bool {
        self.0.iter().any(|r| r.contains(value))
    }
}

impl TicketData {
    fn is_invalid_value_for_field(&self, value: &i64, field: &str) -> bool {
        self.field_ranges.get(field)
            .map(|r| !r.contains(value))
            .unwrap()
    }

    fn is_invalid_value_for_any_field(&self, value: &i64) -> bool {
        self.field_ranges.values().all(|r| !r.contains(value))
    }

    fn ticket_errors(&self, ticket: &Ticket) -> i64 {
        ticket.iter()
            .filter(|value| self.is_invalid_value_for_any_field(value))
            .sum()
    }

    fn ticket_has_invalid_fields(&self, ticket: &Ticket) -> bool {
        ticket.iter().any(|value| self.is_invalid_value_for_any_field(value))
    }

    fn ticket_scanning_error_rate(&self) -> i64 {
        self.nearby_tickets.iter()
            .map(|ticket| self.ticket_errors(ticket))
            .sum()
    }

    fn valid_tickets<'a>(&'a self) -> impl Iterator<Item = &'a Ticket> + 'a {
        self.nearby_tickets.iter()
            .filter(move |ticket| !self.ticket_has_invalid_fields(ticket))
    }

    fn find_field_indices(&self) -> HashMap<String, usize> {
        let mut matcher = FieldMatcher::new(self);

        self.valid_tickets().for_each(|ticket| {
            matcher.eliminate_indices_for_ticket(
                ticket,
                |value, field_name| self.is_invalid_value_for_field(value, field_name)
            );
        });


        while !matcher.is_fully_determined() {
            matcher.eliminate_determined_indices();
        }

        matcher.flatten()
    }
}

struct FieldMatcher {
    ordered_fields: Vec<String>,
    possible_indices: HashMap<String, HashSet<usize>>
}

impl FieldMatcher {
    fn new(ticket_data: &TicketData) -> Self {
        let mut ordered_fields: Vec<String> = ticket_data.field_ranges.keys().cloned().collect();
        ordered_fields.sort();

        let all_indices: HashSet<usize> = (0..ticket_data.your_ticket.len()).collect();

        let possible_indices: HashMap<String, HashSet<usize>> = ticket_data.field_ranges.iter()
            .map(|(name, _)| (name.clone(), all_indices.clone()))
            .collect();

        FieldMatcher {
            ordered_fields,
            possible_indices
        }
    }

    fn eliminate_indices_for_ticket<F>(&mut self, ticket: &Ticket, is_invalid: F)
        where F: Fn(&i64, &str) -> bool
    {
        ticket.iter().enumerate()
            .for_each(|(index, value)| {
                self.possible_indices.iter_mut().for_each(|(field_name, indices)| {
                    if is_invalid(value, field_name) {
                        indices.remove(&index);
                    }
                })
            })
    }

    fn eliminate_determined_indices(&mut self) {
        let determined: HashSet<usize> =
            self.possible_indices.values()
                .filter(|ns| ns.len() == 1)
                .flat_map(|ns| ns.iter().cloned())
                .collect();

        self.possible_indices.values_mut()
            .filter(|ns| ns.len() > 1)
            .for_each(|ns| *ns = ns.difference(&determined).cloned().collect());
    }

    fn is_fully_determined(&self) -> bool {
        self.possible_indices.values().all(|ns| ns.len() == 1)
    }

    fn flatten(&self) -> HashMap<String, usize> {
        self.possible_indices.iter()
            .map(|(name, ns)| (name.clone(), *ns.iter().next().unwrap()))
            .collect()
    }

    fn debug(&self) {
        self.ordered_fields.iter().for_each(|f| {
            let mut ns: Vec<&usize> = self.possible_indices.get(f).unwrap().iter().collect();
            ns.sort();
            println!("{:20} -> {:?}", f, ns);
        });
        println!();
    }
}


// --- parser

fn parse_input(input: &str) -> ParseResult<TicketData> {
    let range = pair(
        left(integer, match_literal("-")),
        integer,
        |min, max| (min..=max)
    );

    let ranges = range
        .sep_by(whitespace_wrap(match_literal("or")))
        .map(|rs| Ranges(rs));

    let field_name = one_or_more(any_char.pred(|c| *c != ':'))
        .map(|cs| cs.iter().collect());

    let field_range = tuple2(
        left(field_name, match_literal(":")),
        whitespace_wrap(ranges)
    );

    let csv = integer.sep_by(match_literal(","));

    let your_ticket = right(
        whitespace_wrap(match_literal("your ticket:")),
        csv.clone()
    );

    let nearby_tickets = right(
        whitespace_wrap(match_literal("nearby tickets:")),
        one_or_more(whitespace_wrap(csv))
    );

    let ticket_data = tuple3(one_or_more(field_range), your_ticket, nearby_tickets)
        .map(|(field_ranges, your_ticket, nearby_tickets)| TicketData {
            field_ranges: field_ranges.into_iter().collect(),
            your_ticket,
            nearby_tickets
        });

    ticket_data.parse(input)
}

// --- problems

fn part1(ticket_data: &TicketData) -> i64 {
    ticket_data.ticket_scanning_error_rate()
}

fn part2(ticket_data: &TicketData) -> i64 {
    let indices = ticket_data.find_field_indices();

    let values: Vec<&i64> = indices.iter()
        .filter(|(name, _)| name.starts_with("departure"))
        .map(|(_, index)| ticket_data.your_ticket.get(*index).unwrap())
        .collect();

    assert_eq!(values.len(), 6);

    values.into_iter().product()
}

fn main() {
    let input = include_str!("input.txt");
    let ticket_data = parse_input(&input).unwrap().1;

    println!("part 1 {:?}", part1(&ticket_data));
    println!("part 2 {:?}", part2(&ticket_data));
}


#[cfg(test)]
#[macro_use] extern crate maplit;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> TicketData {
        TicketData {
            field_ranges: hashmap![
                "class".to_string() => Ranges(vec![1..=3, 5..=7]),
                "row".to_string() => Ranges(vec![6..=11, 33..=44]),
                "seat".to_string() => Ranges(vec![13..=40, 45..=50])
            ],
            your_ticket: vec![7, 1, 14],
            nearby_tickets: vec![
                vec![7 ,3, 47],
                vec![40, 4, 50],
                vec![55, 2, 20],
                vec![38, 6, 12]
            ]
        }
    }

    #[test]
    fn test_parser() {
        let ticket_data = parse_input(
            "class: 1-3 or 5-7
             row: 6-11 or 33-44
             seat: 13-40 or 45-50

             your ticket:
             7,1,14

             nearby tickets:
             7,3,47
             40,4,50
             55,2,20
             38,6,12"
        );

        assert_eq!(ticket_data, Ok(("", sample_data())));
    }

    #[test]
    fn test_ticket_scanning_error_rate() {
        assert_eq!(sample_data().ticket_scanning_error_rate(), 71);
    }

    #[test]
    fn test_find_field_indices() {
        let indices = sample_data().find_field_indices();
        assert_eq!(indices, hashmap![
            "row".to_string() => 0,
            "class".to_string() => 1,
            "seat".to_string() => 2
        ]);
    }
}

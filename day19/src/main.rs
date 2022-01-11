use std::collections::HashMap;
use std::iter::{empty, once};
use parser::*;

// --- model

type RuleID = usize;

#[derive(Debug, Clone, PartialEq)]
enum Rule {
    MatchChar(char),
    Sequence(Vec<RuleID>),
    Alternative(Vec<RuleID>, Vec<RuleID>)
}

#[derive(Debug, PartialEq)]
struct Rules {
    rules: HashMap<RuleID, Rule>
}

type MatchResult<'a> = Box<dyn Iterator<Item = &'a str> + 'a>;

impl Rules {
    fn get(&self, id: &RuleID) -> &Rule {
        self.rules.get(id).unwrap()
    }

    fn match_seq_tail_recursive<'a>(&'a self, seq: &'a [RuleID], input: &'a str) -> MatchResult<'a> {
        let mut remaining: Vec<&str> = self.match_seq_non_recursive(seq, input).collect();
        let mut results: MatchResult<'a> = Box::new(empty());
        while !remaining.is_empty() {
            let next_remaining = remaining.iter().flat_map(|r|
                self.match_seq_non_recursive(seq, r)
            ).collect();

            results = Box::new(results.chain(remaining.into_iter()));
            remaining = next_remaining;
        }
        results
    }

    fn match_seq_non_recursive<'a>(&'a self, seq: &'a [RuleID], input: &'a str) -> MatchResult<'a> {
        seq.iter().fold(
            Box::new(once(input)),
            |remainings, rule| {
                Box::new(remainings.flat_map(move |remaining|
                    self.match_rule(rule, remaining)
                ))
            }
        )
    }

    fn match_seq<'a>(&'a self, id: &RuleID, seq: &'a [RuleID], input: &'a str) -> MatchResult<'a> {
        if seq.last() == Some(id) {
            self.match_seq_tail_recursive(&seq[0..seq.len()-1], input)
        } else {
            self.match_seq_non_recursive(seq, input)
        }
    }

    fn match_rule<'a>(&'a self, id: &RuleID, input: &'a str) -> MatchResult<'a> {
        match self.get(id) {
            Rule::MatchChar(c) => {
                if input.chars().next() == Some(*c) {
                    Box::new(once(&input[c.len_utf8()..]))
                } else {
                    Box::new(empty())
                }
            }

            Rule::Sequence(rs) => {
                self.match_seq(id, rs, input)
            }

            Rule::Alternative(xs, ys) => {
                let mut r = self.match_seq(id, xs, input).peekable();
                if r.peek().is_some() {
                    Box::new(r)
                } else {
                    self.match_seq(id, ys, input)
                }
            }
        }
    }

    fn match_all<'a>(&self, input: &'a str) -> Result<(), &'a str> {
        let mut r = self.match_rule(&0, input);
        match r.next() {
            None => Err("no match"),
            Some(s) if s.is_empty() => Ok(()),
            _ => Err("extra unmatched input")
        }
    }

    fn apply_modification(&mut self) {
        // self.rules.insert(8, Rule::OneOrMore(42));
        self.rules.insert(8, Rule::Alternative(vec![42, 8], vec![42]));
        self.rules.insert(11, Rule::Alternative(vec![42, 31], vec!(42, 11, 31)));
    }
}

// --- parser

fn parse_rules(input: &str) -> ParseResult<Rules> {
    let rule_id = integer.map(|i| i as RuleID);
    let space = match_literal(" ");

    let match_char = any_char
        .between(match_literal(" \""), match_literal("\""))
        .map(Rule::MatchChar);

    let raw_sequence = one_or_more(right(space, rule_id.clone())).boxed();
    let sequence = raw_sequence.clone().map(Rule::Sequence);

    let alternative = pair(left(raw_sequence.clone(), match_literal(" |")), raw_sequence,
                           |a, b| Rule::Alternative(a, b)
    );

    let rule = pair(
        left(rule_id, match_literal(":")),
        match_char.or(alternative).or(sequence),
        |id, def| (id, def)
    );

    let rules = one_or_more(whitespace_wrap(rule))
        .map(|rs| Rules {
            rules: rs.into_iter().collect()
        });

    rules.parse(input)
}

// -- problems

fn count_valid_messages(rules: &Rules, messages: &[&str]) -> usize {
    messages.iter().filter_map(|m| rules.match_all(m).ok()).count()
}

fn main() {
    let input = include_str!("input.txt");

    let mut sections = input.split("\n\n");
    let mut rules = parse_rules(sections.next().unwrap()).unwrap().1;
    let messages: Vec<&str> = sections.next().unwrap().lines().collect();

    println!("part 1 {}", count_valid_messages(&rules, &messages));

    rules.apply_modification();
    println!("part 2 {}", count_valid_messages(&rules, &messages));
}

#[cfg(test)]
#[macro_use] extern crate maplit;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_rules() -> Rules {
        use Rule::*;
        Rules {
            rules: hashmap![
                0 => Sequence(vec![4, 1, 5]),
                1 => Alternative(vec![2, 3], vec![3, 2]),
                2 => Alternative(vec![4, 4], vec![5, 5]),
                3 => Alternative(vec![4, 5], vec![5, 4]),
                4 => MatchChar('a'),
                5 => MatchChar('b')
            ]
        }
    }

    fn part2_sample_rules() -> Rules {
        parse_rules(
            "42: 9 14 | 10 1
             9: 14 27 | 1 26
             10: 23 14 | 28 1
             1: \"a\"
             11: 42 31
             5: 1 14 | 15 1
             19: 14 1 | 14 14
             12: 24 14 | 19 1
             16: 15 1 | 14 14
             31: 14 17 | 1 13
             6: 14 14 | 1 14
             2: 1 24 | 14 4
             0: 8 11
             13: 14 3 | 1 12
             15: 1 | 14
             17: 14 2 | 1 7
             23: 25 1 | 22 14
             28: 16 1
             4: 1 1
             20: 14 14 | 1 15
             3: 5 14 | 16 1
             27: 1 6 | 14 18
             14: \"b\"
             21: 14 1 | 1 14
             25: 1 1 | 1 14
             22: 14 14
             8: 42
             26: 14 22 | 1 20
             18: 15 15
             7: 14 5 | 1 21
             24: 14 1
").unwrap().1
    }

    fn part2_sample_rules_modified() -> Rules {
        let mut rules = part2_sample_rules();
        rules.apply_modification();
        rules
    }

    fn part2_input() -> impl Iterator<Item = &'static str> {
        "abbbbbabbbaaaababbaabbbbabababbbabbbbbbabaaaa
bbabbbbaabaabba
babbbbaabbbbbabbbbbbaabaaabaaa
aaabbbbbbaaaabaababaabababbabaaabbababababaaa
bbbbbbbaaaabbbbaaabbabaaa
bbbababbbbaaaaaaaabbababaaababaabab
ababaaaaaabaaab
ababaaaaabbbaba
baabbaaaabbaaaababbaababb
abbbbabbbbaaaababbbbbbaaaababb
aaaaabbaabaaaaababaa
aaaabbaaaabbaaa
aaaabbaabbaaaaaaabbbabbbaaabbaabaaa
babaaabbbaaabaababbaabababaaab
aabbbbbaabbbaaaaaabbbbbababaaaaabbaaabba".lines()
    }

    #[test]
    fn test_parser() {
        let rules = parse_rules(
            "0: 4 1 5
             1: 2 3 | 3 2
             2: 4 4 | 5 5
             3: 4 5 | 5 4
             4: \"a\"
             5: \"b\""
        );

        assert_eq!(rules, Ok(("", sample_rules())));
    }

    #[test]
    fn test_matcher_success() {
        let rules = sample_rules();

        let result: Vec<&str> = rules.match_rule(&0, "ababbb").collect();
        assert_eq!(result, vec![""]);

        let result: Vec<&str> = rules.match_rule(&0, "abbbab").collect();
        assert_eq!(result, vec![""]);

        let result: Vec<&str> = rules.match_rule(&0, "aaaabbb").collect();
        assert_eq!(result, vec![("b")]);
    }

    #[test]
    fn test_matcher_failure() {
        let rules = sample_rules();
        assert_eq!(rules.match_rule(&0, "bababa").next(), None);
        assert_eq!(rules.match_rule(&0, "aaabbb").next(), None);
    }

    #[test]
    fn test_match_all() {
        let rules = sample_rules();
        assert_eq!(rules.match_all("abbbab"), Ok(()));
        assert_eq!(rules.match_all("aaaabbb"), Err("extra unmatched input"));
    }

    #[test]
    fn test_part2_rules_without_modification() {
        let rules = part2_sample_rules();
        let messages = part2_input();
        assert_eq!(messages.filter_map(|m| rules.match_all(m).ok()).count(), 3);
    }

    #[test]
    fn test_part2_rules_with_modification() {
        let rules = part2_sample_rules_modified();
        let messages = part2_input();
        assert_eq!(messages.filter_map(|m| rules.match_all(m).ok()).count(), 12);
    }

    #[test]
    fn test_part2_rules_with_modification_individual_cases() {
        let rules = part2_sample_rules_modified();
        assert_eq!(rules.match_all("bbabbbbaabaabba"), Ok(()));
        assert_eq!(rules.match_all("babbbbaabbbbbabbbbbbaabaaabaaa"), Ok(()));
        assert_eq!(rules.match_all("aaabbbbbbaaaabaababaabababbabaaabbababababaaa"), Ok(()));
        assert_eq!(rules.match_all("bbbbbbbaaaabbbbaaabbabaaa"), Ok(()));
        assert_eq!(rules.match_all("bbbababbbbaaaaaaaabbababaaababaabab"), Ok(()));
        assert_eq!(rules.match_all("ababaaaaaabaaab"), Ok(()));
        assert_eq!(rules.match_all("ababaaaaabbbaba"), Ok(()));
        assert_eq!(rules.match_all("baabbaaaabbaaaababbaababb"), Ok(()));
        assert_eq!(rules.match_all("abbbbabbbbaaaababbbbbbaaaababb"), Ok(()));
        assert_eq!(rules.match_all("aaaaabbaabaaaaababaa"), Ok(()));
        assert_eq!(rules.match_all("aaaabbaabbaaaaaaabbbabbbaaabbaabaaa"), Ok(()));
        assert_eq!(rules.match_all("aabbbbbaabbbaaaaaabbbbbababaaaaabbaaabba"), Ok(()));
    }
}
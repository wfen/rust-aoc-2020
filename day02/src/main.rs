use std::ops::RangeInclusive;
use std::fmt::Debug;


fn main() -> anyhow::Result<()> {
    // lines() works like split('\n') but it also supports \r\n (CRLF) Windows-style line endings
    // don't bother collecting into a result... just answer the question using filter and count
    let count1 = include_str!("input.txt")
        .lines()
        .map(parse_line1)
        .map(Result::unwrap)
        .filter(|(policy, password)| policy.is_valid(password))
        .count();
    println!("Part 1:");
    println!("  {} passwords are valid", count1);

    let count2 = include_str!("input.txt")
        .lines()
        .map(parse_line2)
        .map(Result::unwrap)
        .filter(|(policy, password)| policy.is_valid(password))
        .count();
    println!("Part 2:");
    println!("  {} passwords are valid", count2);

    Ok(())
}

// instead of implementing the PartialEq and Debug traits, we normally would just derive them
// https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros

struct PasswordPolicy1 {
    byte: u8,
    range: RangeInclusive<usize>,
}

impl PartialEq for PasswordPolicy1 {
    fn eq(&self, other: &Self) -> bool {
        self.byte == other.byte && self.range == other.range
    }
}

impl Debug for PasswordPolicy1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasswordPolicy")
            .field("byte", &self.byte)
            .field("range", &self.range)
            .finish()
    }
}

impl PasswordPolicy1 {
    fn is_valid(&self, password: &str) -> bool {
        // why .copied() ... password.as_bytes().iter() gives us an Iterator<Item = &u8>
        // u8 implements the Copy trait, so we don't need to worry about its ownership
        // iter.filter() when iter is an Iterator<Item = T>, passes &T.
        // we're filtering, avoid "consuming" the items... just read and decide on inclusion
        // filter(|&b| b == self.byte) ... is equivalent to ... filter(|b| *b == self.byte)
        self.range
            .contains(
                &password
                    .as_bytes()
                    .iter()
                    .copied()
                    .filter(|&b| b == self.byte)
                    .count(),
            )
    }
}

#[derive(PartialEq, Debug)]
struct PasswordPolicy2 {
    byte: u8,
    positions: [usize; 2],
}

impl PasswordPolicy2 {
    fn is_valid(&self, password: &str) -> bool {
        // why .copied() ... password.as_bytes().iter() gives us an Iterator<Item = &u8>
        // u8 implements the Copy trait, so we don't need to worry about its ownership
        // iter.filter() when iter is an Iterator<Item = T>, passes &T.
        // we're filtering, avoid "consuming" the items... just read and decide on inclusion
        // filter(|&b| b == self.byte) ... is equivalent to ... filter(|b| *b == self.byte)
        self.positions
            .iter()
            .copied()
            .filter(|&index| password.as_bytes()[index] == self.byte)
            .count()
            == 1
    }
}

fn parse_line1(s: &str) -> anyhow::Result<(PasswordPolicy1, &str)> {
    peg::parser! {
        grammar parser() for str {
            rule number() -> usize
                = n:$(['0'..='9']+) { n.parse().unwrap() }

            rule range() -> RangeInclusive<usize>
                = min:number() "-" max:number() { min..=max }

            rule byte() -> u8
                = letter:$(['a'..='z']) { letter.as_bytes()[0] }

            rule password() -> &'input str
                = letters:$([_]*) { letters }

            pub(crate) rule line() -> (PasswordPolicy1, &'input str)
                = range:range() " " byte:byte() ": " password:password() {
                    (PasswordPolicy1 { range, byte }, password)
            }
        }
    }

    Ok(parser::line(s)?)
}

fn parse_line2(s: &str) -> anyhow::Result<(PasswordPolicy2, &str)> {
    peg::parser! {
        grammar parser() for str {
            rule number() -> usize
                = n:$(['0'..='9']+) { n.parse().unwrap() }

            // Positions are 1-based indices in the input
            rule position() -> usize
                = n:number() { n - 1 }

            rule positions() -> [usize; 2]
                // now using `position()` rather than `number()` (giving 0-based values)
                = first:position() "-" second:position() { [first, second] }

            rule byte() -> u8
                = letter:$(['a'..='z']) { letter.as_bytes()[0] }

            rule password() -> &'input str
                = letters:$([_]*) { letters }

            pub(crate) rule line() -> (PasswordPolicy2, &'input str)
                = positions:positions() " " byte:byte() ": " password:password() {
                    (PasswordPolicy2 { positions, byte }, password)
            }
        }
    }

    Ok(parser::line(s)?)
}

#[cfg(test)]
mod tests {
    use super::PasswordPolicy1;

    #[test]
    fn test_is_valid1() {
        let pp = PasswordPolicy1 {
            range: 1..=3,
            byte: b'a',
        };
        assert!(!pp.is_valid("zeus"), "no 'a's");
        assert!(pp.is_valid("hades"), "single 'a's");
        assert!(pp.is_valid("banana"), "three 'a's");
        assert!(!pp.is_valid("aaaah"), "too many 'a's");
    }

    use super::parse_line1;

    #[test]
    fn test_parse1() {
        assert_eq!(
            parse_line1("1-3 a: banana").unwrap(),
            (
                PasswordPolicy1 {
                    range: 1..=3,
                    byte: b'a',
                },
                "banana"
            )
        );

        /*
        // these checks are specific to our manual parser's error messages
        assert_eq!(
            parse_line1("1-3 a").unwrap_err().to_string(),
            "expected password"
        );
        assert_eq!(
            parse_line1("1-3 : banana").unwrap_err().to_string(),
            "expected password policy byte to be exactly 1 byte"
        );
        */

        // feel free to add more tests!
    }

    use super::PasswordPolicy2;

    #[test]
    fn test_is_valid2() {
        let pp = PasswordPolicy2 {
            positions: [0, 2], // now 0-based
            byte: b'a',
        };
        assert!(pp.is_valid("abcde"), "'a' in position 1");
        assert!(pp.is_valid("bcade"), "'a' in position 3");
        assert!(!pp.is_valid("food"), "no 'a' whatsoever");
        assert!(!pp.is_valid("abacus"), "'a' in both positions");
    }

    use super::parse_line2;

    #[test]
    fn test_parse2() {
        assert_eq!(
            parse_line2("1-3 a: banana").unwrap(),
            (
                PasswordPolicy2 {
                    positions: [0, 2], // now 0-based
                    byte: b'a',
                },
                "banana"
            )
        );

        /*
        // these checks are specific to our manual parser's error messages
        assert_eq!(
            parse_line1("1-3 a").unwrap_err().to_string(),
            "expected password"
        );
        assert_eq!(
            parse_line1("1-3 : banana").unwrap_err().to_string(),
            "expected password policy byte to be exactly 1 byte"
        );
        */

        // feel free to add more tests!
    }
}

/*
// Manually parsing lines versus leveraging a parser generator (i.e. nom, peg)

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("expected {0}")]
    Expected(&'static str),
}

fn parse_line1(s: &str) -> anyhow::Result<(PasswordPolicy1, &str)> {
    let (policy, password) = {
        let mut tokens = s.split(':');
        (
            tokens
                .next()
                .ok_or(ParseError::Expected("password policy"))?,
            tokens
                .next()
                .ok_or(ParseError::Expected("password"))?
                .trim(),
            )
    };

    let (range, byte) = {
        let mut tokens = policy.split(' ');
        (
            tokens
                .next()
                .ok_or(ParseError::Expected("policy range"))?,
            tokens
                .next()
                .ok_or(ParseError::Expected("policy byte"))?,
        )
    };

    let byte = if byte.as_bytes().len() == 1 {
        byte.as_bytes()[0]
    } else {
        return Err(ParseError::Expected("password policy byte to be exactly 1 byte").into())
    };

    let (min, max) = {
        let mut tokens = range.split('-');
        (
            tokens
                .next()
                .ok_or(ParseError::Expected("policy range (lower bound)"))?,
            tokens
                .next()
                .ok_or(ParseError::Expected("policy range (upper bound)"))?,
            )
    };

    let range = (min.parse()?)..=(max.parse()?);

    Ok((PasswordPolicy1 { range, byte }, password))
}
*/

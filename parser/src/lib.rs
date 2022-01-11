use std::rc::Rc;


pub type ParseResult<'a, Output> = Result<(&'a str, Output), &'a str>;

pub trait Parser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output>;

    fn boxed(self) -> BoxedParser<'a, Output>
        where
            Self: Sized + 'a,
            Output: 'a
    {
        BoxedParser::new(self)
    }

    fn map<F, NewOutput>(self, map_fn: F) -> BoxedParser<'a, NewOutput>
        where
            Self: Sized + 'a,
            Output: 'a,
            NewOutput: 'a,
            F: Fn(Output) -> NewOutput + 'a
    {
        BoxedParser::new(map(self, map_fn))
    }

    fn means<NewOutput>(self, value: NewOutput) -> BoxedParser<'a, NewOutput>
        where
            Self: Sized + 'a,
            Output: 'a,
            NewOutput: Copy + 'a
    {
        BoxedParser::new(map(self, move |_| value))
    }

    fn pred<F>(self, pred_fn: F) -> BoxedParser<'a, Output>
        where
            Self: Sized + 'a,
            Output: 'a,
            F: Fn(&Output) -> bool + 'a
    {
        BoxedParser::new(pred(self, pred_fn))
    }

    fn and_then<F, NextP, NewOutput>(self, f: F) -> BoxedParser<'a, NewOutput>
        where
            Self: Sized + 'a,
            Output: 'a,
            NewOutput: 'a,
            NextP: Parser<'a, NewOutput> + 'a,
            F: Fn(Output) -> NextP + 'a
    {
        BoxedParser::new(and_then(self, f))
    }

    fn between<PX, PY, RX, RY>(self, before: PX, after: PY) -> BoxedParser<'a, Output>
        where
            Self: Sized + 'a,
            Output: 'a,
            RX: 'a,
            RY: 'a,
            PX: Parser<'a, RX> + 'a,
            PY: Parser<'a, RY> + 'a
    {
        BoxedParser::new(left(right(before, self), after))
    }

    fn sep_by<PS, RS>(self, sep: PS) -> BoxedParser<'a, Vec<Output>>
        where
            Self: Sized + 'a,
            Output: 'a,
            RS: 'a,
            PS: Parser<'a, RS> + 'a
    {
        BoxedParser::new(sep_by(self, sep))
    }

    fn or<Alternate>(self, alt: Alternate) -> BoxedParser<'a, Output>
        where
            Self: Sized + 'a,
            Output: 'a,
            Alternate: Parser<'a, Output> + 'a
    {
        BoxedParser::new(either(self, alt))
    }

}

#[derive(Clone)]
pub struct BoxedParser<'a, Output>(Rc<dyn Parser<'a, Output> + 'a>);

impl<'a, F, Output> Parser<'a, Output> for F
    where
        F: Fn(&'a str) -> ParseResult<Output>
{
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self(input)
    }
}

impl<'a, Output> BoxedParser<'a, Output> {
    fn new<P>(parser: P) -> Self
        where
            P: Parser<'a, Output> + 'a
    {
        BoxedParser(Rc::new(parser))
    }
}

impl<'a, Output> Parser<'a, Output> for BoxedParser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self.0.parse(input)
    }

    fn boxed(self) -> BoxedParser<'a, Output> {
        self
    }
}

pub fn match_literal<'a>(expected: &'static str) -> impl Parser<'a, ()> {
    move |input: &'a str|
        match input.get(0..expected.len()) {
            Some(next) if next == expected => {
                Ok((&input[expected.len()..], ()))
            }
            _ => Err(input)
        }
}

pub fn identifier(input: &str) -> ParseResult<String> {
    let mut matched = String::new();
    let mut chars = input.chars();

    match chars.next() {
        Some(next) if next.is_alphabetic() => matched.push(next),
        _ => return Err(input)
    }

    while let Some(next) = chars.next() {
        if next.is_alphabetic() || next == '-' {
            matched.push(next);
        } else {
            break;
        }
    }

    let next_index = matched.len();
    Ok((&input[next_index..], matched))
}

pub fn word_ref(input: &str) -> ParseResult<&str> {
    let mut matched = 0;
    let mut chars = input.chars();

    match chars.next() {
        Some(next) if next.is_alphabetic() => matched += 1,
        _ => return Err(input)
    }

    while let Some(next) = chars.next() {
        if next.is_alphabetic() {
            matched += 1;
        } else {
            break;
        }
    }

    Ok((&input[matched..], &input[0..matched]))
}


pub fn pair<'a, P1, P2, R1, R2, F, R>(parser1: P1, parser2: P2, f: F) -> impl Parser<'a, R>
    where
        P1: Parser<'a, R1>,
        P2: Parser<'a, R2>,
        F: Fn(R1, R2) -> R
{
    move |input| {
        parser1.parse(input).and_then(|(next_input, result1)| {
            parser2.parse(next_input)
                .map(|(last_input, result2)| (last_input, f(result1, result2)))
        })
    }
}

pub fn tuple2<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, (R1, R2)>
    where
        P1: Parser<'a, R1>,
        P2: Parser<'a, R2>
{
    move |input| {
        parser1.parse(input).and_then(|(next_input, result1)|
            parser2.parse(next_input)
                .map(|(last_input, result2)| (last_input, (result1, result2)))
        )
    }
}

pub fn tuple3<'a, P1, P2, P3, R1, R2, R3>(parser1: P1, parser2: P2, parser3: P3) -> impl Parser<'a, (R1, R2, R3)>
    where
        P1: Parser<'a, R1>,
        P2: Parser<'a, R2>,
        P3: Parser<'a, R3>
{
    move |input| {
        parser1.parse(input).and_then(|(next_input, result1)|
            parser2.parse(next_input).and_then(|(next_input, result2)|
                parser3.parse(next_input)
                    .map(|(last_input, result3)| (last_input, (result1, result2, result3)))
            )
        )
    }
}

pub fn left<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R1>
    where
        P1: Parser<'a, R1>,
        P2: Parser<'a, R2>
{
    pair(parser1, parser2, |left, _| left)
}

pub fn right<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R2>
    where
        P1: Parser<'a, R1>,
        P2: Parser<'a, R2>
{
    pair(parser1, parser2, |_, right| right)
}

fn map<'a, P, F, A, B>(parser: P, map_fn: F) -> impl Parser<'a, B>
    where
        P: Parser<'a, A>,
        F: Fn(A) -> B
{
    move |input|
        parser.parse(input).map(|(next_input, result)|
            (next_input, map_fn(result)))
}

pub fn one_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
    where
        P: Parser<'a, A>
{
    move |mut input| {
        let mut result = Vec::new();

        if let Ok((next_input, first_item)) = parser.parse(input) {
            input = next_input;
            result.push(first_item);
        } else {
            return Err(input);
        }

        while let Ok((next_input, next_item)) = parser.parse(input) {
            input = next_input;
            result.push(next_item);
        }

        Ok((input, result))
    }
}

pub fn zero_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
    where
        P: Parser<'a, A>
{
    move |mut input| {
        let mut result = Vec::new();

        while let Ok((next_input, next_item)) = parser.parse(input) {
            input = next_input;
            result.push(next_item);
        }

        Ok((input, result))
    }
}

pub fn sep_by<'a, PA, A, PS, S>(parser: PA, sep_parser: PS) -> impl Parser<'a, Vec<A>>
    where
        PA: Parser<'a, A>,
        PS: Parser<'a, S>
{
    move |mut input| {
        let mut result = Vec::new();

        if let Ok((next_input, first_item)) = parser.parse(input) {
            input = next_input;
            result.push(first_item);
        } else {
            return Err(input);
        }

        loop {
            match sep_parser.parse(input) {
                // not matching the sep means end of the list
                Err(_) => {
                    return Ok((input, result))
                }
                // matching the sep means we must match the next item
                Ok((next_input, _)) => {
                    if let Ok((next_input, next_item)) = parser.parse(next_input) {
                        input = next_input;
                        result.push(next_item);
                    } else {
                        return Err(input);
                    }
                }
            }
        }
    }
}

pub fn any_char(input: &str) -> ParseResult<char> {
    match input.chars().next() {
        Some(next) => Ok((&input[next.len_utf8()..], next)),
        _ => Err(input)
    }
}

fn pred<'a, P, A, F>(parser: P, predicate: F) -> impl Parser<'a, A>
    where
        P: Parser<'a, A>,
        F: Fn(&A) -> bool
{
    move |input| {
        if let Ok((next_input, value)) = parser.parse(input) {
            if predicate(&value) {
                return Ok((next_input, value));
            }
        }
        Err(input)
    }
}

pub fn whitespace_char<'a>() -> impl Parser<'a, char> {
    pred(any_char, |c| c.is_whitespace())
}

pub fn space1<'a>() -> impl Parser<'a, Vec<char>> {
    one_or_more(whitespace_char())
}

pub fn space0<'a>() -> impl Parser<'a, Vec<char>> {
    zero_or_more(whitespace_char())
}

pub fn quoted_string<'a>() -> impl Parser<'a, String> {
    right(
        match_literal("\""),
        left(
            zero_or_more(any_char.pred(|c| *c != '"')),
            match_literal("\"")
        )
    )
        .map(|chars| chars.into_iter().collect())
}

pub fn either<'a, P1, P2, A>(parser1: P1, parser2: P2) -> impl Parser<'a, A>
    where
        P1: Parser<'a, A>,
        P2: Parser<'a, A>
{
    move |input|
        match parser1.parse(input) {
            ok@Ok(_) => ok,
            Err(_) => parser2.parse(input)
        }
}

pub fn one_of3<'a, P1, P2, P3, A>(p1: P1, p2: P2, p3: P3) -> impl Parser<'a, A>
    where
        P1: Parser<'a, A>,
        P2: Parser<'a, A>,
        P3: Parser<'a, A>
{
    either(either(p1, p2), p3)
}

pub fn one_of4<'a, P1, P2, P3, P4, A>(p1: P1, p2: P2, p3: P3, p4: P4) -> impl Parser<'a, A>
    where
        P1: Parser<'a, A>,
        P2: Parser<'a, A>,
        P3: Parser<'a, A>,
        P4: Parser<'a, A>
{
    either(either(p1, p2), either(p3, p4))
}

pub fn and_then<'a, P, F, A, B, NextP>(parser: P, f: F) -> impl Parser<'a, B>
    where
        P: Parser<'a, A>,
        NextP: Parser<'a, B>,
        F: Fn(A) -> NextP
{
    move |input|
        match parser.parse(input) {
            Ok((next_input, result)) => f(result).parse(next_input),
            Err(err) => Err(err)
        }
}

pub fn whitespace_wrap<'a, P, A>(parser: P) -> impl Parser<'a, A>
    where
        P: Parser<'a, A>
{
    right(space0(), left(parser, space0()))
}


pub fn integer(input: &str) -> ParseResult<i64> {
    let digit_as_num = any_char.pred(|c| c.is_digit(10)).map(|d| (d as i64) - 48);

    if let Ok((rest, first_digit)) = digit_as_num.parse(input) {
        let mut i = first_digit;
        let mut remainder = rest;
        while let Ok((rest, next_digit)) = digit_as_num.parse(remainder) {
            i = i * 10 + next_digit;
            remainder = rest;
        }
        Ok((remainder, i))
    } else {
        Err(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_parser_matches_string() {
        let parse_joe = match_literal("Hello Joe!");
        assert_eq!(
            Ok(("", ())),
            parse_joe.parse("Hello Joe!")
        );
    }

    #[test]
    fn literal_parser_returns_unparsed_input() {
        let parse_joe = match_literal("Hello Joe!");
        assert_eq!(
            Ok((" Hello Robert!", ())),
            parse_joe.parse("Hello Joe! Hello Robert!")
        );
    }

    #[test]
    fn literal_parser_fails_on_no_match() {
        let parse_joe = match_literal("Hello Joe!");
        assert_eq!(
            Err("Hello Mike!"),
            parse_joe.parse("Hello Mike!")
        );
    }

    #[test]
    fn identifier_parser_matches_identifier() {
        assert_eq!(
            Ok(("", "i-am-an-identifier".to_string())),
            identifier.parse("i-am-an-identifier")
        );
    }

    #[test]
    fn identifier_parser_returns_unparsed_input() {
        assert_eq!(
            Ok((" entirely an identifier", "not".to_string())),
            identifier.parse("not entirely an identifier")
        );
    }

    #[test]
    fn identifier_parser_fails_on_non_alphabetic_character() {
        assert_eq!(
            Err("!not at all an identifier"),
            identifier.parse("!not at all an identifier")
        );
    }

    #[test]
    fn means_combinator() {
        let parser = match_literal("foo").means("bar");
        assert_eq!(Ok(("", "bar")), parser.parse("foo"));
    }

    #[test]
    fn pair_combinator() {
        let tag_opener = right(match_literal("<"), identifier);
        assert_eq!(
            Ok(("/>", "my-first-element".to_string())),
            tag_opener.parse("<my-first-element/>")
        );
        assert_eq!(Err("oops"), tag_opener.parse("oops"));
        assert_eq!(Err("!oops"), tag_opener.parse("<!oops"));
    }

    #[test]
    fn one_or_more_combinator() {
        let parser = one_or_more(match_literal("ha"));
        assert_eq!(Ok(("", vec![(), (), ()])), parser.parse("hahaha"));
        assert_eq!(Err("ahah"), parser.parse("ahah"));
        assert_eq!(Err(""), parser.parse(""));
    }

    #[test]
    fn zero_or_more_combinator() {
        let parser = zero_or_more(match_literal("ha"));
        assert_eq!(Ok(("", vec![(), (), ()])), parser.parse("hahaha"));
        assert_eq!(Ok(("ahah", vec![])), parser.parse("ahah"));
        assert_eq!(Ok(("", vec![])), parser.parse(""));
    }

    #[test]
    fn predicate_combinator() {
        let parser = pred(any_char, |c| *c == 'o');
        assert_eq!(Ok(("mg", 'o')), parser.parse("omg"));
        assert_eq!(Err("lol"), parser.parse("lol"));
    }

    #[test]
    fn sep_by_combinator() {
        let parser = integer.sep_by(match_literal(","));
        assert_eq!(Ok(("", vec![1,2,3,4])), parser.parse("1,2,3,4"));
    }

    #[test]
    fn quoted_string_parser() {
        assert_eq!(
            Ok(("", "Hello Joe!".to_string())),
            quoted_string().parse("\"Hello Joe!\"")
        );
    }

    #[test]
    fn integer_parsre() {
        assert_eq!(
            Ok(("foo", 123)),
            integer.parse("123foo")
        );
    }
}

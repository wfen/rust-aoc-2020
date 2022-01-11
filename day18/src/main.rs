use parser::*;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Token {
    Num(i64),
    Add,
    Mul,
    Open,
    Close
}

fn tokenize(input: &str) -> ParseResult<Vec<Token>> {
    let token = whitespace_wrap(
        integer.map(Token::Num)
            .or(match_literal("+").means(Token::Add))
            .or(match_literal("*").means(Token::Mul))
            .or(match_literal("(").means(Token::Open))
            .or(match_literal(")").means(Token::Close))
    );

    one_or_more(token).parse(input)
}

fn shunting_yard<F>(tokens: &[Token], precedence: F) -> Vec<&Token>
    where
        F: Fn(&Token, &Token) -> bool
{
    let mut stack: Vec<&Token> = vec![];
    let mut result: Vec<&Token> = vec![];

    for token in tokens {
        match token {
            Token::Num(_) => {
                result.push(token)
            }

            Token::Add | Token::Mul => {
                while let Some(t) = stack.last() {
                    if *t == &Token::Add || *t == &Token::Mul && precedence(token, *t) {
                        result.push(*t);
                        stack.pop();
                    } else {
                        break;
                    }
                }
                stack.push(token)
            }

            Token::Open => {
                stack.push(token)
            }

            Token::Close => {
                while let Some(t) = stack.pop() {
                    if t == &Token::Open {
                        break
                    } else {
                        result.push(t);
                    }
                }
            }
        }
    }

    while let Some(t) = stack.pop() {
        result.push(t);
    }

    result
}

fn shunting_yard_v1(tokens: &[Token]) -> Vec<&Token> {
    shunting_yard(tokens, |_, _| true)
}

fn shunting_yard_v2(tokens: &[Token]) -> Vec<&Token> {
    shunting_yard(tokens, |t1, t2| !(t1 == &Token::Add && t2 == &Token::Mul))
}

fn eval_rp(tokens: &[&Token]) -> i64 {
    let mut stack: Vec<i64> = vec![];

    for token in tokens {
        match token {
            Token::Num(n) => {
                stack.push(*n)
            }

            Token::Add => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a + b);
            }

            Token::Mul => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a * b);
            }

            _ => panic!("shunting yard should remove all parens!")
        }
    }

    stack.pop().unwrap()
}

fn eval_v1(input: &str) -> i64 {
    let tokens = tokenize(input).unwrap().1;
    let rp = shunting_yard_v1(&tokens);
    eval_rp(&rp)
}

fn eval_v2(input: &str) -> i64 {
    let tokens = tokenize(input).unwrap().1;
    let rp = shunting_yard_v2(&tokens);
    eval_rp(&rp)
}

fn part1(input: &str) -> i64 {
    input.lines().map(eval_v1).sum()
}

fn part2(input: &str) -> i64 {
    input.lines().map(eval_v2).sum()
}

fn main() {
    let input = include_str!("input.txt");
    println!("part 1 {}", part1(&input));
    println!("part 2 {}", part2(&input));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        use Token::*;
        assert_eq!(tokenize("1 + 2 * (3+9)"), Ok(("", vec![
            Num(1), Add, Num(2), Mul, Open, Num(3), Add, Num(9), Close
        ])) );
    }

    #[test]
    fn test_shunting_yard_v1_simple_add() {
        use Token::*;
        let input = [Num(1), Add, Num(2)];
        assert_eq!(shunting_yard_v1(&input), vec![&Num(1), &Num(2), &Add])
    }

    #[test]
    fn test_shunting_yard_v1_with_parens() {
        use Token::*;
        let input = [Num(1), Add, Open, Num(2), Mul, Num(3), Close, Add, Num(7)];
        assert_eq!(shunting_yard_v1(&input), vec![&Num(1), &Num(2), &Num(3), &Mul, &Add, &Num(7), &Add])
    }

    #[test]
    fn test_eval_rp() {
        use Token::*;
        assert_eq!(eval_rp(&[&Num(1), &Num(2), &Num(3), &Mul, &Num(7), &Add, &Add]), 14);
    }

    #[test]
    fn test_eval_v1() {
        assert_eq!(eval_v1("2 * 3 + (4 * 5)"), 26);
        assert_eq!(eval_v1("5 + (8 * 3 + 9 + 3 * 4 * 3)"), 437);
        assert_eq!(eval_v1("5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))"), 12240);
        assert_eq!(eval_v1("((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2"), 13632);
    }

    #[test]
    fn test_eval_v2() {
        assert_eq!(eval_v2("1 + (2 * 3) + (4 * (5 + 6))"), 51);
        assert_eq!(eval_v2("2 * 3 + (4 * 5)"), 46);
        assert_eq!(eval_v2("5 + (8 * 3 + 9 + 3 * 4 * 3)"), 1445);
        assert_eq!(eval_v2("5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))"), 669060);
        assert_eq!(eval_v2("((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2"), 23340);
    }
}

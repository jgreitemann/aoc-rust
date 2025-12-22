use anyhow::{anyhow, bail};
use aoc_companion::prelude::*;
use aoc_utils::iter::IterUtils as _;
use itertools::Itertools;

pub(crate) struct Door {
    expressions: Vec<Vec<Token>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        input
            .lines()
            .map(|line| tokenize(line).try_collect())
            .try_collect()
            .map(|expressions| Door { expressions })
    }

    fn part1(&self) -> Result<u64> {
        self.expressions
            .iter()
            .map(parse_part1)
            .map_ok(|ast| ast.eval())
            .try_sum()
    }

    fn part2(&self) -> Result<u64> {
        self.expressions
            .iter()
            .map(parse_part2)
            .map_ok(|ast| ast.eval())
            .try_sum()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Token {
    LParen,
    RParen,
    Plus,
    Times,
    Number(u64),
}

fn tokenize(expr: &str) -> impl Iterator<Item = Result<Token>> {
    expr.bytes()
        .filter(|b| !b.is_ascii_whitespace())
        .map(|b| match b {
            b'(' => Ok(Token::LParen),
            b')' => Ok(Token::RParen),
            b'+' => Ok(Token::Plus),
            b'*' => Ok(Token::Times),
            b'0'..=b'9' => Ok(Token::Number((b - b'0') as u64)),
            0..128 => Err(anyhow!(
                "invalid token {:?}",
                char::from_u32(b as u32).unwrap()
            )),
            _ => Err(anyhow!("invalid token: non-ASCII character")),
        })
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Ast {
    Number(u64),
    Add(Box<Ast>, Box<Ast>),
    Mul(Box<Ast>, Box<Ast>),
}

impl Ast {
    fn eval(&self) -> u64 {
        match self {
            Ast::Number(n) => *n,
            Ast::Add(lhs, rhs) => lhs.eval() + rhs.eval(),
            Ast::Mul(lhs, rhs) => lhs.eval() * rhs.eval(),
        }
    }
}

fn parse_part1<'a, I: IntoIterator<Item = &'a Token>>(tokens: I) -> Result<Ast> {
    fn do_parse<'a, I: Iterator<Item = &'a Token>>(tokens: &mut I) -> Result<Ast> {
        let parse_operand = |tokens: &mut I| -> Result<Ast> {
            match tokens.next() {
                Some(Token::LParen) => do_parse(tokens),
                Some(Token::Number(n)) => Ok(Ast::Number(*n)),
                Some(Token::Plus) | Some(Token::Times) => {
                    Err(anyhow!("expected a number or expression, got an operator"))
                }
                Some(Token::RParen) | None => Err(anyhow!("unexpected EOL or ')'")),
            }
        };
        let mut ast = parse_operand(tokens)?;
        loop {
            ast = match tokens.next() {
                Some(Token::Plus) => Ast::Add(Box::new(ast), Box::new(parse_operand(tokens)?)),
                Some(Token::Times) => Ast::Mul(Box::new(ast), Box::new(parse_operand(tokens)?)),
                Some(Token::RParen) | None => return Ok(ast),
                _ => bail!("unexpected token"),
            }
        }
    }
    do_parse(&mut tokens.into_iter())
}

fn parse_part2<'a, I: IntoIterator<Item = &'a Token>>(tokens: I) -> Result<Ast> {
    fn do_parse<'a, I: Iterator<Item = &'a Token>>(tokens: &mut I) -> Result<Ast> {
        enum Parens {
            Yes(Ast),
            No(Ast),
        }

        let parse_operand = |tokens: &mut I| -> Result<Ast> {
            match tokens.next() {
                Some(Token::LParen) => do_parse(tokens),
                Some(Token::Number(n)) => Ok(Ast::Number(*n)),
                Some(Token::Plus) | Some(Token::Times) => {
                    Err(anyhow!("expected a number or expression, got an operator"))
                }
                Some(Token::RParen) | None => Err(anyhow!("unexpected EOL or ')'")),
            }
        };
        let mut ast = Parens::Yes(parse_operand(tokens)?);
        loop {
            ast = match (ast, tokens.next()) {
                (Parens::No(Ast::Mul(lhs, rhs)), Some(Token::Plus)) => Parens::No(Ast::Mul(
                    lhs,
                    Box::new(Ast::Add(rhs, Box::new(parse_operand(tokens)?))),
                )),
                (Parens::Yes(ast) | Parens::No(ast), Some(Token::Plus)) => {
                    Parens::No(Ast::Add(Box::new(ast), Box::new(parse_operand(tokens)?)))
                }
                (Parens::Yes(ast) | Parens::No(ast), Some(Token::Times)) => {
                    Parens::No(Ast::Mul(Box::new(ast), Box::new(parse_operand(tokens)?)))
                }
                (Parens::Yes(ast) | Parens::No(ast), Some(Token::RParen) | None) => return Ok(ast),
                _ => bail!("unexpected token"),
            }
        }
    }
    do_parse(&mut tokens.into_iter())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUTS: &[&str] = &[
        "1 + 2 * 3 + 4 * 5 + 6",
        "1 + (2 * 3) + (4 * (5 + 6))",
        "2 * 3 + (4 * 5)",
        "5 + (8 * 3 + 9 + 3 * 4 * 3)",
        "5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))",
        "((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2",
    ];

    macro_rules! ast {
        ($n:literal) => {
            Ast::Number($n)
        };
        (($lhs:tt + $rhs:tt)) => {
            Ast::Add(Box::new(ast!($lhs)), Box::new(ast!($rhs)))
        };
        (($lhs:tt * $rhs:tt)) => {
            Ast::Mul(Box::new(ast!($lhs)), Box::new(ast!($rhs)))
        };
    }

    fn example_asts_part1() -> Vec<Ast> {
        vec![
            ast!((((((1 + 2) * 3) + 4) * 5) + 6)),
            ast!(((1 + (2 * 3)) + (4 * (5 + 6)))),
            ast!(((2 * 3) + (4 * 5))),
            ast!((5 + (((((8 * 3) + 9) + 3) * 4) * 3))),
            ast!(((5 * 9) * (((((7 * 3) * 3) + 9) * 3) + ((8 + 6) * 4)))),
            ast!((((((((2 + 4) * 9) * (((6 + 9) * 8) + 6)) + 6) + 2) + 4) * 2)),
        ]
    }
    fn example_asts_part2() -> Vec<Ast> {
        vec![
            ast!((((1 + 2) * (3 + 4)) * (5 + 6))),
            ast!(((1 + (2 * 3)) + (4 * (5 + 6)))),
            ast!((2 * (3 + (4 * 5)))),
            ast!((5 + (((8 * ((3 + 9) + 3)) * 4) * 3))),
            ast!(((5 * 9) * (((7 * 3) * (3 + 9)) * (3 + ((8 + 6) * 4))))),
            ast!(((((((2 + 4) * 9) * (((6 + 9) * (8 + 6)) + 6)) + 2) + 4) * 2)),
        ]
    }

    const EXAMPLE_EXPRESSIONS: &[&[Token]] = &[
        &[
            Token::Number(1),
            Token::Plus,
            Token::Number(2),
            Token::Times,
            Token::Number(3),
            Token::Plus,
            Token::Number(4),
            Token::Times,
            Token::Number(5),
            Token::Plus,
            Token::Number(6),
        ],
        &[
            Token::Number(1),
            Token::Plus,
            Token::LParen,
            Token::Number(2),
            Token::Times,
            Token::Number(3),
            Token::RParen,
            Token::Plus,
            Token::LParen,
            Token::Number(4),
            Token::Times,
            Token::LParen,
            Token::Number(5),
            Token::Plus,
            Token::Number(6),
            Token::RParen,
            Token::RParen,
        ],
        &[
            Token::Number(2),
            Token::Times,
            Token::Number(3),
            Token::Plus,
            Token::LParen,
            Token::Number(4),
            Token::Times,
            Token::Number(5),
            Token::RParen,
        ],
        &[
            Token::Number(5),
            Token::Plus,
            Token::LParen,
            Token::Number(8),
            Token::Times,
            Token::Number(3),
            Token::Plus,
            Token::Number(9),
            Token::Plus,
            Token::Number(3),
            Token::Times,
            Token::Number(4),
            Token::Times,
            Token::Number(3),
            Token::RParen,
        ],
        &[
            Token::Number(5),
            Token::Times,
            Token::Number(9),
            Token::Times,
            Token::LParen,
            Token::Number(7),
            Token::Times,
            Token::Number(3),
            Token::Times,
            Token::Number(3),
            Token::Plus,
            Token::Number(9),
            Token::Times,
            Token::Number(3),
            Token::Plus,
            Token::LParen,
            Token::Number(8),
            Token::Plus,
            Token::Number(6),
            Token::Times,
            Token::Number(4),
            Token::RParen,
            Token::RParen,
        ],
        &[
            Token::LParen,
            Token::LParen,
            Token::Number(2),
            Token::Plus,
            Token::Number(4),
            Token::Times,
            Token::Number(9),
            Token::RParen,
            Token::Times,
            Token::LParen,
            Token::Number(6),
            Token::Plus,
            Token::Number(9),
            Token::Times,
            Token::Number(8),
            Token::Plus,
            Token::Number(6),
            Token::RParen,
            Token::Plus,
            Token::Number(6),
            Token::RParen,
            Token::Plus,
            Token::Number(2),
            Token::Plus,
            Token::Number(4),
            Token::Times,
            Token::Number(2),
        ],
    ];

    #[test]
    fn example_expressions_are_tokenized() {
        itertools::assert_equal(
            EXAMPLE_INPUTS
                .iter()
                .map(|line| -> Vec<Token> { tokenize(line).try_collect().unwrap() }),
            EXAMPLE_EXPRESSIONS.iter().cloned(),
        );
    }

    #[test]
    fn example_expressions_are_parsed_for_part1() {
        itertools::assert_equal(
            EXAMPLE_EXPRESSIONS
                .iter()
                .map(|tokens| parse_part1(*tokens).unwrap()),
            example_asts_part1(),
        );
    }

    #[test]
    fn example_expressions_are_parsed_for_part2() {
        itertools::assert_equal(
            EXAMPLE_EXPRESSIONS
                .iter()
                .map(|tokens| parse_part2(*tokens).unwrap()),
            example_asts_part2(),
        );
    }

    #[test]
    fn evaluate_example_expressions() {
        itertools::assert_equal(
            example_asts_part1().iter().map(Ast::eval),
            [71, 51, 26, 437, 12240, 13632],
        );
        itertools::assert_equal(
            example_asts_part2().iter().map(Ast::eval),
            [231, 51, 46, 1445, 669060, 23340],
        );
    }
}

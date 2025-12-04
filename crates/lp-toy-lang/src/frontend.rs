//! AST and parser for the toy language.

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
use std::{boxed::Box, format, string::String, string::ToString, vec::Vec};
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, format, string::String, string::ToString, vec::Vec};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, multispace0},
    combinator::{map, verify},
    multi::{many0, separated_list0},
    sequence::{delimited, terminated, tuple},
    IResult,
};

/// The AST node for expressions.
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(String),
    Identifier(String),
    Assign(String, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Ne(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Ge(Box<Expr>, Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    IfElse(Box<Expr>, Vec<Expr>, Vec<Expr>),
    WhileLoop(Box<Expr>, Vec<Expr>),
    Call(String, Vec<Expr>),
    GlobalDataAddr(String),
}

/// Parse whitespace and discard result
fn blank(input: &str) -> IResult<&str, ()> {
    map(multispace0, |_| ())(input)
}

#[inline]
fn identifier_pred(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

#[inline]
fn verify_identifier(s: &str) -> bool {
    !s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(true)
}

/// Parse an identifier: [a-zA-Z_][a-zA-Z0-9_]*
fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(
        verify(take_while1(identifier_pred), verify_identifier),
        |s: &str| s.to_string(),
    )(input)
}

/// Parse a literal: [0-9]+
fn parse_literal(input: &str) -> IResult<&str, String> {
    map(take_while1(|c: char| c.is_ascii_digit()), |s: &str| {
        s.to_string()
    })(input)
}

/// Parse a global data address: &identifier
fn parse_global_data_addr(input: &str) -> IResult<&str, Expr> {
    map(
        tuple((terminated(char('&'), blank), parse_identifier)),
        |(_, name)| Expr::GlobalDataAddr(name),
    )(input)
}

/// Parse a primary expression (identifiers, literals, calls, parenthesized expressions)
fn parse_primary(input: &str) -> IResult<&str, Expr> {
    alt((
        // Global data address: &identifier
        parse_global_data_addr,
        // Function call: identifier(args)
        map(
            tuple((
                terminated(parse_identifier, blank),
                terminated(char('('), blank),
                separated_list0(
                    terminated(char(','), blank),
                    terminated(parse_expression, blank),
                ),
                terminated(char(')'), blank),
            )),
            |(name, _, args, _)| Expr::Call(name, args),
        ),
        // Identifier
        map(terminated(parse_identifier, blank), Expr::Identifier),
        // Literal
        map(terminated(parse_literal, blank), Expr::Literal),
    ))(input)
}

/// Parse multiplicative operators (*, /)
fn parse_multiplicative(input: &str) -> IResult<&str, Expr> {
    let (input, first) = parse_primary(input)?;

    let mut result = first;
    let mut input = input;

    loop {
        let result_opt = alt((terminated(tag("*"), blank), terminated(tag("/"), blank)))(input);

        let (new_input, op) = match result_opt {
            Ok(r) => r,
            Err(_) => break,
        };

        let (new_input, rhs) = terminated(parse_primary, blank)(new_input)?;

        result = match op {
            "*" => Expr::Mul(Box::new(result), Box::new(rhs)),
            "/" => Expr::Div(Box::new(result), Box::new(rhs)),
            _ => unreachable!(),
        };

        input = new_input;
    }

    Ok((input, result))
}

/// Parse additive operators (+, -)
fn parse_additive(input: &str) -> IResult<&str, Expr> {
    let (input, first) = parse_multiplicative(input)?;

    let mut result = first;
    let mut input = input;

    loop {
        let result_opt = alt((terminated(tag("+"), blank), terminated(tag("-"), blank)))(input);

        let (new_input, op) = match result_opt {
            Ok(r) => r,
            Err(_) => break,
        };

        let (new_input, rhs) = terminated(parse_multiplicative, blank)(new_input)?;

        result = match op {
            "+" => Expr::Add(Box::new(result), Box::new(rhs)),
            "-" => Expr::Sub(Box::new(result), Box::new(rhs)),
            _ => unreachable!(),
        };

        input = new_input;
    }

    Ok((input, result))
}

/// Parse comparison operators (==, !=, <, <=, >, >=)
fn parse_comparison(input: &str) -> IResult<&str, Expr> {
    let (input, first) = parse_additive(input)?;

    let result_opt = alt((
        map(
            tuple((
                terminated(tag("=="), blank),
                terminated(parse_additive, blank),
            )),
            |(_, rhs)| Expr::Eq(Box::new(first.clone()), Box::new(rhs)),
        ),
        map(
            tuple((
                terminated(tag("!="), blank),
                terminated(parse_additive, blank),
            )),
            |(_, rhs)| Expr::Ne(Box::new(first.clone()), Box::new(rhs)),
        ),
        map(
            tuple((
                terminated(tag("<="), blank),
                terminated(parse_additive, blank),
            )),
            |(_, rhs)| Expr::Le(Box::new(first.clone()), Box::new(rhs)),
        ),
        map(
            tuple((
                terminated(tag(">="), blank),
                terminated(parse_additive, blank),
            )),
            |(_, rhs)| Expr::Ge(Box::new(first.clone()), Box::new(rhs)),
        ),
        map(
            tuple((
                terminated(tag("<"), blank),
                terminated(parse_additive, blank),
            )),
            |(_, rhs)| Expr::Lt(Box::new(first.clone()), Box::new(rhs)),
        ),
        map(
            tuple((
                terminated(tag(">"), blank),
                terminated(parse_additive, blank),
            )),
            |(_, rhs)| Expr::Gt(Box::new(first.clone()), Box::new(rhs)),
        ),
    ))(input);

    match result_opt {
        Ok((input, result)) => Ok((input, result)),
        Err(_) => Ok((input, first)),
    }
}

/// Parse an if-else expression
fn parse_if_else(input: &str) -> IResult<&str, Expr> {
    let (input, _) = terminated(tag("if"), blank)(input)?;
    let (input, cond) = terminated(parse_expression, blank)(input)?;
    let (input, _) = terminated(char('{'), blank)(input)?;
    let (input, then_body) = parse_statements(input)?;
    let (input, _) = terminated(char('}'), blank)(input)?;
    let (input, _) = terminated(tag("else"), blank)(input)?;
    let (input, _) = terminated(char('{'), blank)(input)?;
    let (input, else_body) = parse_statements(input)?;
    let (input, _) = terminated(char('}'), blank)(input)?;

    Ok((input, Expr::IfElse(Box::new(cond), then_body, else_body)))
}

/// Parse a while loop expression
fn parse_while_loop(input: &str) -> IResult<&str, Expr> {
    let (input, _) = terminated(tag("while"), blank)(input)?;
    let (input, cond) = terminated(parse_expression, blank)(input)?;
    let (input, _) = terminated(char('{'), blank)(input)?;
    let (input, loop_body) = parse_statements(input)?;
    let (input, _) = terminated(char('}'), blank)(input)?;

    Ok((input, Expr::WhileLoop(Box::new(cond), loop_body)))
}

/// Parse an assignment expression
fn parse_assignment(input: &str) -> IResult<&str, Expr> {
    let (input, ident) = terminated(parse_identifier, blank)(input)?;
    let (input, _) = terminated(char('='), blank)(input)?;
    let (input, expr) = terminated(parse_expression, blank)(input)?;

    Ok((input, Expr::Assign(ident, Box::new(expr))))
}

/// Parse an expression (top-level)
fn parse_expression(input: &str) -> IResult<&str, Expr> {
    alt((
        parse_if_else,
        parse_while_loop,
        parse_assignment,
        parse_comparison,
    ))(input)
}

/// Parse a statement (expression followed by newline)
fn parse_statement(input: &str) -> IResult<&str, Expr> {
    let (input, expr) = parse_expression(input)?;
    // Consume trailing whitespace (including newline)
    let (input, _) = blank(input)?;

    Ok((input, expr))
}

/// Parse zero or more statements
fn parse_statements(input: &str) -> IResult<&str, Vec<Expr>> {
    many0(parse_statement)(input)
}

/// Parse a function
fn parse_function(input: &str) -> IResult<&str, (String, Vec<String>, String, Vec<Expr>)> {
    let (input, _) = blank(input)?; // Leading whitespace
    let (input, _) = terminated(tag("fn"), blank)(input)?;
    let (input, name) = terminated(parse_identifier, blank)(input)?;
    let (input, params) = delimited(
        terminated(char('('), blank),
        separated_list0(
            terminated(char(','), blank),
            terminated(parse_identifier, blank),
        ),
        terminated(char(')'), blank),
    )(input)?;
    let (input, _) = terminated(tag("->"), blank)(input)?;
    let (input, returns) = delimited(
        terminated(char('('), blank),
        terminated(parse_identifier, blank),
        terminated(char(')'), blank),
    )(input)?;
    let (input, _) = blank(input)?; // Optional whitespace before {
    let (input, _) = terminated(char('{'), blank)(input)?;
    let (input, stmts) = parse_statements(input)?;
    let (input, _) = terminated(char('}'), blank)(input)?;
    let (input, _) = blank(input)?; // Optional trailing whitespace

    Ok((input, (name, params, returns, stmts)))
}

/// Public API: Parse a function from input string
pub fn function(input: &str) -> Result<(String, Vec<String>, String, Vec<Expr>), String> {
    let trimmed = input.trim();
    match parse_function(trimmed) {
        Ok(("", result)) => Ok(result),
        Ok((remaining, result)) => {
            if remaining.trim().is_empty() {
                Ok(result)
            } else {
                Err(format!("Unexpected input remaining: {}", remaining))
            }
        }
        Err(e) => Err(format!("Parse error: {:?}", e)),
    }
}

/// Parser module (for backward compatibility with existing code)
pub mod parser {
    /// Parse a function (re-exported from parent module)
    pub use super::function;
}

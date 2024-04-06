use std::{error::Error, fmt::Display};

pub mod ast;
pub mod combinators;
mod parse_state;

pub use self::parse_state::*;

use crate::lexer::{GetPosition, Span, Token};

use self::{
    ast::{AstNode, Statement},
    combinators::Comb,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub position: Option<Span>,
}

impl ParseError {
    pub fn eof(item: &str) -> ParseError {
        ParseError {
            message: format!("hit EOF while parsing {item}"),
            position: None,
        }
    }
}

impl ParseState<Token> {
    pub fn span(&self) -> Result<Span, ParseError> {
        match self.peek() {
            Some(token) => Ok(token.position()),
            None => Err(ParseError {
                message: "hit EOF".into(),
                position: None,
            }),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(pos) = &self.position {
            f.write_str(pos.to_string(&self.message).as_str())
        } else {
            f.write_str(&self.message)
        }
    }
}

impl Error for ParseError {}

pub trait FromTokens<T> {
    fn parse(tokens: &mut ParseState<T>) -> Result<AstNode, ParseError>;
}

pub fn parse(tokens: &mut ParseState<Token>) -> Result<Vec<Statement<()>>, Box<dyn Error>> {
    let mut statements = vec![];

    let matcher = Comb::STATEMENT;
    while tokens.peek().is_some() {
        match matcher.parse(tokens) {
            Ok(result) => {
                let [AstNode::Statement(statement)] = result.as_slice() else {
                    unreachable!()
                };
                statements.push(statement.clone());
            }
            Err(e) => {
                if let Some(e) = tokens.errors.first() {
                    return Err(Box::new(e.clone()));
                }
                return Err(Box::new(e.clone()));
            }
        }
    }

    if let Some(e) = tokens.errors.first() {
        return Err(Box::new(e.clone()));
    }

    Ok(statements)
}
mod initialization;

pub use self::initialization::*;

use crate::{
    lexer::{Token, Tokens},
    parser::{FromTokens, ParseError},
};

use super::AstNode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Initialization(Initialization),
}

impl FromTokens for Statement {
    fn parse(tokens: &mut Tokens) -> Result<AstNode, ParseError>
    where
        Self: Sized,
    {
        let Some(next) = tokens.peek() else {
            todo!();
        };

        let AstNode::Initialization(init) = Initialization::parse(tokens)? else {
            unreachable!()
        };

        match next {
            Token::Let { .. } => Ok(AstNode::Statement(Statement::Initialization(init))),
            _ => todo!(),
        }
    }
}

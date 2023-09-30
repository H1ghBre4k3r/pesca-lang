mod expression;
mod statement;

use crate::lexer::Token;
use crate::lexer::Tokens;

pub use self::expression::*;
pub use self::statement::*;

use super::combinators::Comb;
use super::FromTokens;
use super::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstNode {
    Expression(Expression),
    Id(Id),
    Num(Num),
    Statement(Statement),
    Initialization(Initialization),
    Function(Function),
    Lambda(Lambda),
    If(If),
    Parameter(Parameter),
    TypeName(TypeName),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeName {
    Literal(String),
    Fn {
        params: Vec<TypeName>,
        return_type: Box<TypeName>,
    },
    Tuple(Vec<TypeName>),
}

impl FromTokens<Token> for TypeName {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
        if let Ok(type_name) = Self::parse_literal(tokens) {
            return Ok(type_name);
        };

        if let Ok(function) = Self::parse_fn(tokens) {
            return Ok(function);
        };

        if let Ok(tuple) = Self::parse_tuple(tokens) {
            return Ok(tuple);
        };

        Err(ParseError {
            message: "could not parse type name".into(),
            position: None,
        })
    }
}

impl TypeName {
    fn parse_literal(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
        let matcher = !Comb::ID;

        let result = matcher.parse(tokens)?;

        let Some(AstNode::Id(type_name)) = result.get(0) else {
            return Err(ParseError {
                message: "Could not parse type literal".into(),
                position: None,
            });
        };

        Ok(TypeName::Literal(type_name.0.clone()).into())
    }

    fn parse_tuple(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
        let index = tokens.get_index();

        let matcher = Comb::LPAREN >> (Comb::TYPE_NAME % Comb::COMMA) >> Comb::RPAREN;

        let result = matcher.parse(tokens).map_err(|e| {
            tokens.set_index(index);
            e
        })?;

        let mut elems = vec![];

        for type_name in &result {
            let AstNode::TypeName(type_name) = type_name else {
                unreachable!()
            };
            elems.push(type_name.clone());
        }

        Ok(TypeName::Tuple(elems).into())
    }

    fn parse_fn(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
        let index = tokens.get_index();

        let AstNode::TypeName(TypeName::Tuple(params)) = Self::parse_tuple(tokens)? else {
            unreachable!()
        };

        let matcher = Comb::SMALL_RIGHT_ARROW >> Comb::TYPE_NAME;

        let result = matcher.parse(tokens).map_err(|e| {
            tokens.set_index(index);
            e
        })?;

        let Some(AstNode::TypeName(type_name)) = result.get(0) else {
            unreachable!()
        };

        Ok(TypeName::Fn {
            params,
            return_type: Box::new(type_name.clone()),
        }
        .into())
    }
}

impl From<TypeName> for AstNode {
    fn from(value: TypeName) -> Self {
        Self::TypeName(value)
    }
}
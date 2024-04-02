use crate::{
    lexer::Token,
    parser::{
        ast::{AstNode, Statement},
        combinators::Comb,
        FromTokens, ParseError, ParseState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block<T> {
    pub statements: Vec<Statement<T>>,
    pub info: T,
}

impl FromTokens<Token> for Block<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let matcher = Comb::LBRACE >> (Comb::STATEMENT ^ Comb::RBRACE);

        let mut result = matcher.parse(tokens)?.into_iter();

        let mut statements = vec![];

        while let Some(AstNode::Statement(statement)) = result.next() {
            statements.push(statement);
        }

        Ok(Block {
            statements,
            info: (),
        }
        .into())
    }
}

impl From<Block<()>> for AstNode {
    fn from(value: Block<()>) -> Self {
        AstNode::Block(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, Span},
        parser::ast::{Expression, Id, Initialisation, Num},
    };

    use super::*;

    #[test]
    fn test_empty_block() {
        let mut tokens = Lexer::new("{ }").lex().expect("something is wrong").into();

        let result = Block::parse(&mut tokens);

        assert_eq!(
            Ok(Block {
                statements: vec![],
                info: ()
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_simple_block() {
        let mut tokens = Lexer::new("{ x }")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Block::parse(&mut tokens);

        assert_eq!(
            Ok(Block {
                statements: vec![Statement::YieldingExpression(Expression::Id(Id {
                    name: "x".into(),
                    info: (),
                    position: Span::default()
                }))],
                info: ()
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_complex_block() {
        let mut tokens = Lexer::new(
            "{
                let a = 42;
                a
            }",
        )
        .lex()
        .expect("something is wrong")
        .into();

        let result = Block::parse(&mut tokens);

        assert_eq!(
            Ok(Block {
                statements: vec![
                    Statement::Initialization(Initialisation {
                        id: Id {
                            name: "a".into(),
                            info: (),
                            position: Span::default()
                        },
                        mutable: false,
                        value: Expression::Num(Num::Integer(42, (), Span::default())),
                        type_name: None,
                        info: ()
                    },),
                    Statement::YieldingExpression(Expression::Id(Id {
                        name: "a".into(),
                        info: (),
                        position: Span::default()
                    }))
                ],
                info: ()
            }
            .into()),
            result
        )
    }
}

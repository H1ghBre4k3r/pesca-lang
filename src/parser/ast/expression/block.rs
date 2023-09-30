use crate::{
    lexer::{Token, Tokens},
    parser::{
        ast::{AstNode, Statement},
        combinators::Comb,
        FromTokens, ParseError,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

impl FromTokens<Token> for Block {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
        let matcher = Comb::LBRACE >> (Comb::STATEMENT ^ ()) >> Comb::RBRACE;

        let mut result = matcher.parse(tokens)?.into_iter();

        let mut statements = vec![];

        while let Some(AstNode::Statement(statement)) = result.next() {
            statements.push(statement);
        }

        Ok(Block { statements }.into())
    }
}

impl From<Block> for AstNode {
    fn from(value: Block) -> Self {
        AstNode::Block(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::Lexer,
        parser::ast::{Expression, Id, Initialization, Num},
    };

    use super::*;

    #[test]
    fn test_empty_block() {
        let mut tokens = Lexer::new("{ }").lex().expect("something is wrong").into();

        let result = Block::parse(&mut tokens);

        assert_eq!(Ok(Block { statements: vec![] }.into()), result)
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
                statements: vec![Statement::Expression(Expression::Id(Id("x".into())))]
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
                    Statement::Initialization(Initialization {
                        id: Id("a".into()),
                        value: Expression::Num(Num(42)),
                        type_name: None
                    },),
                    Statement::Expression(Expression::Id(Id("a".into())))
                ]
            }
            .into()),
            result
        )
    }
}
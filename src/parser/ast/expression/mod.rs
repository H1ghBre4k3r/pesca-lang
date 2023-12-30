mod array;
mod block;
mod function;
mod id;
mod if_expression;
mod lambda;
mod num;
mod postfix;

pub use self::array::*;
pub use self::block::*;
pub use self::function::*;
pub use self::id::*;
pub use self::if_expression::*;
pub use self::lambda::*;
pub use self::num::*;
pub use self::postfix::*;

use crate::lexer::Tokens;
use crate::parser::combinators::Comb;
use crate::{
    lexer::TokenKind,
    parser::{FromTokens, ParseError},
};

use super::AstNode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparisonOperation {
    Equals,
    Greater,
    Less,
    GreaterOrEquals,
    LessOrEquals,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Id(Id),
    Num(Num),
    Function(Function),
    Lambda(Lambda),
    If(If),
    Block(Block),
    Addition(Box<Expression>, Box<Expression>),
    Substraction(Box<Expression>, Box<Expression>),
    Multiplication(Box<Expression>, Box<Expression>),
    Parens(Box<Expression>),
    Postfix(Postfix),
    Comparison {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        operation: ComparisonOperation,
    },
    Array(Array),
}

impl FromTokens<TokenKind> for Expression {
    fn parse(tokens: &mut Tokens<TokenKind>) -> Result<AstNode, ParseError> {
        let mut expr = if let Some(TokenKind::LParen { .. }) = tokens.peek() {
            let matcher = Comb::LPAREN >> Comb::EXPR >> Comb::RPAREN;
            let result = matcher.parse(tokens)?;
            let expr = match result.first() {
                Some(AstNode::Expression(rhs)) => rhs.clone(),
                None | Some(_) => unreachable!(),
            };
            Expression::Parens(Box::new(expr))
        } else {
            let matcher = Comb::FUNCTION
                | Comb::IF
                | Comb::NUM
                | Comb::ID
                | Comb::LAMBDA
                | Comb::BLOCK
                | Comb::ARRAY;
            let result = matcher.parse(tokens)?;
            match result.first() {
                Some(AstNode::Id(id)) => Expression::Id(id.clone()),
                Some(AstNode::Num(num)) => Expression::Num(num.clone()),
                Some(AstNode::Function(func)) => {
                    return Ok(Expression::Function(func.clone()).into())
                }
                Some(AstNode::Lambda(lambda)) => {
                    return Ok(Expression::Lambda(lambda.clone()).into())
                }
                Some(AstNode::If(if_expression)) => Expression::If(if_expression.clone()),
                Some(AstNode::Block(block)) => Expression::Block(block.clone()),
                Some(AstNode::Array(array)) => Expression::Array(array.clone()),
                None | Some(_) => unreachable!(),
            }
        };

        loop {
            let Some(next) = tokens.peek() else {
                return Ok(expr.into());
            };

            let tuple = match next {
                TokenKind::Times { .. } => {
                    tokens.next();
                    Expression::Multiplication
                }
                TokenKind::Plus { .. } => {
                    tokens.next();
                    Expression::Addition
                }
                TokenKind::Minus { .. } => {
                    tokens.next();
                    Expression::Substraction
                }
                TokenKind::LParen { .. } => {
                    expr = Expression::Postfix(Self::parse_call(expr, tokens)?);
                    continue;
                }
                TokenKind::LBracket { .. } => {
                    expr = Expression::Postfix(Self::parse_index(expr, tokens)?);
                    continue;
                }
                TokenKind::Equal { .. }
                | TokenKind::GreaterThan { .. }
                | TokenKind::LessThan { .. }
                | TokenKind::GreaterOrEqual { .. }
                | TokenKind::LessOrEqual { .. } => {
                    return Ok(Self::parse_comparison(expr, tokens)?.into());
                }
                _ => return Ok(expr.into()),
            };

            let matcher = Comb::EXPR;
            let result = matcher.parse(tokens)?;
            let rhs = match result.first() {
                Some(AstNode::Expression(rhs)) => rhs.clone(),
                None | Some(_) => unreachable!(),
            };

            expr = tuple(Box::new(expr), Box::new(rhs))
        }
    }
}

impl Expression {
    fn parse_call(expr: Expression, tokens: &mut Tokens<TokenKind>) -> Result<Postfix, ParseError> {
        let matcher = Comb::LPAREN >> (Comb::EXPR % Comb::COMMA) >> Comb::RPAREN;

        let result = matcher.parse(tokens)?.into_iter();

        let mut args = vec![];

        for arg in result {
            let AstNode::Expression(arg) = arg else {
                unreachable!()
            };

            args.push(arg);
        }

        Ok(Postfix::Call {
            expr: Box::new(expr),
            args,
        })
    }

    fn parse_index(
        expr: Expression,
        tokens: &mut Tokens<TokenKind>,
    ) -> Result<Postfix, ParseError> {
        let matcher = Comb::LBRACKET >> Comb::EXPR >> Comb::RBRACKET;

        let result = matcher.parse(tokens)?;

        let Some(AstNode::Expression(index)) = result.first().cloned() else {
            unreachable!()
        };

        Ok(Postfix::Index {
            expr: Box::new(expr),
            index: Box::new(index),
        })
    }

    fn parse_comparison(
        lhs: Expression,
        tokens: &mut Tokens<TokenKind>,
    ) -> Result<Expression, ParseError> {
        let Some(next) = tokens.next() else {
            unreachable!()
        };

        let comparator = match next {
            TokenKind::Equal { .. } => ComparisonOperation::Equals,
            TokenKind::GreaterThan { .. } => ComparisonOperation::Greater,
            TokenKind::LessThan { .. } => ComparisonOperation::Less,
            TokenKind::GreaterOrEqual { .. } => ComparisonOperation::GreaterOrEquals,
            TokenKind::LessOrEqual { .. } => ComparisonOperation::LessOrEquals,
            _ => unreachable!(),
        };

        let matcher = Comb::EXPR;
        let result = matcher.parse(tokens)?;
        let rhs = match result.first() {
            Some(AstNode::Expression(rhs)) => rhs.clone(),
            None | Some(_) => unreachable!(),
        };

        Ok(Expression::Comparison {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            operation: comparator,
        })
    }
}

impl From<Expression> for AstNode {
    fn from(value: Expression) -> Self {
        AstNode::Expression(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::Lexer,
        parser::ast::{Statement, TypeName},
    };

    use super::*;

    #[test]
    fn test_parse_id() {
        let tokens = vec![TokenKind::Id {
            value: "some_id".into(),
            position: (0, 0),
        }];

        assert_eq!(
            Expression::parse(&mut tokens.into()),
            Ok(AstNode::Expression(Expression::Id(Id("some_id".into()))))
        )
    }

    #[test]
    fn test_parse_num() {
        let tokens = vec![TokenKind::Num {
            value: 42,
            position: (0, 0),
        }];

        assert_eq!(
            Expression::parse(&mut tokens.into()),
            Ok(AstNode::Expression(Expression::Num(Num(42))))
        )
    }

    #[test]
    fn test_parse_function_simple() {
        let mut tokens = Lexer::new("fn (): i32 {}")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Function(Function {
                id: None,
                parameters: vec![],
                statements: vec![],
                return_type: TypeName::Literal("i32".into())
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_parse_function_complex() {
        let mut tokens = Lexer::new(
            "fn (x: i32, y: i32): i32 {
            return x + y;
        }",
        )
        .lex()
        .expect("something is wrong")
        .into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Function(Function {
                id: None,
                parameters: vec![
                    Parameter {
                        name: Id("x".into()),
                        type_name: Some(TypeName::Literal("i32".into()))
                    },
                    Parameter {
                        name: Id("y".into()),
                        type_name: Some(TypeName::Literal("i32".into()))
                    }
                ],
                return_type: TypeName::Literal("i32".into()),
                statements: vec![Statement::Return(Expression::Addition(
                    Box::new(Expression::Id(Id("x".into()))),
                    Box::new(Expression::Id(Id("y".into()))),
                ))]
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_parse_lambda_simple() {
        let mut tokens = Lexer::new("\\() => 42")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Lambda(Lambda {
                parameters: vec![],
                expression: Box::new(Expression::Num(Num(42)))
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_parse_lambda_complex() {
        let mut tokens = Lexer::new("\\(x, y) => { x + y }")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Lambda(Lambda {
                parameters: vec![
                    Parameter {
                        name: Id("x".into()),
                        type_name: None
                    },
                    Parameter {
                        name: Id("y".into()),
                        type_name: None
                    }
                ],
                expression: Box::new(Expression::Block(Block {
                    statements: vec![Statement::YieldingExpression(Expression::Addition(
                        Box::new(Expression::Id(Id("x".into()))),
                        Box::new(Expression::Id(Id("y".into()))),
                    ))]
                }))
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_parse_if() {
        let mut tokens = Lexer::new("if x { 3 + 4 } else { 42 + 1337 }")
            .lex()
            .expect("should work")
            .into();

        assert_eq!(
            Ok(Expression::If(If {
                condition: Box::new(Expression::Id(Id("x".into()))),
                statements: vec![Statement::YieldingExpression(Expression::Addition(
                    Box::new(Expression::Num(Num(3))),
                    Box::new(Expression::Num(Num(4)))
                ))],
                else_statements: vec![Statement::YieldingExpression(Expression::Addition(
                    Box::new(Expression::Num(Num(42))),
                    Box::new(Expression::Num(Num(1337)))
                ))],
            })
            .into()),
            Expression::parse(&mut tokens)
        )
    }

    #[test]
    fn test_parse_postfix_call_simple() {
        let mut tokens = Lexer::new("foo()").lex().expect("should work").into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Postfix(Postfix::Call {
                expr: Box::new(Expression::Id(Id("foo".into()))),
                args: vec![]
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_parse_postfix_call_complex() {
        let mut tokens = Lexer::new("(\\(x, y) => x + y)(42, 1337)")
            .lex()
            .expect("should work")
            .into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Postfix(Postfix::Call {
                expr: Box::new(Expression::Parens(Box::new(Expression::Lambda(Lambda {
                    parameters: vec![
                        Parameter {
                            name: Id("x".into()),
                            type_name: None
                        },
                        Parameter {
                            name: Id("y".into()),
                            type_name: None
                        }
                    ],
                    expression: Box::new(Expression::Addition(
                        Box::new(Expression::Id(Id("x".into()))),
                        Box::new(Expression::Id(Id("y".into())))
                    ))
                })))),
                args: vec![Expression::Num(Num(42)), Expression::Num(Num(1337))]
            })
            .into()),
            result
        );
    }

    #[test]
    fn test_parse_array_empty() {
        let mut tokens = Lexer::new("[]").lex().expect("something is wrong").into();

        let result = Expression::parse(&mut tokens);
        assert_eq!(
            Ok(Expression::Array(Array::Literal { values: vec![] }).into()),
            result
        );
    }

    #[test]
    fn test_parse_array_simple_literal() {
        let mut tokens = Lexer::new("[42, 1337]")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Expression::parse(&mut tokens);
        assert_eq!(
            Ok(Expression::Array(Array::Literal {
                values: vec![Expression::Num(Num(42)), Expression::Num(Num(1337))]
            })
            .into()),
            result
        );
    }

    #[test]
    fn test_parse_index_simple() {
        let mut tokens = Lexer::new("foo[42]")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Postfix(Postfix::Index {
                expr: Box::new(Expression::Id(Id("foo".into()))),
                index: Box::new(Expression::Num(Num(42)))
            })
            .into()),
            result
        )
    }
}

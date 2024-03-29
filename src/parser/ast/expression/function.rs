use crate::{
    lexer::{Token, Tokens},
    parser::{
        ast::{AstNode, Statement, TypeName},
        combinators::Comb,
        FromTokens, ParseError,
    },
};

use super::Id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function<T> {
    pub id: Option<Id<T>>,
    pub parameters: Vec<Parameter<T>>,
    pub return_type: TypeName,
    pub statements: Vec<Statement<T>>,
    pub info: T,
}

impl FromTokens<Token> for Function<()> {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
        let matcher = Comb::FN_KEYWORD
            >> !Comb::ID
            >> Comb::LPAREN
            // parameter list (optional)
            >> (Comb::PARAMETER % Comb::COMMA)
            >> Comb::RPAREN
            // return type
            >> Comb::COLON
            >> Comb::TYPE_NAME
            // body of the function
            >> Comb::LBRACE
            >> (Comb::STATEMENT ^ ())
            >> Comb::RBRACE;

        let mut result = matcher.parse(tokens)?.into_iter().peekable();

        let id = match result.next_if(|item| matches!(item, AstNode::Id(_))) {
            Some(AstNode::Id(id)) => Some(id),
            _ => None,
        };

        let mut parameters = vec![];

        while let Some(AstNode::Parameter(param)) =
            result.next_if(|item| matches!(item, AstNode::Parameter(_)))
        {
            parameters.push(param);
        }

        let Some(AstNode::TypeName(return_type)) = result.next() else {
            unreachable!();
        };

        let mut statements = vec![];

        while let Some(AstNode::Statement(param)) =
            result.next_if(|item| matches!(item, AstNode::Statement(_)))
        {
            statements.push(param);
        }

        Ok(Function {
            id,
            parameters,
            return_type,
            statements,
            info: (),
        }
        .into())
    }
}

impl From<Function<()>> for AstNode {
    fn from(value: Function<()>) -> Self {
        AstNode::Function(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter<T> {
    pub name: Id<T>,
    pub type_name: Option<TypeName>,
    pub info: T,
}

impl FromTokens<Token> for Parameter<()> {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
        let matcher = Comb::ID >> !(Comb::COLON >> Comb::TYPE_NAME);
        let result = matcher.parse(tokens)?;

        let Some(AstNode::Id(name)) = result.first() else {
            unreachable!()
        };

        let type_name = result.get(1).map(|type_name| {
            let AstNode::TypeName(type_name) = type_name else {
                unreachable!()
            };
            type_name.clone()
        });

        Ok(Parameter {
            name: name.clone(),
            type_name,
            info: (),
        }
        .into())
    }
}

impl From<Parameter<()>> for AstNode {
    fn from(value: Parameter<()>) -> Self {
        AstNode::Parameter(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::Lexer,
        parser::ast::{BinaryExpression, Expression},
    };

    use super::*;

    #[test]
    fn test_simple_function() {
        let mut tokens = Lexer::new("fn (): i32 {}")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Function::parse(&mut tokens);

        assert_eq!(
            Ok(Function {
                id: None,
                parameters: vec![],
                return_type: TypeName::Literal("i32".into()),
                statements: vec![],
                info: ()
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_function_with_single_param() {
        let mut tokens = Lexer::new("fn (x: i32): i32 {}")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Function::parse(&mut tokens);

        assert_eq!(
            Ok(Function {
                id: None,
                parameters: vec![Parameter {
                    name: Id("x".into(), ()),
                    type_name: Some(TypeName::Literal("i32".into())),
                    info: ()
                }],
                return_type: TypeName::Literal("i32".into()),
                statements: vec![],
                info: ()
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_function_with_multiple_params() {
        let mut tokens = Lexer::new("fn (x: i32, y: i32): i32 {}")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Function::parse(&mut tokens);

        assert_eq!(
            Ok(Function {
                id: None,
                parameters: vec![
                    Parameter {
                        name: Id("x".into(), ()),
                        type_name: Some(TypeName::Literal("i32".into())),
                        info: ()
                    },
                    Parameter {
                        name: Id("y".into(), ()),
                        type_name: Some(TypeName::Literal("i32".into())),
                        info: ()
                    }
                ],
                return_type: TypeName::Literal("i32".into()),
                statements: vec![],
                info: ()
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_function_with_statements() {
        let mut tokens = Lexer::new("fn (x: i32, y: i32): i32 { return x + y; }")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Function::parse(&mut tokens);

        assert_eq!(
            Ok(Function {
                id: None,
                parameters: vec![
                    Parameter {
                        name: Id("x".into(), ()),
                        type_name: Some(TypeName::Literal("i32".into())),
                        info: ()
                    },
                    Parameter {
                        name: Id("y".into(), ()),
                        type_name: Some(TypeName::Literal("i32".into())),
                        info: ()
                    }
                ],
                return_type: TypeName::Literal("i32".into()),
                statements: vec![Statement::Return(Expression::Binary(Box::new(
                    BinaryExpression::Addition(
                        Expression::Id(Id("x".into(), ())),
                        Expression::Id(Id("y".into(), ())),
                    )
                )))],
                info: ()
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_function_with_name() {
        let mut tokens = Lexer::new("fn main(x: i32, y: i32): i32 { return x + y; }")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Function::parse(&mut tokens);

        assert_eq!(
            Ok(Function {
                id: Some(Id("main".into(), ())),
                parameters: vec![
                    Parameter {
                        name: Id("x".into(), ()),
                        type_name: Some(TypeName::Literal("i32".into())),
                        info: ()
                    },
                    Parameter {
                        name: Id("y".into(), ()),
                        type_name: Some(TypeName::Literal("i32".into())),
                        info: ()
                    }
                ],
                return_type: TypeName::Literal("i32".into()),
                statements: vec![Statement::Return(Expression::Binary(Box::new(
                    BinaryExpression::Addition(
                        Expression::Id(Id("x".into(), ())),
                        Expression::Id(Id("y".into(), ())),
                    )
                )))],
                info: ()
            }
            .into()),
            result
        )
    }
}

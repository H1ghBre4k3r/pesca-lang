use std::{cell::RefCell, rc::Rc};

use crate::{
    parser::ast::{BinaryExpression, BinaryOperator},
    typechecker::{
        context::Context,
        error::{TypeCheckError, TypeMismatch},
        types::Type,
        TypeCheckable, TypeInformation, TypeResult,
    },
};

impl TypeCheckable for BinaryExpression<()> {
    type Output = BinaryExpression<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let context = ctx.clone();
        let BinaryExpression {
            left,
            right,
            operator,
            position,
            ..
        } = self;

        let left = left.check(ctx)?;
        let right = right.check(ctx)?;

        let left_type = { left.get_info().type_id.borrow() }.clone();
        let right_type = { right.get_info().type_id.borrow() }.clone();

        let compount_type = if let (Some(left_type), Some(right_type)) = (left_type, right_type) {
            if left_type != right_type {
                return Err(TypeCheckError::TypeMismatch(
                    TypeMismatch {
                        expected: left_type,
                        actual: right_type,
                    },
                    position,
                ));
            }
            Some(left_type)
        } else {
            None
        };

        let type_id = match operator {
            BinaryOperator::Add
            | BinaryOperator::Substract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide => compount_type,
            BinaryOperator::Equals
            | BinaryOperator::GreaterThan
            | BinaryOperator::LessThan
            | BinaryOperator::GreaterOrEqual
            | BinaryOperator::LessOrEqual => Some(Type::Boolean),
        };

        Ok(BinaryExpression {
            left,
            right,
            operator,
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(type_id)),
                context,
            },
            position,
        })
    }

    fn revert(this: &Self::Output) -> Self {
        let BinaryExpression {
            left,
            right,
            operator,
            position,
            ..
        } = this;

        BinaryExpression {
            left: TypeCheckable::revert(left),
            right: TypeCheckable::revert(right),
            operator: *operator,
            info: (),
            position: position.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use anyhow::Result;

    use crate::{
        lexer::Span,
        parser::ast::{BinaryExpression, BinaryOperator, Expression, Num},
        typechecker::{
            context::Context,
            error::{TypeCheckError, TypeMismatch},
            types::Type,
            TypeCheckable,
        },
    };

    #[test]
    fn test_simple_addition() -> Result<()> {
        let mut ctx = Context::default();
        let exp = BinaryExpression {
            left: Expression::Num(Num::Integer(42, (), Span::default())),
            right: Expression::Num(Num::Integer(1337, (), Span::default())),
            operator: BinaryOperator::Add,
            info: (),
            position: Span::default(),
        };

        let exp = exp.check(&mut ctx)?;

        assert_eq!(exp.info.type_id, Rc::new(RefCell::new(Some(Type::Integer))));

        Ok(())
    }

    #[test]
    fn test_simple_equality() -> Result<()> {
        let mut ctx = Context::default();
        let exp = BinaryExpression {
            left: Expression::Num(Num::Integer(42, (), Span::default())),
            right: Expression::Num(Num::Integer(1337, (), Span::default())),
            operator: BinaryOperator::Equals,
            info: (),
            position: Span::default(),
        };

        let exp = exp.check(&mut ctx)?;

        assert_eq!(exp.info.type_id, Rc::new(RefCell::new(Some(Type::Boolean))));

        Ok(())
    }

    #[test]
    fn test_addition_with_incompatible_types() -> Result<()> {
        let mut ctx = Context::default();
        let exp = BinaryExpression {
            left: Expression::Num(Num::Integer(42, (), Span::default())),
            right: Expression::Num(Num::FloatingPoint(1337.0, (), Span::default())),
            operator: BinaryOperator::Add,
            info: (),
            position: Span::default(),
        };

        let res = exp.check(&mut ctx);

        assert_eq!(
            res,
            Err(TypeCheckError::TypeMismatch(
                TypeMismatch {
                    expected: Type::Integer,
                    actual: Type::FloatingPoint,
                },
                Span::default()
            ))
        );

        Ok(())
    }
}

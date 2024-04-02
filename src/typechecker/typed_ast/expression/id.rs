use crate::{
    parser::ast::Id,
    typechecker::{
        context::Context,
        error::{TypeCheckError, UndefinedVariable},
        types::Type,
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Id<()> {
    type Output = Id<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let Id { name, position, .. } = self;

        let Some(type_id) = ctx.scope.resolve_name(&name) else {
            return Err(TypeCheckError::UndefinedVariable(UndefinedVariable {
                variable_name: name,
            }));
        };

        Ok(Id {
            name,
            info: TypeInformation {
                type_id,
                context: ctx.clone(),
            },
            position,
        })
    }

    fn revert(this: &Self::Output) -> Self {
        let Id { name, position, .. } = this;

        Id {
            name: name.to_owned(),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for Id<TypeInformation> {
    fn update_type(&mut self, type_id: Type) -> TypeResult<()> {
        // propagate type update to variable refences by this id
        self.info.context.scope.update_variable(&self.name, type_id)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use crate::{
        lexer::Span,
        parser::ast::{Expression, Id},
        typechecker::{
            context::Context,
            error::{TypeCheckError, UndefinedVariable},
            types::Type,
            TypeCheckable, TypeInformation,
        },
    };

    #[test]
    fn test_no_member_modification() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();
        ctx.scope
            .add_variable(
                "foo",
                Expression::Id(Id {
                    name: "foo".into(),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                        context: Context::default(),
                    },
                    position: Span::default(),
                }),
            )
            .expect("something went wrong");

        let id = Id {
            name: "foo".into(),
            info: (),
            position: Span::default(),
        };

        let id = id.check(&mut ctx)?;

        assert_eq!(id.name, "foo".to_string());

        Ok(())
    }

    #[test]
    fn test_correct_type_inference() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();
        ctx.scope
            .add_variable(
                "foo",
                Expression::Id(Id {
                    name: "foo".into(),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                        context: Context::default(),
                    },
                    position: Span::default(),
                }),
            )
            .expect("something went wrong");

        let id = Id {
            name: "foo".into(),
            info: (),
            position: Span::default(),
        };

        let id = id.check(&mut ctx)?;

        assert_eq!(
            id.info,
            TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                context: Context::default(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_error_on_undefined() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let id = Id {
            name: "foo".into(),
            info: (),
            position: Span::default(),
        };

        let res = id.check(&mut ctx);

        assert_eq!(
            res,
            Err(TypeCheckError::UndefinedVariable(UndefinedVariable {
                variable_name: "foo".into()
            }))
        );

        Ok(())
    }

    #[test]
    fn test_retrival_of_constant() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();
        ctx.scope.add_constant("foo", Type::Integer)?;

        let id = Id {
            name: "foo".into(),
            info: (),
            position: Span::default(),
        };

        let id = id.check(&mut ctx)?;

        assert_eq!(id.info.type_id, Rc::new(RefCell::new(Some(Type::Integer))));

        Ok(())
    }
}

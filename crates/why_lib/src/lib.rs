use std::fs;

use lexer::{Lexer, Token};
use parser::{ast::TopLevelStatement, parse};
use sha2::{Digest, Sha256};
use typechecker::{TypeChecker, TypeInformation, ValidatedTypeInformation};

pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod typechecker;

#[derive(Debug, Clone)]
pub struct Module<T> {
    path: String,
    pub inner: T,
}

impl<A> Module<A> {
    fn convert<B>(&self, inner: B) -> Module<B> {
        let Module { path, .. } = self;

        Module {
            path: path.clone(),
            inner,
        }
    }

    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.path.as_bytes());
        let result = hasher.finalize();
        format!("{result:x}")
    }
}

impl Module<()> {
    pub fn new(path: String) -> Self {
        Self { path, inner: () }
    }

    pub fn lex(&self) -> anyhow::Result<Module<Vec<Token>>> {
        let input = fs::read_to_string(&self.path)?;

        let lexer = Lexer::new(&input);

        Ok(self.convert(lexer.lex()?))
    }
}

impl Module<Vec<Token>> {
    pub fn parse(&self) -> anyhow::Result<Module<Vec<TopLevelStatement<()>>>> {
        let tokens = self.inner.clone();

        Ok(self.convert(parse(&mut tokens.into())?))
    }
}

impl Module<Vec<TopLevelStatement<()>>> {
    pub fn check(&self) -> anyhow::Result<Module<Vec<TopLevelStatement<TypeInformation>>>> {
        let statements = self.inner.clone();
        let typechecker = TypeChecker::new(statements);

        Ok(self.convert(typechecker.check()?))
    }
}

impl Module<Vec<TopLevelStatement<TypeInformation>>> {
    pub fn validate(
        &self,
    ) -> anyhow::Result<Module<Vec<TopLevelStatement<ValidatedTypeInformation>>>> {
        Ok(self.convert(TypeChecker::validate(self.inner.clone())?))
    }
}

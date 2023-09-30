use pesca_parser_derive::{LooseEq, Token as ParseToken};

type Position = (usize, usize);

#[derive(Debug, Clone, ParseToken, LooseEq)]
pub enum Token {
    #[terminal]
    Eq {
        position: Position,
    },
    #[terminal]
    Let {
        position: Position,
    },
    Id {
        value: String,
        position: Position,
    },
    Num {
        value: u64,
        position: Position,
    },
    #[terminal]
    Semicolon {
        position: Position,
    },
    // TODO: think about lexing comments
    Comment {
        value: String,
        position: Position,
    },
    #[terminal]
    Plus {
        position: Position,
    },
    #[terminal]
    Times {
        position: Position,
    },
    #[terminal]
    LParen {
        position: Position,
    },
    #[terminal]
    RParen {
        position: Position,
    },
    #[terminal]
    LBrace {
        position: Position,
    },
    #[terminal]
    RBrace {
        position: Position,
    },
    #[terminal]
    FnKeyword {
        position: Position,
    },
    #[terminal]
    IfKeyword {
        position: Position,
    },
    #[terminal]
    ElseKeyword {
        position: Position,
    },
    #[terminal]
    ReturnKeyword {
        position: Position,
    },
    #[terminal]
    Colon {
        position: Position,
    },
    #[terminal]
    Comma {
        position: Position,
    },
    #[terminal]
    SmallRightArrow {
        position: Position,
    },
    #[terminal]
    BigRightArrow {
        position: Position,
    },
    #[terminal]
    Backslash {
        position: Position,
    },
}


use logos::Logos;

/// Primitive tokens that might occur in a FIRRTL file
#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t,]+")]
pub enum Token {
    #[regex("[a-zA-Z_][a-zA-Z0-9_$-]*", |lex| lex.slice().parse().ok())]
    IdentKw(String),

    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#, |lex| lex.slice().parse().ok())]
    StringLiteral(String),

    #[regex(r#"'([^'\\]|\\t|\\u|\\n|\\|\\')*'"#, |lex| lex.slice().parse().ok())]
    RawString(String),

    #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
    LiteralInt(String),

    #[regex("[+-][0-9]+", |lex| lex.slice().parse().ok())]
    LiteralSInt(String),

    #[token(".")]  Period,
    #[token(":")]  Colon,
    #[token("?")]  Question,
    #[token("(")]  LParen,
    #[token(")")]  RParen,
    #[token("{")]  LBrace,
    #[token("}")]  RBrace,
    #[token("[")]  LSquare,
    #[token("]")]  RSquare,
    #[token("<")]  Less,
    #[token("<-")] LessMinus,
    #[token("<=")] LessEqual,
    #[token(">")]  Greater,
    #[token("=")]  Equal,
    #[token("=>")] EqualGreater,
}
impl Token {
    pub fn punctuation_from_str(s: &str) -> Self { 
        match s {
            "."  => Self::Period,
            ":"  => Self::Colon,
            "?"  => Self::Question,
            "("  => Self::LParen,
            ")"  => Self::RParen,
            "{"  => Self::LBrace,
            "}"  => Self::RBrace,
            "["  => Self::LSquare,
            "]"  => Self::RSquare,
            "<"  => Self::Less,
            "<-" => Self::LessMinus,
            "<=" => Self::LessEqual,
            ">"  => Self::Greater,
            "="  => Self::Equal,
            "=>" => Self::EqualGreater,
            _ => panic!("Cannot convert '{}' into Token?", s),
        }
    }
}



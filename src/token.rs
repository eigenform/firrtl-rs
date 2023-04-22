
use logos::Logos;

/// Primitive tokens that might occur in a FIRRTL file
#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t,]+")]
pub enum Token {
    #[regex("[a-zA-Z_][a-zA-Z0-9_$-]*", |lex| lex.slice().parse().ok())]
    IdentKw(String),

    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#, |lex| lex.slice().parse().ok())]
    LiteralString(String),

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

impl Token {
    pub fn is_lit_int(&self) -> bool {
        matches!(self, Token::LiteralInt(_))
    }
    pub fn is_lit_sint(&self) -> bool {
        matches!(self, Token::LiteralSInt(_))
    }
    pub fn is_lit_str(&self) -> bool {
        matches!(self, Token::LiteralString(_))
    }
    pub fn is_raw_str(&self) -> bool {
        matches!(self, Token::RawString(_))
    }
    pub fn is_punc(&self) -> bool { 
        match self { 
            Self::Period 
            | Self::Colon
            | Self::Question 
            | Self::LParen 
            | Self::RParen 
            | Self::LBrace 
            | Self::RBrace 
            | Self::LSquare 
            | Self::RSquare 
            | Self::Less 
            | Self::LessMinus 
            | Self::LessEqual 
            | Self::Greater 
            | Self::Equal 
            | Self::EqualGreater => true, 
            _ => false,
        }
    }
    pub fn is_identkw(&self) -> bool { 
        matches!(self, Token::IdentKw(_))
    }

    pub fn get_identkw(&self) -> Option<&str> {
        if let Token::IdentKw(s) = self { Some(s) } else { None }
    }
    pub fn get_lit_int(&self) -> Option<&str> {
        if let Token::LiteralInt(s) = self { Some(s) } else { None }
    }
    pub fn get_lit_sint(&self) -> Option<&str> {
        if let Token::LiteralSInt(s) = self { Some(s) } else { None }
    }
    pub fn get_lit_str(&self) -> Option<&str> {
        if let Token::LiteralString(s) = self { Some(s) } else { None }
    }
    pub fn get_raw_str(&self) -> Option<&str> {
        if let Token::RawString(s) = self { Some(s) } else { None }
    }

    pub fn match_punc(&self, p: &str) -> Option<bool> {
        if self.is_punc() {
            Some(self == &Self::punctuation_from_str(p))
        } else { 
            None
        }
    }
    pub fn match_identkw(&self, kw: &str) -> Option<bool> { 
        if let Token::IdentKw(s) = self { 
            Some(s.as_str() == kw)
        } else { 
            None
        }
    }

}



use logos::Logos;
use crate::ast;

/// A primitive token that might occur in a FIRRTL file.
///
/// NOTE: There isn't exactly a hard distinction between variable names
/// ("identifiers") and "keywords" in FIRRTL. Since this usually depends 
/// on some context, it makes more sense for us to treat these as the 
/// same underlying token ([Token::IdentKw]).
///
#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t,]+")]
pub enum Token {
    /// An "identifier" or a "keyword"
    #[regex("[a-zA-Z_][a-zA-Z0-9_$-]*", |lex| lex.slice().parse().ok())]
    IdentKw(String),

    /// A double-quoted literal string
    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#, |lex| lex.slice().parse().ok())]
    LiteralString(String),

    /// A single-quoted literal string
    #[regex(r#"'([^'\\]|\\t|\\u|\\n|\\|\\')*'"#, |lex| lex.slice().parse().ok())]
    RawString(String),

    /// A literal integer value
    #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
    LiteralInt(String),

    /// A literal floating-point value
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse().ok())]
    LiteralFloat(String),

    /// A literal signed integer value
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
    pub fn get_lit_float(&self) -> Option<&str> {
        if let Token::LiteralFloat(s) = self { Some(s) } else { None }
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

    pub fn get_unsigned_numeric_literal(&self) -> Option<ast::LiteralNumeric> {
        match self {
            Token::LiteralInt(s) => {
                Some(ast::LiteralNumeric::UInt(s.parse().unwrap()))
            },
            Token::LiteralString(s) => {
                let slice = &s[1..s.len()-1];
                if let Some(hex_num) = slice.strip_prefix('h') {
                    Some(ast::LiteralNumeric::UInt(
                        usize::from_str_radix(hex_num, 16).unwrap())
                    )
                }
                else if let Some(oct_num) = slice.strip_prefix('o') {
                    Some(ast::LiteralNumeric::UInt(
                        usize::from_str_radix(oct_num, 8).unwrap())
                    )
                }
                else if let Some(bin_num) = slice.strip_prefix('b') {
                    Some(ast::LiteralNumeric::UInt(
                        usize::from_str_radix(bin_num, 2).unwrap())
                    )
                } else {
                    panic!("unexpected numeric unsigned literal format {:?}", s);
                }
            },
            _ => panic!("unexpected token {:?} for unsigned literal", self),
        }
    }
    pub fn get_signed_numeric_literal(&self) -> Option<ast::LiteralNumeric> {
        match self {
            Token::LiteralSInt(s) => {
                Some(ast::LiteralNumeric::SInt(s.parse().unwrap()))
            },
            Token::LiteralString(s) => {
                let slice = &s[1..s.len()-1];
                if let Some(hex_num) = slice.strip_prefix('h') {
                    Some(ast::LiteralNumeric::SInt(
                        isize::from_str_radix(hex_num, 16).unwrap())
                    )
                }
                else if let Some(oct_num) = slice.strip_prefix('o') {
                    Some(ast::LiteralNumeric::SInt(
                        isize::from_str_radix(oct_num, 8).unwrap())
                    )
                }
                else if let Some(bin_num) = slice.strip_prefix('b') {
                    Some(ast::LiteralNumeric::SInt(
                        isize::from_str_radix(bin_num, 2).unwrap())
                    )
                } else {
                    panic!("unexpected numeric signed literal format {:?}", s);
                }
            },
            _ => panic!("unexpected token {:?} for signed literal", self),
        }
    }

}


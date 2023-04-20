
pub use logos::Logos;
use miette::{ Diagnostic, SourceSpan, SourceOffset, NamedSource, Result };
use thiserror::Error;
use std::ops::Range;
use std::collections::*;
use std::str::FromStr;

#[derive(Error, Debug, Diagnostic)]
#[error("Lexer error")]
pub struct LexerError {
    #[source_code]
    pub src: NamedSource,
    #[label("Somewhere around here ...")]
    pub span: SourceSpan,
}

// NOTE: This skips horizontal whitespace and comments
#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \n\r,]+")]
#[logos(skip r";.*\n")]
pub enum LogosToken<'src> {
    #[regex("[a-zA-Z_][a-zA-Z0-9_$-]*", |lex| lex.slice())]
    IdentKw(&'src str),

    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#, |lex| lex.slice())]
    StringLiteral(&'src str),

    #[regex(r#"'([^'\\]|\\t|\\u|\\n|\\|\\')*'"#, |lex| lex.slice())]
    RawString(&'src str),

    #[regex(r#"%\[([^'\\\[\]]|\\t|\\u|\\n|\\|\\')*\]"#, |lex| lex.slice())]
    InlineNotation(&'src str),

    #[regex(r#"@\[([^'\\\[\]]|\\t|\\u|\\n|\\|\\')*\]"#, |lex| lex.slice())]
    FileInfo(&'src str),

    #[regex(r"<[0-9][0-9]*>")]
    Width(&'src str),

    //#[regex("[a-zA-Z]")] Alpha,
    #[regex("[0-9]+", |lex| lex.slice())]
    LiteralInt(&'src str),

    #[regex("[+-][0-9]+", |lex| lex.slice())]
    LiteralSignedInt(&'src str),

    #[regex(r"[0-9]\.[0-9]\.[0-9]", |lex| lex.slice())]
    LiteralVersion(&'src str),

    // NOTE: The only relevant horizontal whitespace occurs after a newline. 
    // This matches a newline, but only returns the whitespace.
    #[regex(r"[\n\r\v\f]([ \t,])+", |lex| &lex.slice()[1..] )]
    WhitespaceH(&'src str),

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirrtlPunctuation {
    Period, Colon, Question, LParen, RParen, LBrace, RBrace, LSquare, RSquare,
    Less, LessEqual, LessMinus, Greater, Equal, EqualGreater
}
impl <'src> FirrtlPunctuation {
    fn from_lt(t: &LogosToken<'src>) -> Option<Self> { 
        match t {
            LogosToken::Period       => Some(Self::Period),
            LogosToken::Colon        => Some(Self::Colon),
            LogosToken::Question     => Some(Self::Question),
            LogosToken::LParen       => Some(Self::LParen),
            LogosToken::RParen       => Some(Self::RParen),
            LogosToken::LBrace       => Some(Self::LBrace),
            LogosToken::RBrace       => Some(Self::RBrace),
            LogosToken::LSquare      => Some(Self::LSquare),
            LogosToken::RSquare      => Some(Self::RSquare),
            LogosToken::Less         => Some(Self::Less),
            LogosToken::LessEqual    => Some(Self::LessEqual),
            LogosToken::LessMinus    => Some(Self::LessMinus),
            LogosToken::Greater      => Some(Self::Greater),
            LogosToken::Equal        => Some(Self::Equal),
            LogosToken::EqualGreater => Some(Self::EqualGreater),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirrtlLiteral<'src> {
    Int(&'src str),
    SignedInt(&'src str),
    Float(&'src str),
    Version(&'src str),
    String(&'src str),
    RawString(&'src str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirrtlToken<'src> {
    /// Identifier/keyword
    IdentKw(&'src str),
    /// Literals
    Literal(FirrtlLiteral<'src>),
    /// Punctuation
    Punctuation(FirrtlPunctuation),
    /// File info
    FileInfo(&'src str),
    /// Type width (in bits)
    TypeWidth(usize),
    /// Horizontal whitespace
    WhitespaceH(usize),
    /// End-of-file
    EOF, 
}
impl <'src> FirrtlToken<'src> {
    pub fn is_identkw(&self) -> bool { 
        matches!(self, Self::IdentKw(_)) 
    }
    pub fn is_punctuation(&self, p: FirrtlPunctuation) -> bool { 
        matches!(self, Self::Punctuation(p)) 
    }
    pub fn is_whitespace(&self) -> bool { 
        matches!(self, Self::WhitespaceH(_)) 
    }
}




pub struct FirrtlLexer<'src> {
    /// The input string
    src: &'src str,

    filename: String,
    /// A stream of [LogosToken] 
    llexer: logos::Lexer<'src, LogosToken<'src>>,
}
impl <'src> FirrtlLexer<'src> {
    pub fn new(filename: &str, src: &'src str) -> Self { 
        Self { 
            src, 
            filename: filename.to_string(),
            llexer: LogosToken::lexer(&src),
        }
    }

    fn firrtl_lex(&mut self, cur: LogosToken<'src>) -> FirrtlToken<'src>
    {
        println!("Start at {:?}, {:?}", cur, self.llexer.span());
        match cur { 
            LogosToken::Width(s) => {
                FirrtlToken::TypeWidth(s[1..s.len()-1].parse().unwrap())
            },
            LogosToken::WhitespaceH(s) => FirrtlToken::WhitespaceH(s.len()),
            LogosToken::IdentKw(s)  => FirrtlToken::IdentKw(s),
            LogosToken::FileInfo(s) => FirrtlToken::FileInfo(s),
            LogosToken::LiteralInt(s) => {
                FirrtlToken::Literal(FirrtlLiteral::Int(s))
            },
            LogosToken::LiteralSignedInt(s) => {
                FirrtlToken::Literal(FirrtlLiteral::SignedInt(s))
            },
            LogosToken::StringLiteral(s) => {
                FirrtlToken::Literal(FirrtlLiteral::String(s))
            },
            LogosToken::RawString(s) => {
                FirrtlToken::Literal(FirrtlLiteral::RawString(s))
            },
            LogosToken::LiteralVersion(s) => {
                FirrtlToken::Literal(FirrtlLiteral::Version(s))
            },

            lt if FirrtlPunctuation::from_lt(&lt).is_some() => {
                FirrtlToken::Punctuation(
                    FirrtlPunctuation::from_lt(&lt).unwrap()
                )
            },
            _ => unimplemented!("{:?}", cur),
        }
    }

}

impl <'src> Iterator for FirrtlLexer<'src> {
    type Item = Result<FirrtlToken<'src>>;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        // FIXME: Probably want to return span info too?
        let span = self.llexer.span();
        if let Some(ltok_res) = self.llexer.next() {
            match ltok_res {
                // Convert from [LogosToken] into [FirrtlToken]
                Ok(ltok) => Some(Ok(self.firrtl_lex(ltok))),
                // Emit fancy spanned errors with [miette]
                Err(ltok_err) => {
                    let src = String::from_str(self.src).unwrap();
                    let rsrc = NamedSource::new(self.filename.as_str(), src);
                    return Some(Err(LexerError {
                        src: rsrc, span: (span.start, span.len()).into()
                    }.into()));
                },
            }
        } else { 
            None
        }
    }
}



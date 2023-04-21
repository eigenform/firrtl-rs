
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
// NOTE: Remember that *matches* on whitespace depend on newline
#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \n\r,]+")]
//#[logos(skip r";.*\n")]
//#[logos(skip r";([^\n\r])+")]
pub enum LogosToken<'src> {

    #[regex(r";([^\n\r])*")]
    Comment(&'src str),

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

    //#[regex(r"<[0-9][0-9]*>")]
    //Width(&'src str),

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
pub enum FirrtlPunct {
    Period, Colon, Question, LParen, RParen, LBrace, RBrace, LSquare, RSquare,
    Less, LessEqual, LessMinus, Greater, Equal, EqualGreater
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
    Ignore,
    /// Identifier/keyword
    IdentKw(&'src str),
    /// Literals
    Literal(FirrtlLiteral<'src>),
    /// Punctuation
    Punct(FirrtlPunct),
    /// File info
    FileInfo(&'src str),
    /// Horizontal whitespace
    WhitespaceH(usize),
    /// End-of-file
    EOF, 
}
impl <'src> FirrtlToken<'src> {
    pub fn is_identkw(&self) -> bool { 
        matches!(self, Self::IdentKw(_)) 
    }
    pub fn is_punctuation(&self) -> bool {
        matches!(self, Self::Punct(_)) 
    }
    pub fn is_whitespace(&self) -> bool { 
        matches!(self, Self::WhitespaceH(_)) 
    }
    pub fn is_int_lit(&self) -> bool {
        matches!(self, Self::Literal(FirrtlLiteral::Int(_)))
    }


    pub fn get_whitespace(&self) -> Option<usize> {
        if let Self::WhitespaceH(x) = self { Some(*x) } else { None }
    }
    pub fn whitespace_matches(&self, x: usize) -> bool {
        matches!(self, Self::WhitespaceH(x))
    }
    pub fn punct_matches(&self, p: FirrtlPunct) -> bool {
        matches!(self, Self::Punct(p)) 
    }
    pub fn identkw_matches(&self, s: &str) -> bool {
        matches!(self, Self::IdentKw(s))
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

    /// Convert from [LogosToken] to [FirrtlToken].
    fn firrtl_lex(&mut self, cur: LogosToken<'src>) -> FirrtlToken<'src>
    {
        println!("Start at {:?}, {:?}", cur, self.llexer.span());
        match cur { 
            LogosToken::Comment(s) => FirrtlToken::Ignore,
            //LogosToken::Width(s) => {
            //    FirrtlToken::TypeWidth(s[1..s.len()-1].parse().unwrap())
            //},
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

            LogosToken::Period => FirrtlToken::Punct(FirrtlPunct::Period),
            LogosToken::Colon => FirrtlToken::Punct(FirrtlPunct::Colon),
            LogosToken::Question => FirrtlToken::Punct(FirrtlPunct::Question),
            LogosToken::LParen => FirrtlToken::Punct(FirrtlPunct::LParen),
            LogosToken::RParen => FirrtlToken::Punct(FirrtlPunct::RParen),
            LogosToken::LBrace => FirrtlToken::Punct(FirrtlPunct::LBrace),
            LogosToken::RBrace => FirrtlToken::Punct(FirrtlPunct::RBrace),
            LogosToken::LSquare => FirrtlToken::Punct(FirrtlPunct::LSquare),
            LogosToken::RSquare => FirrtlToken::Punct(FirrtlPunct::RSquare),
            LogosToken::Less => FirrtlToken::Punct(FirrtlPunct::Less),
            LogosToken::LessEqual => FirrtlToken::Punct(FirrtlPunct::LessEqual),
            LogosToken::LessMinus => FirrtlToken::Punct(FirrtlPunct::LessMinus),
            LogosToken::Greater => FirrtlToken::Punct(FirrtlPunct::Greater),
            LogosToken::Equal => FirrtlToken::Punct(FirrtlPunct::Equal),
            LogosToken::EqualGreater => 
                FirrtlToken::Punct(FirrtlPunct::EqualGreater),
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



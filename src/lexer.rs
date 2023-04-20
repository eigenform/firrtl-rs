
pub use logos::Logos;
use miette::{ Diagnostic, SourceSpan, SourceOffset, NamedSource, Result };
use thiserror::Error;
use std::ops::Range;
use std::collections::*;
use std::str::FromStr;

//#[allow(non_camel_case_types)]
//pub enum Keyword {
//    Analog,
//    AsyncReset,
//    Clock,
//    FIRRTL,
//    Fixed,
//    Probe,
//    RWProbe,
//    Reset,
//    SInt,
//    UInt,
//    attach,
//    circuit,
//    cmem,
//    define,
//    defname,
//    intrinsic,
//    r#else,
//    extmodule,
//    intmodule,
//    flip,
//    infer,
//    input,
//    inst,
//    invalid,
//    is,
//    mem,
//    module,
//    mport,
//    new,
//    node,
//    of,
//    old,
//    output,
//    parameter,
//    rdwr,
//    read,
//    r#ref,
//    reg,
//    reset,
//    skip,
//    smem,
//    undefined,
//    version,
//    when,
//    wire,
//    with,
//    write,
//}
//
//#[allow(non_camel_case_types)]
//pub enum LpKeyword {
//    printf,
//    stop,
//    assert,
//    assume,
//    cover,
//    force,
//    force_initial,
//    release,
//    release_initial,
//    read,
//    probe,
//    rwprobe,
//}
//
//pub enum Punctuation {
//    Period,
//    Colon,
//    Question,
//    LParen,
//    RParen,
//    LBrace,
//    RBrace,
//    LSquare,
//    RSquare,
//    Less,
//    LessEqual,
//    LessMinus,
//    Greater,
//    Equal,
//    EqualGreater
//}
//
//
//pub enum Token<'input> {
//    Ident(&'input str),
//    Literal(&'input str),
//    Keyword(Keyword),
//    LpKeyword(LpKeyword),
//}
//
//pub struct Lexer<'input> {
//    input: &'input str,
//}
//impl <'input> Lexer<'input> {
//
//    /// Create a new [Lexer] for the provided input.
//    pub fn new(input: &'input str) -> Self {
//        Self { input }
//    }
//
//    pub fn lex(&self) {
//        let chars: Vec<char> = self.input.chars().collect();
//        let mut cur = 0;
//        loop {
//            match chars[cur] {
//                ' ' | '\t' | '\n' | '\r' | ',' => continue,
//                _ => unimplemented!("{}", chars[cur]),
//            }
//        }
//    }
//}

#[derive(Error, Debug, Diagnostic)]
#[error("Lexer error")]
pub struct LexerError {
    #[source_code]
    pub src: NamedSource,
    #[label("Somewhere around here ...")]
    pub span: SourceSpan,
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\r,]+")]
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

    //#[regex("[a-zA-Z]")] Alpha,
    #[regex("[0-9]+", |lex| lex.slice())]
    LiteralInt(&'src str),

    #[regex("[+-][0-9]+", |lex| lex.slice())]
    LiteralSignedInt(&'src str),

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

#[derive(Clone, Copy, Debug)]
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

#[derive(Debug)]
pub enum FirrtlLiteral<'src> {
    Int(&'src str),
    SignedInt(&'src str),
    Float(&'src str),
    Version(&'src str),
    String(&'src str),
    RawString(&'src str),
}

#[derive(Debug)]
pub enum FirrtlToken<'src> {
    /// Identifier/keyword
    IdentKw(&'src str),
    /// Literals
    Literal(FirrtlLiteral<'src>),
    /// Punctuation
    Punctuation(FirrtlPunctuation),
    /// File info
    FileInfo(&'src str),
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

    fn firrtl_lex(&mut self, cur: LogosToken<'src>) -> Option<Result<FirrtlToken<'src>>> {
        println!("Start at {:?}, {:?}", cur, self.llexer.span());
        loop { 
            match cur { 
                LogosToken::IdentKw(s)  => {
                    return Some(Ok(FirrtlToken::IdentKw(s)));
                },
                LogosToken::FileInfo(s) => {
                    return Some(Ok(FirrtlToken::FileInfo(s)));
                },
                LogosToken::LiteralInt(s) => {
                    return Some(Ok(FirrtlToken::Literal(FirrtlLiteral::Int(s))));
                },
                LogosToken::LiteralSignedInt(s) => {
                    return Some(Ok(FirrtlToken::Literal(FirrtlLiteral::SignedInt(s))));
                },
                LogosToken::StringLiteral(s) => {
                    return Some(Ok(FirrtlToken::Literal(FirrtlLiteral::String(s))));
                },
                LogosToken::RawString(s) => {
                    return Some(Ok(FirrtlToken::Literal(FirrtlLiteral::RawString(s))));
                },
                lt if FirrtlPunctuation::from_lt(&lt).is_some() => {
                    return Some(Ok(FirrtlToken::Punctuation(
                        FirrtlPunctuation::from_lt(&lt).unwrap()
                    )))
                },
                _ => unimplemented!("{:?}", cur),
            }
        }
    }

}

impl <'src> Iterator for FirrtlLexer<'src> {
    type Item = Result<FirrtlToken<'src>>;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let span = self.llexer.span();

        if let Some(ltok_res) = self.llexer.next() {
            match ltok_res {
                // Fancy spanned errors via [miette]
                Err(ltok_err) => {
                    let src = String::from_str(self.src).unwrap();
                    let rsrc = NamedSource::new(self.filename.as_str(), src);
                    return Some(Err(LexerError {
                        src: rsrc, span: (span.start, span.len()).into()
                    }.into()));
                },
                Ok(ltok) => self.firrtl_lex(ltok),
            }

        } else { 
            None
        }

    }
}


//    pub fn lex_from_file(filename: &str) -> Result<()> {
//        let s = Self::string_from_file(filename);
//        let input = s.clone();
//
//        // Use [logos] to create a stream of tokens
//        let mut lexer = LogosToken::lexer(&input);
//        return lexer;
//
//        loop { 
//            let span = lexer.span();
//            match lexer.next() {
//                Some(t) => match t {
//                    Ok(t) => { 
//                        println!("{:?} {:?}", t, span);
//                    },
//                    Err(e) => {
//                        Err(LexerError {
//                            src: NamedSource::new("parse-basic.fir", s.clone()),
//                            span: (span.start, span.len()).into(),
//                        })?;
//                    },
//                },
//                None => break,
//            }
//        }
//        Ok(())
//
//
//
//    }
//
//}


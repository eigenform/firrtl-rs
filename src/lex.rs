//! FIRRTL lexing/tokenization

use std::ops::Range;
use std::collections::BTreeSet;
use logos::Logos;

use crate::file::*;
use crate::token::*;


/// A fully-tokenized FIRRTL line, corresponding to a single [FirrtlLine] 
/// contained in a [FirrtlFile]. 
///
/// # Implementation Details
///
/// - Indentation for the line is separate from the list of tokens
/// - FIRRTL "file info" is separate from the list of tokens
///
#[derive(Debug)]
pub struct FirrtlTokenizedLine {
    /// List of tokens
    pub tokens: Vec<Token>,
    /// Indentation level of this line
    pub indent_level: usize,
    /// Optional FIRRTL-defined source file info
    pub info: Option<String>,
    /// Original line number in the source .fir file
    pub sf_line: usize,
    /// The span of each [Token] in the original source file content
    pub spans: Vec<Range<usize>>,
    /// The original string before tokenization
    pub content: String,
}
impl FirrtlTokenizedLine {
    /// Returns the indentation level of this line
    pub fn indent_level(&self) -> usize { 
        self.indent_level
    }
    /// Returns the corresponding line number in the original source file
    pub fn line_number(&self) -> usize { 
        self.sf_line
    }
    /// Returns the number of tokens in this line
    pub fn len(&self) -> usize { 
        self.tokens.len()
    }
    /// Returns the content that produced this tokenized line
    pub fn content(&self) -> &str { 
        &self.content
    }
}

/// FIRRTL parser error code.
#[derive(Debug)]
pub enum ParseErrorKind {
    ExpectedToken(String),
    ExpectedKeyword(String),
    ExpectedPunctuation(String),
    Other(String),
}

/// FIRRTL parser error.
#[derive(Debug)]
pub struct FirrtlParseError {
    /// The type of error
    pub kind: ParseErrorKind,
    /// The span/context for this error
    pub span: Range<usize>,
}

/// State used to implement a parser over some set of [FirrtlTokenizedLine].
pub struct FirrtlStream<'a> {
    /// Reference to some tokenized FIRRTL source file.
    file: &'a FirrtlFile,

    /// The set of tokenized lines in the stream
    //lines: &'a [FirrtlTokenizedLine],

    /// The total number of tokenized lines in the stream
    length: usize,

    /// Per-module context for allowing a parser to resolve ambiguity between 
    /// "identifiers" and "keywords".
    ///
    /// NOTE: At some point, it would be nice if we didn't need this
    module_ctx: BTreeSet<&'a str>,

    /// The index of the current line
    gcur: usize,
    /// The index of the current token [within the current line]
    lcur: usize,
}
impl <'a> FirrtlStream<'a> {
    //pub fn new(file: &'a FirrtlFile, lines: &'a [FirrtlTokenizedLine]) -> Self { 
    pub fn new(file: &'a FirrtlFile) -> Self { 
        Self { 
            file,
            length: file.lines.len(),
            module_ctx: BTreeSet::new(),
            gcur: 0,
            lcur: 0,
        }
    }

    pub fn clear_module_ctx(&mut self) {
        self.module_ctx.clear();
    }
    pub fn check_module_ctx(&self, kw: &'a str) -> bool {
        self.module_ctx.get(kw).is_some()
    }
    pub fn add_module_ctx(&mut self, kw: &'a str) {
        self.module_ctx.insert(kw);
    }
}

impl <'a> FirrtlStream<'a> {
    /// Move to the next token in the stream. 
    pub fn next_token(&mut self) {
        if self.lcur == self.line().len() - 1 {
            self.gcur += 1;
            self.lcur  = 0;
        } else { 
            self.lcur += 1;
        }
    }

    /// Explicitly move to the next line in the stream. 
    pub fn next_line(&mut self) {
        self.gcur += 1;
        self.lcur  = 0;
        assert!(self.gcur < self.length);
    }

    /// Get the current line
    pub fn line(&self) -> &'a FirrtlTokenizedLine {
        &self.file.lines[self.gcur]
    }

    /// Returns 'true' when the current cursor points to the start of a line.
    pub fn is_sol(&self) -> bool {
        self.lcur == 0
    }

    /// Get the current token
    pub fn token(&self) -> &'a Token {
        &self.file.lines[self.gcur].tokens[self.lcur]
    }

    /// Get a slice of the remaining tokens on the current line
    pub fn remaining_tokens(&self) -> &'a [Token] {
        &self.file.lines[self.gcur].tokens[self.lcur..]
    }

    /// Get the current indentation level.
    ///
    /// NOTE: Returns `0` if the stream has reached EOF.
    pub fn indent_level(&self) -> usize {
        if self.gcur >= self.length {
            0
        } else {
            self.file.lines[self.gcur].indent_level()
        }
    }

    /// Peek at the token 'N'-steps ahead of the cursor. 
    pub fn peekn_token(&self, n: usize) -> &'a Token {
        assert!(self.lcur + n < self.line().len());
        &self.file.lines[self.gcur].tokens[self.lcur + n]
    }

}

/// For recovering the span from the original file during error-handling.
impl <'a> FirrtlStream<'a> {
    fn get_source_line(&self) -> usize { 
        self.line().sf_line
    }

    fn get_source_span(&self) -> Range<usize> {
        let s = self.line().spans[self.lcur].start;
        let e = self.line().spans[self.lcur].end;
        s..e
    }

    fn err_keyword(&self, kw: &str) -> FirrtlParseError {
        FirrtlParseError {
            kind: ParseErrorKind::ExpectedKeyword(kw.to_string()),
            span: self.get_source_span(),
        }
    }
    fn err_punct(&self, kw: &str) -> FirrtlParseError {
        FirrtlParseError {
            kind: ParseErrorKind::ExpectedPunctuation(kw.to_string()),
            span: self.get_source_span(),
        }
    }
    fn err_token(&self, kw: &str) -> FirrtlParseError {
        FirrtlParseError {
            kind: ParseErrorKind::ExpectedToken(kw.to_string()),
            span: self.get_source_span(),
        }
    }
    fn err_other(&self, kw: &str) -> FirrtlParseError {
        FirrtlParseError {
            kind: ParseErrorKind::Other(kw.to_string()),
            span: self.get_source_span(),
        }
    }
}

/// All of these methods attempt to *match* the underlying data from a [Token]. 
impl <'a> FirrtlStream<'a> {
    pub fn match_punc(&self, p: &'a str) -> Result<(), FirrtlParseError> {
        if self.token() == &Token::punctuation_from_str(p) {
            Ok(())
        } else { 
            Err(self.err_punct(p))
        }
    }

    pub fn match_identkw(&self, kw: &'a str) -> Result<(), FirrtlParseError> {
        let idkw = self.get_identkw()?;
        if idkw == kw {
            Ok(())
        } else { 
            Err(self.err_keyword(kw))
        }
    }

    /// Return the first matching keyword
    pub fn match_identkw_multi(&self, kw: &[&'a str]) 
        -> Result<&'a str, FirrtlParseError>
    {
        let idkw = self.get_identkw()?;
        if let Some(m) = kw.iter().find(|k| *k == &idkw) {
            Ok(m)
        } else {
            Err(self.err_keyword(&format!("{:?}", kw)))
        }
    }
}

/// All of these methods attempt to *read* the underlying data from a [Token]. 
impl <'a> FirrtlStream<'a> {
    pub fn get_identkw(&self) -> Result<&'a str, FirrtlParseError> {
        if let Token::IdentKw(s) = self.token() {
            Ok(s)
        } else { 
            Err(self.err_token("identifier/keyword"))
        }
    }
    pub fn get_lit_int(&self) -> Result<&'a str, FirrtlParseError> {
        if let Some(lit) = self.token().get_lit_int() {
            Ok(lit)
        } else { 
            Err(self.err_token("integer literal"))
        }
    }
    pub fn get_lit_sint(&self) -> Result<&'a str, FirrtlParseError> {
        if let Some(lit) = self.token().get_lit_sint() {
            Ok(lit)
        } else { 
            Err(self.err_token("signed integer literal"))
        }
    }
    pub fn get_lit_float(&self) -> Result<&'a str, FirrtlParseError> {
        if let Some(lit) = self.token().get_lit_float() {
            Ok(lit)
        } else { 
            Err(self.err_token("floating-point literal"))
        }
    }
    pub fn get_lit_str(&self) -> Result<&'a str, FirrtlParseError> {
        if let Some(lit) = self.token().get_lit_str() {
            Ok(lit)
        } else {
            Err(self.err_token("string literal"))
        }
    }
    pub fn get_lit_raw_str(&self) -> Result<&'a str, FirrtlParseError> {
        if let Some(lit) = self.token().get_raw_str() {
            Ok(lit)
        } else {
            Err(self.err_token("raw string literal"))
        }
    }
}



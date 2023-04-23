
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
    tokens: Vec<Token>,
    /// Indentation level of this line
    indent_level: usize,
    /// Optional FIRRTL-defined source file info
    info: Option<String>,
    /// Original line number in the source .fir file
    sf_line: usize,
    /// The span of each [Token] in the original source file content
    spans: Vec<Range<usize>>,
    /// The original string before tokenization
    content: String,
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

/// Used to convert a [FirrtlFile] into a set of [FirrtlTokenizedLine].
///
/// # Implementation Details
///
/// It's a lot easier to deal with things line-by-line, since FIRRTL
/// depends on indentation (and otherwise doesn't care about whitespace).
///
pub struct FirrtlLexer;
impl FirrtlLexer {

    /// Tokenize each [FirrtlLine] in the provided [FirrtlFile], producing a 
    /// list of [FirrtlTokenizedLine].
    pub fn lex(sf: &FirrtlFile) -> Vec<FirrtlTokenizedLine> {
        let mut tokenized_lines = Vec::new();

        for sfl in &sf.lines {
            let sf_line       = sfl.line_number();
            let sf_line_start = sfl.line_start();
            let indent_level  = sfl.indent_level();

            // FIRRTL "file info" optionally comes at the end of a line. 
            // Separate meaningful line content from any file info.
            let (content, info) = if let Some(idx) = sfl.contents().find('@') {
                (&sfl.contents()[..idx], Some(sfl.contents()[idx..].to_string()))
            } else {
                (sfl.contents(), None)
            };

            // Extract a set of tokens/spans from each line
            let mut tokens = Vec::new();
            let mut spans  = Vec::new();
            let mut lexer = Token::lexer(&content);
            while let Some(t) = lexer.next() {
                let perline_span = lexer.span();
                let start = sf_line_start + perline_span.start;
                let end   = sf_line_start + perline_span.end;
                let token_span = start..end;
                match t {
                    Ok(token) => {
                        tokens.push(token);
                        spans.push(token_span);
                    },
                    // Some error occured while tokenizing this line.
                    // FIXME: Proper error-handling instead of panic!()
                    Err(e) => {
                        println!("{:?}", e);
                        panic!("unknown token at line {}, offset {:?}",
                               sf_line, token_span);
                    },
                }
            }
            let tokenized_line = FirrtlTokenizedLine {
                tokens, spans, sf_line, info, indent_level,
                content: content.to_string(),
            };
            tokenized_lines.push(tokenized_line);
        }
        tokenized_lines
    }
}

/// FIRRTL parse error.
///
/// NOTE: At some point, you might want fancy spanned errors 
/// (ie. with [miette] or something similar)
#[derive(Debug)]
pub enum FirrtlStreamErr {
    ExpectedToken(String),
    ExpectedKeyword(String),
    ExpectedPunctuation(String),
    Other(&'static str),
}

/// State used to implement a parser over some set of [FirrtlTokenizedLine].
pub struct FirrtlStream<'a> {
    /// The set of tokenized lines in the stream
    lines: &'a [FirrtlTokenizedLine],
    /// The total number of tokenized lines in the stream
    length: usize,

    /// Per-module context for allowing a parser to resolve ambiguity between 
    /// "identifiers" and "keywords"
    module_ctx: BTreeSet<&'a str>,
    /// The index of the current line
    gcur: usize,
    /// The index of the current token [within the current line]
    lcur: usize,
}
impl <'a> FirrtlStream<'a> {
    pub fn new(lines: &'a [FirrtlTokenizedLine]) -> Self { 
        Self { 
            lines,
            length: lines.len(),
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

    pub fn next_token(&mut self) {
        if self.lcur == self.line().len() - 1 {
            println!("[*] Moved to next line");
            self.gcur += 1;
            self.lcur  = 0;
        } else { 
            self.lcur += 1;
        }
        //assert!(self.lcur < self.lines[self.gcur].tokens.len());
    }
    pub fn next_line(&mut self) {
        self.gcur += 1;
        self.lcur  = 0;
        assert!(self.gcur < self.length);
    }

    /// Get the current line
    pub fn line(&self) -> &'a FirrtlTokenizedLine {
        &self.lines[self.gcur]
    }

    /// Returns 'true' when the current cursor points to the start of a line.
    pub fn is_sol(&self) -> bool {
        self.lcur == 0
    }

    /// Get the current token
    pub fn token(&self) -> &'a Token {
        &self.lines[self.gcur].tokens[self.lcur]
    }

    /// Get a slice of the remaining tokens on the current line
    pub fn remaining_tokens(&self) -> &'a [Token] {
        &self.lines[self.gcur].tokens[self.lcur..]
    }

    /// Get the current indentation level
    pub fn indent_level(&self) -> usize {
        self.lines[self.gcur].indent_level()
    }

    pub fn peek_line(&self) -> &'a FirrtlTokenizedLine {
        &self.lines[self.gcur + 1]
    }

    pub fn peek_token(&self) -> &'a Token {
        &self.lines[self.gcur].tokens[self.lcur + 1]
    }

    pub fn peekn_token(&self, n: usize) -> &'a Token {
        &self.lines[self.gcur].tokens[self.lcur + n]
    }


    pub fn get_identkw(&self) -> Result<&'a str, FirrtlStreamErr> {
        if let Token::IdentKw(s) = self.token() {
            Ok(s)
        } else { 
            let e = format!("expected Token::IdentKw, got {:?}",
                            self.token());
            Err(FirrtlStreamErr::ExpectedToken(e))
        }
    }

    pub fn get_lit_int(&self) -> Result<&'a str, FirrtlStreamErr> {
        if let Some(lit) = self.token().get_lit_int() {
            Ok(lit)
        } else { 
            Err(FirrtlStreamErr::ExpectedToken("expected lit int".to_string()))
        }
    }
    pub fn get_lit_sint(&self) -> Result<&'a str, FirrtlStreamErr> {
        if let Some(lit) = self.token().get_lit_sint() {
            Ok(lit)
        } else { 
            Err(FirrtlStreamErr::ExpectedToken("expected lit sint".to_string()))
        }
    }


    pub fn get_lit_float(&self) -> Result<&'a str, FirrtlStreamErr> {
        if let Some(lit) = self.token().get_lit_float() {
            Ok(lit)
        } else { 
            Err(FirrtlStreamErr::ExpectedToken("expected lit flt".to_string()))
        }
    }


    pub fn get_lit_str(&self) -> Result<&'a str, FirrtlStreamErr> {
        if let Some(lit) = self.token().get_lit_str() {
            Ok(lit)
        } else {
            Err(FirrtlStreamErr::ExpectedToken("expected lit str".to_string()))
        }
    }

    pub fn match_identkw(&self, kw: &'a str) -> Result<(), FirrtlStreamErr> {
        let idkw = self.get_identkw()?;
        if idkw == kw {
            Ok(())
        } else { 
            Err(FirrtlStreamErr::ExpectedKeyword(kw.to_string()))
        }
    }

    pub fn match_punc(&self, p: &'a str) -> Result<(), FirrtlStreamErr> {
        if self.token() == &Token::punctuation_from_str(p) {
            Ok(())
        } else { 
            Err(FirrtlStreamErr::ExpectedPunctuation(p.to_string()))
        }
    }

}



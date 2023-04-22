
use std::ops::Range;
use logos::Logos;

use crate::file::*;
use crate::token::*;

/// A "tokenized" line in some [FirrtlFile], with optional FIRRTL file info.
#[derive(Debug)]
pub struct FirrtlTokenizedLine {
    /// List of tokens
    tokens: Vec<Token>,

    /// The original string before tokenization
    content: String,

    /// List of spans [in the original source .fir file] for tokens
    spans: Vec<Range<usize>>,

    /// Optional source file info embedded in FIRRTL
    info: Option<String>,

    /// Original line number in the source .fir file
    sf_line: usize,

    /// Indentation level of this line
    indent_level: usize,
}
impl FirrtlTokenizedLine {
    pub fn indent_level(&self) -> usize { 
        self.indent_level
    }
    pub fn line_number(&self) -> usize { 
        self.sf_line
    }
    pub fn len(&self) -> usize { 
        self.tokens.len()
    }
}

pub struct FirrtlLexer;
impl FirrtlLexer {
    // It's a lot easier to deal with things line-by-line, since FIRRTL
    // depends on indentation (and doesn't care about whitespace otherwise). 
    pub fn lex(sf: &FirrtlFile) -> Vec<FirrtlTokenizedLine> {
        let mut tokenized_lines = Vec::new();

        for sfl in &sf.lines {
            let sf_line       = sfl.line_number();
            let sf_line_start = sfl.line_start();
            let indent_level  = sfl.indent_level();

            // File info optionally comes at the end of a line. 
            // Separate actual line content from FIRRTL file info.
            let (content, info) = if let Some(idx) = sfl.line.find('@') {
                (&sfl.line[..idx], Some(sfl.line[idx..].to_string()))
            } else {
                (sfl.line.as_str(), None)
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

#[derive(Debug)]
pub enum FirrtlStreamErr {
    ExpectedToken(String),
    ExpectedKeyword(String),
    ExpectedPunctuation(String),

    Other(&'static str),
}


pub struct FirrtlStream<'a> {
    lines: &'a [FirrtlTokenizedLine],
    length: usize,
    gcur: usize,
    lcur: usize,
}
impl <'a> FirrtlStream<'a> {
    pub fn new(lines: &'a [FirrtlTokenizedLine]) -> Self { 
        Self { 
            lines,
            length: lines.len(),
            gcur: 0,
            lcur: 0,
        }
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
    /// Get the current token
    pub fn token(&self) -> &'a Token {
        &self.lines[self.gcur].tokens[self.lcur]
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

    pub fn get_identkw(&self) -> Result<&'a str, FirrtlStreamErr> {
        if let Token::IdentKw(s) = self.token() {
            Ok(&s)
        } else { 
            Err(FirrtlStreamErr::ExpectedToken("expected identkw".to_string()))
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



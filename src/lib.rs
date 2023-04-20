#![allow(unused_braces)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(unused_imports)]

pub mod ast;
pub mod lexer;

/// Read a file into a [String].
fn read_file(filename: &str) -> String {
    use std::fs::File;
    use std::io::*;

    let mut f = File::open(filename).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

use crate::lexer::*;
use std::collections::VecDeque;
use miette::{ Diagnostic, SourceSpan, SourceOffset, NamedSource, Result };
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("Parser error")]
pub enum ParserError<'src> {
    ExpectedToken(FirrtlToken<'src>),
    ExpectedKeyword(&'src str),
}
pub struct FirrtlParser<'src> {
    /// Index of the current token
    cur: usize,

    indent_stack: VecDeque<usize>,

    /// List of tokens to visit
    tokens: Vec<FirrtlToken<'src>>,
}
impl <'src, 'a> FirrtlParser<'src> {
    /// Return a reference to the current [FirrtlToken]
    fn token(&self) -> &FirrtlToken<'src> {
        &self.tokens[self.cur]
    }
    /// Return a reference to some *next* [FirrtlToken]
    fn peek_token(&self, n: usize) -> &FirrtlToken<'src> {
        &self.tokens[self.cur + n]
    }

    pub fn consume_punctuation(&mut self, p: FirrtlPunctuation) -> Result<()> {
        match *self.token() {
            FirrtlToken::Punctuation(x) if x == p => {
                self.cur += 1;
                Ok(())
            },
            _ => panic!("expected {:?}, got {:?}", p, self.token()),
        }
    }

    pub fn consume_whitespace(&mut self) -> Result<()> {
        match *self.token() {
            FirrtlToken::WhitespaceH(n) => {
                self.cur += 1;
                Ok(())
            },
            _ => panic!("expected whitespace, got {:?}", self.token()),
        }
    }

    pub fn consume_int_lit(&mut self) -> Result<&'src str> {
        match *self.token() {
            FirrtlToken::Literal(FirrtlLiteral::Int(s)) => {
                self.cur += 1;
                Ok(s)
            },
            _ => panic!("expected whitespace, got {:?}", self.token()),
        }
    }


    pub fn consume_keyword(&mut self, kw: &str) -> Result<()> {
        let token = &self.tokens[self.cur];
        let kw_token = FirrtlToken::IdentKw(kw);
        if *self.token() == kw_token {
            self.cur += 1;
            Ok(())
        } else { 
            panic!("{:?} {:?}", ParserError::ExpectedKeyword(kw), self.token());
        }
    }

    pub fn consume_ident(&mut self) -> Result<&'src str> {
        match *self.token() {
            FirrtlToken::IdentKw(s) => {
                self.cur += 1;
                Ok(s)
            },
            _ => panic!("expected ident, found {:?}", self.token()),
        }
    }

    pub fn consume_version(&mut self) -> Result<&'src str> {
        match *self.token() {
            FirrtlToken::Literal(FirrtlLiteral::Version(s)) => {
                self.cur += 1;
                Ok(s)
            },
            _ => panic!("expected version number"),
        }
    }

    pub fn maybe_parse_fileinfo(&mut self) -> Option<&'src str> {
        match *self.token() {
            FirrtlToken::FileInfo(s) => {
                self.cur += 1;
                Some(s)
            },
            _ => None,
        }
    }


    pub fn parse_firrtl_version(&mut self) -> Result<()> {
        self.consume_keyword("FIRRTL")?;
        self.consume_keyword("version")?;
        self.consume_version()?;
        Ok(())
    }

    pub fn parse_circuit(&mut self) -> Result<()> {
        self.consume_keyword("circuit")?;
        let circuit_id = self.consume_ident()?;
        self.consume_punctuation(FirrtlPunctuation::Colon)?;

        loop {
            match *self.token() {
                FirrtlToken::EOF => break,
                FirrtlToken::WhitespaceH(n) => {
                    let m_indent = self.consume_whitespace()?;
                    self.parse_module()?;
                },
                _ => panic!("unexpected in circuit {:?}", self.token()),
            }
        }
        self.parse_module()?;

        Ok(())
    }

    pub fn parse_module(&mut self) -> Result<()> {
        match *self.token() {
            FirrtlToken::IdentKw("module") => {
                self.consume_keyword("module")?;
                let module_id = self.consume_ident()?;
                self.consume_punctuation(FirrtlPunctuation::Colon)?;
                let info = self.maybe_parse_fileinfo();

                self.parse_portlist()?;
                self.parse_statements()?;
                unimplemented!("module");
            },
            FirrtlToken::IdentKw("extmodule") => {
                unimplemented!("extmodule");
            },
            FirrtlToken::IdentKw("intmodule") => {
                unimplemented!("intmodule");
            },
            _ => panic!("unexpected token {:?}", self.token()),
        }
        Ok(())
    }

    pub fn parse_statements(&mut self) -> Result<()> {
        match *self.token() {
            _ => panic!("statements {:?}", self.token()),
        }
    }

    pub fn parse_portlist(&mut self) -> Result<()> {
        loop {

            // Determine if the next four tokens qualify as a port decl.
            // and terminate the loop otherwise
            let have_ws = self.token().is_whitespace();
            let have_dir = match self.peek_token(1) {
                FirrtlToken::IdentKw(s) => {
                    if *s == "input" || *s == "output" { true } else { false }
                },
                _ => false,
            };
            let have_ident = self.peek_token(2).is_identkw();
            let have_colon = self.peek_token(3)
                .is_punctuation(FirrtlPunctuation::Colon);
            if !(have_ws && have_dir && have_ident && have_colon) {
                break;
            }

            println!("parsing port ...");
            let indent = self.consume_whitespace()?;
            match *self.token() {
                FirrtlToken::IdentKw(s) if s == "input" || s == "output" => {
                    self.consume_keyword(s)?;
                    let port_name = self.consume_ident()?;
                    self.consume_punctuation(FirrtlPunctuation::Colon)?;
                    let port_type = self.parse_type()?;
                },
                _ => panic!("unexpected {:?}", self.token()),
            }
        }
        Ok(())
    }

    pub fn parse_type(&mut self) -> Result<()> {
        match *self.token() {
            FirrtlToken::IdentKw("Clock") => {
                self.consume_keyword("Clock")?;
                Ok(())
            },
            FirrtlToken::IdentKw("Reset") => {
                self.consume_keyword("Reset")?;
                Ok(())
            },
            FirrtlToken::IdentKw("AsyncReset") => {
                self.consume_keyword("AsyncReset")?;
                Ok(())
            },
            FirrtlToken::IdentKw(t) if t == "UInt" || t == "SInt" || t == "Analog" => {
                self.consume_keyword(t)?;
                let width = self.maybe_parse_width();

                Ok(())
            },
            _ => panic!("expected type, got {:?}", self.token()),
        }
    }

    pub fn maybe_parse_width(&mut self) -> Option<usize> {
        match *self.token() {
            FirrtlToken::TypeWidth(w) => {
                self.cur += 1;
                Some(w)
            },
            _ => None,
        }
    }

}


#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::*;
    use miette::{ SourceSpan, SourceOffset, NamedSource, Result };

    #[test]
    fn firrtl_parse_test() -> Result<()> {

        let src = read_file("parse-basic.fir");
        let lexer = FirrtlLexer::new("parse-basic.fir", &src);

        // Collect all of the tokens (or exit with some lexer error)
        let mut tokens = Vec::new();
        for t in lexer {
            match t {
                Err(e) => return Err(e),
                Ok(ft) => {
                    println!("{:?}", ft);
                    tokens.push(ft);
                },
            }
        }
        tokens.push(FirrtlToken::EOF);

        let mut p = FirrtlParser { 
            cur: 0, 
            indent_stack: VecDeque::new(),
            tokens,
        };
        p.parse_firrtl_version()?;
        p.parse_circuit()?;

        Ok(())

    }

}




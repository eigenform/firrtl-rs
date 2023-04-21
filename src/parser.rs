
use std::collections::VecDeque;
use miette::{ Diagnostic, SourceSpan, SourceOffset, NamedSource, Result };
use thiserror::Error;

use crate::lexer::*;

#[derive(Error, Debug, Diagnostic)]
#[error("Parser error")]
pub enum ParserError<'src> {
    ExpectedToken(FirrtlToken<'src>),
    ExpectedKeyword(&'src str),
}

/// State used to parse a FIRRTL file.
pub struct FirrtlParser<'src> {
    /// Index of the current token
    cur: usize,
    /// Current indentation level
    indent_stack: usize,
    /// List of tokens to visit
    tokens: Vec<FirrtlToken<'src>>,
}

/// These are helper functions used to manage the parser state when 
/// we're passing over all of the tokens. 
impl <'src, 'a> FirrtlParser<'src> {
    /// Set the current indentation level
    fn set_current_indent(&mut self, x: usize) {
        self.indent_stack = x;
    }
    /// Get the current indentation level
    fn get_current_indent(&self) -> usize { 
        self.indent_stack
    }

    /// Return a reference to the current [FirrtlToken]
    fn token(&self) -> &FirrtlToken<'src> {
        &self.tokens[self.cur]
    }
    /// Return a reference to some *next* [FirrtlToken]
    fn peek_token(&self, n: usize) -> &FirrtlToken<'src> {
        &self.tokens[self.cur + n]
    }

    /// Move cursor to the next [FirrtlToken] in the stream.
    fn consume_token(&mut self) -> Result<()> {
        // NOTE: Transparently skips any whitespace tokens and update
        // the current indentation level?
        //if let FirrtlToken::WhitespaceH(n) = self.token() {
        //    self.set_current_indent(*n);
        //}
        println!("Consumed {:?}", self.token());
        self.cur += 1;
        Ok(())
    }

    pub fn peek_whitespace(&self) -> Option<usize> {
        if let FirrtlToken::WhitespaceH(n) = *self.token()
        { Some(n) } else { None }
    }
    pub fn peek_punct(&self) -> Option<FirrtlPunct> {
        if let FirrtlToken::Punct(p) = *self.token()
        { Some(p) } else { None }
    }
    pub fn peek_ident(&self) -> Option<&'src str> {
        if let FirrtlToken::IdentKw(s) = *self.token() 
        { Some(s) } else { None }
    }

    pub fn consume_punct(&mut self, p: FirrtlPunct) -> Result<()> {
        if self.token().punct_matches(p) {
            self.consume_token()
        } else {
            panic!("expected {:?}, got {:?}", p, self.token());
        }
    }
    pub fn consume_whitespace(&mut self) -> Result<usize> {
        match *self.token() {
            FirrtlToken::WhitespaceH(n) => {
                self.consume_token()?;
                Ok(n)
            },
            _ => panic!("expected whitespace, got {:?}", self.token()),
        }
    }
    pub fn consume_int_lit(&mut self) -> Result<&'src str> {
        match *self.token() {
            FirrtlToken::Literal(FirrtlLiteral::Int(s)) => {
                self.consume_token()?;
                Ok(s)
            },
            _ => panic!("expected int lit, got {:?}", self.token()),
        }
    }
    pub fn consume_keyword(&mut self, kw: &'src str) -> Result<()> {
        if self.token().identkw_matches(kw) {
            self.consume_token()?;
            Ok(())
        } else {
            panic!("{:?} {:?}", ParserError::ExpectedKeyword(kw), self.token());
        }
    }
    pub fn consume_ident(&mut self) -> Result<&'src str> {
        match *self.token() {
            FirrtlToken::IdentKw(s) => {
                self.consume_token()?;
                Ok(s)
            },
            _ => panic!("expected ident, found {:?}", self.token()),
        }
    }
    pub fn consume_version(&mut self) -> Result<&'src str> {
        match *self.token() {
            FirrtlToken::Literal(FirrtlLiteral::Version(s)) => {
                self.consume_token()?;
                Ok(s)
            },
            _ => panic!("expected version number"),
        }
    }
    pub fn maybe_parse_fileinfo(&mut self) -> Option<&'src str> {
        match *self.token() {
            FirrtlToken::FileInfo(s) => {
                self.consume_token().unwrap();
                Some(s)
            },
            _ => None,
        }
    }
    pub fn maybe_parse_width(&mut self) -> Option<usize> {
        let lt = self.token().punct_matches(FirrtlPunct::Less);
        let lit = self.peek_token(1).is_int_lit();
        let gt = self.peek_token(2).punct_matches(FirrtlPunct::Greater);
        if !(lt && lit && gt) {
            return None;
        }
        self.consume_punct(FirrtlPunct::Less).unwrap();
        let width = self.consume_int_lit().unwrap();
        self.consume_punct(FirrtlPunct::Greater).unwrap();
        let width = width.parse().unwrap();
        Some(width)
    }

}

/// These are the main methods used to build the AST. 
impl <'src, 'a> FirrtlParser<'src> {
    pub fn parse_firrtl_version(&mut self) -> Result<()> {
        self.consume_keyword("FIRRTL")?;
        self.consume_keyword("version")?;
        self.consume_version()?;
        Ok(())
    }

    pub fn parse_circuit(&mut self) -> Result<()> {
        assert!(self.token().identkw_matches("circuit"));

        self.consume_keyword("circuit")?;
        let circuit_id = self.consume_ident()?;
        self.consume_punct(FirrtlPunct::Colon)?;
        println!("[*] Parsing circuit {}", circuit_id);

        loop {
            let m_indent = self.consume_whitespace()?;
            self.set_current_indent(m_indent);

            match *self.token() {
                FirrtlToken::EOF => break,
                FirrtlToken::IdentKw("module") => {
                    let module = self.parse_module()?;
                },
                FirrtlToken::IdentKw("extmodule") => {
                    let extmodule = self.parse_extmodule()?;
                },
                FirrtlToken::IdentKw("intmodule") => {
                    let intmodule = self.parse_intmodule()?;
                }
                FirrtlToken::WhitespaceH(x) => {
                    if x != m_indent { panic!("unexpected circut idt?"); }
                    self.consume_whitespace()?;
                },
                _ => panic!("unexpected in circuit {:?}", self.token()),
            }
        }

        Ok(())
    }
    pub fn parse_extmodule(&mut self) -> Result<()> {
        self.consume_keyword("extmodule")?;
        let id = self.consume_ident()?;
        self.consume_punct(FirrtlPunct::Colon)?;
        let info = self.maybe_parse_fileinfo();
        self.parse_portlist()?;

        // defname is optional
        if let Some(idt) = self.peek_whitespace() {
            // This must be a module without any statements
            if idt < self.get_current_indent() { 
                return Ok(()); 
            }
        } else {
            panic!("expected whitespace, got {:?}", self.token());
        }

        if self.token().identkw_matches("defname") {
            self.consume_keyword("defname")?;
            self.consume_punct(FirrtlPunct::Equal)?;
            let defname_id = self.consume_ident()?;
        }

        // optional list of parameter declarations


        // optional list of ref declarations


        unimplemented!();
    }
    pub fn parse_intmodule(&mut self) -> Result<()> {
        unimplemented!();
    }


    pub fn parse_module(&mut self) -> Result<()> {
        assert!(self.token().identkw_matches("module"));

        self.consume_keyword("module")?;
        let module_id = self.consume_ident()?;
        self.consume_punct(FirrtlPunct::Colon)?;
        let info = self.maybe_parse_fileinfo();

        self.parse_portlist()?;

        if let Some(idt) = self.peek_whitespace() {
            // This must be a module without any statements
            if idt < self.get_current_indent() { return Ok(()); }
        } else {
            panic!("No whitespace before start of statements?")
        }

        self.parse_statements()?;

        Ok(())
    }

    pub fn parse_portlist(&mut self) -> Result<()> {

        let indent = if let Some(indent) = self.peek_whitespace() {
            if indent > self.get_current_indent() {
                indent
            } else {
                panic!("portlist not indented?");
            }
        } else { 
            panic!("expected whitespace");
        };

        loop {
            // Determine if the next four tokens qualify as a port declaration.
            // If this is something else, we're done parsing the portlist.
            let have_ws = self.token().is_whitespace();
            let have_dir = match self.peek_token(1) {
                FirrtlToken::IdentKw("input")  => true,
                FirrtlToken::IdentKw("output") => true,
                _ => false,
            };
            let have_ident = self.peek_token(2).is_identkw();
            let have_colon = self.peek_token(3)
                .punct_matches(FirrtlPunct::Colon);
            println!("{} {} {} {}", have_ws, have_dir, have_ident, have_colon);
            if !(have_ws && have_dir && have_ident && have_colon) {
                break;
            }

            let port_indent = self.consume_whitespace()?;
            if port_indent != indent { panic!("bad indent?"); }
            self.set_current_indent(port_indent);
            let port = self.parse_port()?;
        }
        Ok(())
    }

    pub fn parse_port(&mut self) -> Result<()> {
        let dir = match self.consume_ident()? {
            "input" => {},
            "output" => {},
            _ => panic!("expected input or output"),
        };
        let port_name = self.consume_ident()?;
        self.consume_punct(FirrtlPunct::Colon)?;
        let port_type = self.parse_type()?;
        Ok(())
    }


    pub fn parse_statements(&mut self) -> Result<()> {
        let indent = if let Some(indent) = self.peek_whitespace() {
            if indent == self.get_current_indent() {
                indent
            } else {
                panic!("statements: expected ws {}, got {}", 
                       self.get_current_indent(), indent);
            }
        } else { 
            panic!("expected whitespace");
        };

        loop {
            // If this is a dedent, the list of statements is over?
            // Otherwise, this statement has invalid indentation.
            if let Some(i) = self.peek_whitespace() {
                if i < indent { break; }
                if i > indent {
                    panic!("unexpected indent for statement?");
                }
            }
            let stmt_indent = self.consume_whitespace()?;
            assert!(stmt_indent == indent);
            let statement = self.parse_statement()?;
        }
        Ok(())
    }

    pub fn parse_statement(&mut self) -> Result<()> {
        assert!(self.token().is_identkw());

        // Cases where the first element is a *reference* (identifier)
        let is_assign = self.peek_token(1).punct_matches(FirrtlPunct::LessEqual);
        let is_invalidate = self.peek_token(1).identkw_matches("is")
            && self.peek_token(2).identkw_matches("invalid");
        if is_assign {
            let ident = self.consume_ident()?;
            self.consume_punct(FirrtlPunct::LessEqual)?;
            let expr = self.parse_expr()?;
            return Ok(())
        }
        if is_invalidate {
            let ident = self.consume_ident()?;
            self.consume_keyword("is")?;
            self.consume_keyword("invalid")?;
            let info = self.maybe_parse_fileinfo();
            return Ok(())
        }

        // Otherwise, this is a keyword
        match self.consume_ident()? {
            _ => unimplemented!("statement keyword"),
        }

        unreachable!();
    }

    pub fn parse_expr(&mut self) -> Result<()> {
        let is_op = self.peek_token(1).punct_matches(FirrtlPunct::LParen);

        if is_op {
            let ident_op = self.consume_ident()?;
            self.consume_punct(FirrtlPunct::LParen)?;
            unimplemented!("expr op {:?}", ident_op);
        } 
        // Otherwise, this is a reference
        else {
            let reference = self.parse_reference()?;
            Ok(())
        }
    }

    pub fn parse_static_reference(&mut self) -> Result<()> {
        let id = self.consume_ident()?;
        loop {
            if self.token().punct_matches(FirrtlPunct::Period) {
                self.consume_punct(FirrtlPunct::Period)?;
                let field_id = self.consume_ident()?;
                continue;
            }
            if self.token().punct_matches(FirrtlPunct::LSquare) {
                self.consume_punct(FirrtlPunct::LSquare)?;
                let index = self.consume_int_lit()?;
                self.consume_punct(FirrtlPunct::RSquare)?;
                continue;
            }
            break;
        }
        Ok(())
    }

    pub fn parse_reference(&mut self) -> Result<()> {
        let static_ref = self.parse_static_reference()?;
        loop {
            if self.token().punct_matches(FirrtlPunct::LSquare) {
                self.consume_punct(FirrtlPunct::LSquare)?;
                let expr = self.parse_expr()?;
                self.consume_punct(FirrtlPunct::RSquare)?;
                continue;
            }
            break;
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

}


#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::*;
    use miette::{ SourceSpan, SourceOffset, NamedSource, Result };

    /// Read a file into a [String].
    fn read_file(filename: &str) -> String {
        use std::fs::File;
        use std::io::*;

        let mut f = File::open(filename).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    }

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
                    if ft == FirrtlToken::Ignore { continue; }
                    //println!("{:?}", ft);
                    tokens.push(ft);
                },
            }
        }
        tokens.push(FirrtlToken::EOF);

        let mut p = FirrtlParser { 
            cur: 0, 
            indent_stack: 0,
            tokens,
        };
        p.parse_firrtl_version()?;
        p.parse_circuit()?;

        Ok(())

    }

}



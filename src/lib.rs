
pub mod file;
pub mod token;
pub mod lex;
pub mod parse;


//pub struct FirrtlParser<'a> {
//    lines: &'a [FirrtlTokenizedLine],
//    length: usize,
//    lcur: usize,
//    gcur: usize,
//}
//impl <'a> FirrtlParser<'a> {
//    pub fn new(lines: &'a [FirrtlTokenizedLine]) -> Self {
//        Self { 
//            lines,
//            length: lines.len(),
//            lcur: 0,
//            gcur: 0,
//        }
//    }
//}
//impl <'a> FirrtlParser<'a> {
//    /// Return the current line
//    fn current_line(&self) -> &FirrtlTokenizedLine {
//        &self.lines[self.gcur]
//    }
//    /// Increment the local cursor, "consuming" a token.
//    fn consume_token(&mut self) {
//        self.lcur += 1;
//    }
//    /// Increment the global cursor, moving to the next line. 
//    fn consume_line(&mut self) {
//        self.gcur += 1;
//        self.lcur = 0;
//    }
//
//    /// Return the current token
//    fn token(&mut self) -> &Token {
//        &self.lines[self.gcur].tokens[self.lcur]
//    }
//    fn eat_keyword(&mut self, kw: &str) {
//        match self.token() {
//            Token::IdentKw(ref s) => { if kw == s { self.consume_token(); } },
//            _ => panic!("expected keyword {}", kw),
//        }
//    }
//    fn eat_punct(&mut self, p: &str) {
//        if *self.token() == Token::punctuation_from_str(p) {
//            self.consume_token();
//        } else {
//            panic!("expected punctuation '{}'", p);
//        }
//    }
//    fn parse_ident(&mut self) -> &'a str {
//        match self.token() {
//            Token::IdentKw(s) => {
//                self.consume_token();
//                s
//            },
//            _ => panic!("expected ident, got {:?}", self.token()),
//        }
//    }
//
//    pub fn parse(&mut self) {
//        // FIRRTL version
//        let line = self.current_line();
//        assert!(line.indent_level() == 0);
//        println!("Parsing {:?}", line.content);
//        println!("{:?}", line.tokens);
//        self.eat_keyword("FIRRTL");
//        self.eat_keyword("version");
//        self.consume_line();
//
//        // Circuit declaration
//        let line = self.current_line();
//        assert!(line.indent_level() == 0);
//        println!("Parsing {:?}", line.content);
//        println!("{:?}", line.tokens);
//        self.eat_keyword("circuit");
//        let circuit_id = self.parse_ident();
//        self.eat_punct(":");
//        self.consume_line();
//
//        let line = self.current_line();
//        let module_indent = line.indent_level();
//        assert!(module_indent > 0);
//
//        self.eat_keyword("module");
//        let module_id = self.parse_ident();
//        self.eat_punct(":");
//
//    }
//}


#[cfg(test)]
mod tests {
    use crate::file::*;
    use crate::lex::*;
    use crate::parse::*;


    #[test]
    fn read_firrtl_file() -> Result<(), FirrtlStreamErr> {
        use std::fs::File;
        use std::io::*;

        let filename = "./parse-basic.fir";
        let mut f = File::open(filename).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        let sf    = FirrtlFile::new(filename, &s);
        let tok   = FirrtlLexer::lex(&sf);
        let mut stream = FirrtlStream::new(&tok);
        FirrtlParser::parse(&mut stream)
    }
}





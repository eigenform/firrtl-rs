#![allow(unused_parens)]

mod file;
mod token;
mod lex;
mod parse;
pub mod ast;

#[cfg(test)]
mod tests {
    use crate::file::*;
    use crate::lex::*;
    use crate::parse::*;

    #[test]
    fn circt_parse_basic() -> Result<(), FirrtlStreamErr> {
        use std::fs::File;
        use std::io::*;

        let filename = "./parse-basic.fir";
        //let filename = "./chisel-tests/firrtl/GCD.fir";
        let mut f = File::open(filename).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        let sf    = FirrtlFile::new(filename, &s);
        let tok   = FirrtlLexer::lex(&sf);
        let mut stream = FirrtlStream::new(&tok);
        let circuit = FirrtlParser::parse(&mut stream)?;
        circuit.dump();
        Ok(())
    }


}





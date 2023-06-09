//! A library for parsing FIRRTL.

#![allow(unused_parens)]

mod token;
mod lex;
mod parse;

pub mod pass;
pub mod file;
pub mod ast;

pub use lex::FirrtlParseError;
pub use file::FirrtlFile;

#[cfg(test)]
mod tests {
    use crate::file::*;
    use crate::lex::*;
    use crate::parse::*;

    #[test]
    fn circt_parse_basic() -> Result<(), FirrtlParseError> {
        use std::fs::File;
        use std::io::*;

        //let filename = "../parse-basic.fir";
        let filename = "../chisel-tests/firrtl/MyAlu.fir";
        let sf = FirrtlFile::from_file(filename);
        let circuit = sf.parse()?;
        Ok(())
    }
}





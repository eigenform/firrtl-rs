
pub mod file;
pub mod token;
pub mod lex;
pub mod parse;
pub mod ast;

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





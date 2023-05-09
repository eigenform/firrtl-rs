//! Simple passes over the FIRRTL AST

#[cfg(test)]
mod tests {
    use crate::file::*;
    use crate::pass::*;
    use crate::lex::*;

    #[test]
    fn pass() -> Result<(), FirrtlParseError> {
        use std::fs::File;
        use std::io::*;
        let filename = "../chisel-tests/firrtl/MyAlu.fir";
        let sf = FirrtlFile::from_file(filename);
        let circuit = sf.parse()?;
        Ok(())
    }
}



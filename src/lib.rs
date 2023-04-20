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

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::*;
    use miette::{ SourceSpan, SourceOffset, NamedSource, Result };

    #[test]
    fn firrtl_parse_test() {

        let src = read_file("parse-basic.fir");
        let lexer = FirrtlLexer::new("parse-basic.fir", &src);

        for t in lexer {
            match t {
                Ok(ft) => {
                    println!("{:?}", ft);
                },
                Err(e) => {
                    panic!("{:?}", e);
                },
            }
        }


        //let s = read_file("parse-basic.fir".to_string());
        //let input = s.clone();
        //let mut lexer = Token::lexer(&input);
        //loop { 
        //    let span = lexer.span();
        //    match lexer.next() {
        //        Some(t) => match t {
        //            Ok(t) => { 
        //                println!("{:?} {:?}", t, span);
        //            },
        //            Err(e) => {
        //                Err(LexerError {
        //                    src: NamedSource::new("parse-basic.fir", s.clone()),
        //                    span: (span.start, span.len()).into(),
        //                })?;
        //            },
        //        },
        //        None => break,
        //    }
        //}
        //Ok(())

    }

}




#![allow(unused_braces)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(unused_imports)]


#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub firrtl);

pub mod ast;
pub mod lexer;

/// Read a file into a [String].
fn read_file(filename: String) -> String {
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

    //#[test]
    //fn test_lalrpop() {
    //    let s = read_file("chisel-tests/firrtl/MyNestedModule.fir".to_string());
    //    let lexer = Lexer::new(&s);
    //    let res = firrtl::CircuitParser::new().parse(lexer).unwrap();
    //    println!("{:?}", res);
    //}

    #[test]
    fn test_lalrpop_2() {
        let s = read_file("chisel-tests/firrtl/GCD.fir".to_string());
        let lexer = Lexer::new(&s);
        let res = firrtl::CircuitParser::new().parse(lexer).unwrap();
        println!("{:?}", res);
    }

}




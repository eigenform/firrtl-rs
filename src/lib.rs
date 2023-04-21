
use std::sync::*;
use std::rc::*;
use regex::*;


pub struct FirrtlLine<'a> {
    /// The line number in the original source file
    line_number: usize,
    /// Index of the first relevant character in the original source file
    line_start: usize,
    /// Contents
    line: &'a str,
}
impl <'a> FirrtlLine<'a> {
    pub fn indent_level(&self) -> usize {
        self.line_start - 1
    }
    pub fn line_number(&self) -> usize {
        self.line_number
    }
    pub fn line_start(&self) -> usize {
        self.line_start
    }
    pub fn contents(&self) -> &'a str {
        self.line
    }
}

pub struct FirrtlFile<'a> {
    /// Set of lines from some FIRRTL source file
    pub lines: Vec<FirrtlLine<'a>>,
}
impl <'a> FirrtlFile<'a> {
    fn char_is_indent_whitespace(c: &char) -> bool {
        c == &' ' || c == &'\t'
    }

    fn char_is_whitespace(c: &char) -> bool {
        c == &' ' || c == &'\t' || c == &','
    }

    fn read_lines(content: &'a str) -> Vec<FirrtlLine<'a>> {
        let mut res = Vec::new();

        // NOTE: These line numbers start at 0, not 1!
        let lines = content.lines().enumerate();
        for (original_line_num, line) in lines {
            // The indentation level of this line
            let indent_level = line.chars()
                .take_while(|c| Self::char_is_indent_whitespace(c))
                .count();

            // Actual line contents start *after* any indentation
            let post_indent_line = &line[indent_level..]; 

            // Line contents occur *before* any comment
            let line_content = if let Some(i) = post_indent_line.find(';') {
                &post_indent_line[..i]
            } else {
                post_indent_line
            };

            // Ignore any empty lines
            if line_content.is_empty() { 
                continue; 
            }

            res.push(FirrtlLine {
                line_number: original_line_num + 1,
                line_start:  indent_level + 1,
                line: line_content,
            });
            //println!("{:02} {} {:idt$}{:?}", 
            //    indent_level, 
            //    original_line_num, 
            //    "",
            //    line_content, 
            //    idt=indent_level
            //);
        }
        res
    }

    pub fn new(filename: &str, contents: &'a str) -> Self {
        let lines = Self::read_lines(contents);
        Self { 
            lines
        }
    }
}

use logos::Logos;
#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t,]+")]
pub enum Token<'a> {
    #[regex("[a-zA-Z_][a-zA-Z0-9_$-]*", |lex| lex.slice())]
    IdentKw(&'a str),

    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#, |lex| lex.slice())]
    StringLiteral(&'a str),

    #[regex(r#"'([^'\\]|\\t|\\u|\\n|\\|\\')*'"#, |lex| lex.slice())]
    RawString(&'a str),

    #[regex("[0-9]+", |lex| lex.slice())]
    LiteralInt(&'a str),

    #[regex("[+-][0-9]+", |lex| lex.slice())]
    LiteralSignedInt(&'a str),

    #[token(".")]  Period,
    #[token(":")]  Colon,
    #[token("?")]  Question,
    #[token("(")]  LParen,
    #[token(")")]  RParen,
    #[token("{")]  LBrace,
    #[token("}")]  RBrace,
    #[token("[")]  LSquare,
    #[token("]")]  RSquare,
    #[token("<")]  Less,
    #[token("<-")] LessMinus,
    #[token("<=")] LessEqual,
    #[token(">")]  Greater,
    #[token("=")]  Equal,
    #[token("=>")] EqualGreater,


}

pub struct FirrtlParser<'a> {
    /// Associated source file
    sf: FirrtlFile<'a>,
    /// Index into the [FirrtlFile]
    sf_cursor: usize,

}
impl <'a> FirrtlParser<'a> {
    pub fn new(sf: FirrtlFile<'a>) -> Self { 
        Self {
            sf,
            sf_cursor: 0,
        }
    }

    // It's a lot easier to deal with things line-by-line, since FIRRTL
    // depends on indentation (and doesn't care about whitespace otherwise)
    pub fn lex(&mut self) {
        for sfl in &self.sf.lines {
            // File info optionally comes at the end of a line. 
            // Separate actual line content from FIRRTL file info.
            let (content, fileinfo) = if let Some(idx) = sfl.line.find('@') {
                (&sfl.line[..idx], Some(&sfl.line[idx..]))
            } else {
                (sfl.line, None)
            };

            // Extract a set of tokens from each line
            println!("Lexing line {}, {}", sfl.line_number(), content);
            let mut x = Token::lexer(&content);
            let mut tokens = Vec::new();
            while let Some(t) = x.next() {
                let line_span = x.span();
                match t {
                    Ok(t) => {
                        println!("{:?}", t);
                        tokens.push(t);
                    },
                    Err(e) => {
                        let start = sfl.line_start() + line_span.start;
                        let end   = sfl.line_start() + line_span.end;
                        let ln    = sfl.line_number();
                        panic!("unknown token at {}:{}..{}", ln, start,end);
                    },
                }
            }
            println!("{:?}", tokens);
            println!();

        }
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        use std::fs::File;
        use std::io::*;
        let filename = "./parse-basic.fir";
        let mut f = File::open(filename).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        let sf    = FirrtlFile::new(filename, &s);
        let mut p = FirrtlParser::new(sf);
        p.lex();
    }
}





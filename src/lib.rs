
use std::sync::*;
use std::rc::*;
use regex::*;

/// A suitably-trimmed "line" (without comments) in some [FirrtlFile].
#[derive(Debug)]
pub struct FirrtlLine<'a> {
    /// The line number in the original source file
    line_number: usize,
    /// Index of the first relevant character in the original source file
    line_start: usize,
    /// Contents (hopefully meaningful tokens)
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

/// Representing input from a FIRRTL source file. 
pub struct FirrtlFile<'a> {
    /// Set of effective lines
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
impl <'a> Token<'a> {
    pub fn punctuation_from_str(s: &str) -> Self { 
        match s {
            "."  => Self::Period,
            ":"  => Self::Colon,
            "?"  => Self::Question,
            "("  => Self::LParen,
            ")"  => Self::RParen,
            "{"  => Self::LBrace,
            "}"  => Self::RBrace,
            "["  => Self::LSquare,
            "]"  => Self::RSquare,
            "<"  => Self::Less,
            "<-" => Self::LessMinus,
            "<=" => Self::LessEqual,
            ">"  => Self::Greater,
            "="  => Self::Equal,
            "=>" => Self::EqualGreater,
            _ => panic!("Cannot convert '{}' into Token?", s),
        }
    }
}

/// A "tokenized" line in some [FirrtlFile], with optional FIRRTL file info.
#[derive(Debug)]
pub struct FirrtlTokenizedLine<'a> {
    tokens: Vec<Token<'a>>,
    info: Option<&'a str>,
}

/// State used for lexing/parsing a [FirrtlFile].
pub struct FirrtlParser<'a> {
    /// Associated source file
    sf: FirrtlFile<'a>,

    /// Tokenized lines.
    ///
    /// These should be in one-to-one correspondence with [FirrtlLine]
    /// in the associated [FirrtlFile].
    token_lines: Vec<FirrtlTokenizedLine<'a>>,

    /// "Global cursor" (index into 'token_lines')
    gcur: usize,
    /// "Local cursor" (index into 'token_lines[tlcur].tokens')
    lcur:  usize,
}
impl <'a> FirrtlParser<'a> {
    pub fn new(sf: FirrtlFile<'a>) -> Self { 
        Self {
            sf,
            token_lines: Vec::new(),
            gcur: 0,
            lcur: 0,
        }
    }
}

impl <'a> FirrtlParser<'a> {
    // It's a lot easier to deal with things line-by-line, since FIRRTL
    // depends on indentation (and doesn't care about whitespace otherwise). 
    pub fn lex(&mut self) {
        for sfl in &self.sf.lines {
            let mut tokens = Vec::new();

            // File info optionally comes at the end of a line. 
            // Separate actual line content from FIRRTL file info.
            let (content, fileinfo) = if let Some(idx) = sfl.line.find('@') {
                (&sfl.line[..idx], Some(&sfl.line[idx..]))
            } else {
                (sfl.line, None)
            };

            // Extract a set of tokens from each line
            //println!("Lexing line {}, {}", sfl.line_number(), content);
            let mut x = Token::lexer(&content);
            while let Some(t) = x.next() {
                let line_span = x.span();
                let start = sfl.line_start() + line_span.start;
                let end   = sfl.line_start() + line_span.end;
                let ln    = sfl.line_number();
                match t {
                    Ok(t) => tokens.push(t),
                    Err(e) => {
                        println!("{:?}", e);
                        panic!("unknown token at line {}, offset {}..{}", 
                               ln, start,end);
                    },
                }
            }
            //println!("{:?}", tokens);
            self.token_lines.push(FirrtlTokenizedLine {
                tokens,
                info: fileinfo
            });
            //println!();
        }
    }
}

impl <'a> FirrtlParser<'a> {
    /// Return the current line
    fn current_line(&self) -> (&FirrtlTokenizedLine, &FirrtlLine) {
        (&self.token_lines[self.gcur], &self.sf.lines[self.gcur])
    }
    /// Increment the local cursor, "consuming" a token.
    fn consume_token(&mut self) {
        self.lcur += 1;
    }
    /// Increment the global cursor, moving to the next line. 
    fn consume_line(&mut self) {
        self.gcur += 1;
        self.lcur = 0;
    }

    /// Return the current token
    fn token(&mut self) -> &Token<'a> {
        &self.token_lines[self.gcur].tokens[self.lcur]
    }
    fn eat_keyword(&mut self, kw: &str) {
        if *self.token() == Token::IdentKw(kw) {
            self.consume_token();
        } else {
            panic!("expected keyword {}", kw);
        }
    }
    fn eat_punct(&mut self, p: &str) {
        if *self.token() == Token::punctuation_from_str(p) {
            self.consume_token();
        } else {
            panic!("expected punctuation '{}'", p);
        }
    }
    fn parse_ident(&mut self) -> &'a str {
        match *self.token() {
            Token::IdentKw(s) => {
                self.consume_token();
                s
            },
            _ => panic!("expected ident, got {:?}", self.token()),
        }
    }

    pub fn parse(&mut self) {
        // FIRRTL version
        let (line, sfl) = self.current_line();
        println!("Parsing {:?}", sfl);
        println!("{:?}", line.tokens);
        self.eat_keyword("FIRRTL");
        self.eat_keyword("version");
        self.consume_line();

        // Circuit declaration
        let (line, sfl) = self.current_line();
        println!("Parsing {:?}", sfl);
        println!("{:?}", line.tokens);
        self.eat_keyword("circuit");
        let circuit_id = self.parse_ident();
        self.eat_punct(":");
        self.consume_line();

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
        p.parse();
    }
}





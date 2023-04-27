//! Interface for recovering an AST from FIRRTL input. 
//!
//!

use crate::lex;
use crate::token;
use crate::parse;
use crate::ast;

/// Container for input from a FIRRTL (.fir) source file.
pub struct FirrtlFile {
    /// Source filename
    pub filename: String,
    /// Original file contents (for error-handling).
    pub raw_contents: String,
    /// Set of tokenized, "effective" lines
    pub lines: Vec<lex::FirrtlTokenizedLine>,
}
impl FirrtlFile {
    /// Given some string containing the contents of a .fir file, produce a 
    /// list of lines ([FirrtlLine]) that contain meaningful data.
    fn read_lines(content: &str) -> Vec<FirrtlLine> {
        fn char_is_indent_whitespace(c: &char) -> bool {
            c == &' ' || c == &'\t'
        }
        let mut res = Vec::new();

        // NOTE: These line numbers start at 0, not 1!
        let lines = content.lines().enumerate();
        for (original_line_num, line) in lines {
            // The indentation level of this line
            let indent_level = line.chars()
                .take_while(|c| char_is_indent_whitespace(c))
                .count();

            // Actual line contents start *after* any indentation
            let post_indent_line = &line[indent_level..]; 

            // Meaningful line contents occur *before* any comment
            let line_content = if let Some(i) = post_indent_line.find(';') {
                &post_indent_line[..i]
            } else {
                post_indent_line
            };

            // Ignore any lines without meaningful content
            if line_content.is_empty() { 
                continue; 
            }

            res.push(FirrtlLine {
                line_number: original_line_num + 1,
                line_start:  indent_level + 1,
                line: line_content.to_string(),
            });
        }
        res
    }

    /// Tokenize a set of [FirrtlLine] into a list of [FirrtlTokenizedLine].
    fn tokenize_lines(lines: &[FirrtlLine]) 
        -> Vec<lex::FirrtlTokenizedLine> 
    {
        use logos::Logos;
        let mut tokenized_lines = Vec::new();

        for sfl in lines {
            let sf_line       = sfl.line_number();
            let sf_line_start = sfl.line_start();
            let indent_level  = sfl.indent_level();

            // FIRRTL "file info" optionally comes at the end of a line. 
            // Separate meaningful line content from any file info.
            let (content, info) = if let Some(idx) = sfl.contents().find('@') {
                (&sfl.contents()[..idx], Some(sfl.contents()[idx..].to_string()))
            } else {
                (sfl.contents(), None)
            };

            // Extract a set of tokens/spans from each line
            let mut tokens = Vec::new();
            let mut spans  = Vec::new();
            let mut lexer = token::Token::lexer(&content);
            while let Some(t) = lexer.next() {
                let perline_span = lexer.span();
                let start = sf_line_start + perline_span.start;
                let end   = sf_line_start + perline_span.end;
                let token_span = start..end;
                match t {
                    Ok(token) => {
                        tokens.push(token);
                        spans.push(token_span);
                    },
                    // Some error occured while tokenizing this line.
                    // FIXME: Proper error-handling instead of panic!()
                    Err(e) => {
                        println!("{:?}", e);
                        panic!("unknown token at line {}, offset {:?}",
                               sf_line, token_span);
                    },
                }
            }
            let tokenized_line = lex::FirrtlTokenizedLine {
                tokens, spans, sf_line, info, indent_level,
                content: content.to_string(),
            };
            tokenized_lines.push(tokenized_line);
        }
        tokenized_lines
    }
}

/// This is the public interface to a [FirrtlFile]. 
impl FirrtlFile {
    /// Import FIRRTL from some file
    pub fn from_file(filename: &str) -> Self { 
        use std::fs::File;
        use std::io::Read;
        let mut f = File::open(filename).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        Self::from_str(filename, &s)
    }

    /// Import FIRRTL from a string. 
    pub fn from_str(filename: &str, contents: &str) -> Self {
        // Preprocess into a set of [FirrtlLine]
        let raw_lines = Self::read_lines(contents);

        // Produce a set of [FirrtlTokenizedLine]
        let lines = Self::tokenize_lines(&raw_lines);

        Self { 
            raw_contents: contents.to_string(),
            filename: filename.to_string(),
            lines
        }
    }

    /// Convert this [FirrtlFile] into the corresponding [Circuit].
    pub fn parse(&self) -> Result<ast::Circuit, lex::FirrtlParseError> {
        let mut stream = lex::FirrtlStream::new(&self);
        let circuit = parse::FirrtlParser::parse(&mut stream)?;
        Ok(circuit)
    }


}

/// A suitably-trimmed "line" (without comments).
///
/// NOTE: This is not exposed to library users. 
#[derive(Debug)]
struct FirrtlLine {
    /// The line number in the original source file
    line_number: usize,
    /// Index of the first relevant character in the original source file
    line_start: usize,
    /// Contents (hopefully meaningful tokens)
    line: String,
}
impl FirrtlLine {
    pub fn indent_level(&self) -> usize {
        self.line_start - 1
    }
    pub fn line_number(&self) -> usize {
        self.line_number
    }
    pub fn line_start(&self) -> usize {
        self.line_start
    }
    pub fn contents(&self) -> &str {
        &self.line
    }
}



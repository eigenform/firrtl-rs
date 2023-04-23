
/// A suitably-trimmed "line" (without comments) in some [FirrtlFile].
#[derive(Debug)]
pub struct FirrtlLine {
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

/// Representing input from a FIRRTL (.fir) source file.
pub struct FirrtlFile {
    /// Source filename
    pub filename: String,

    /// Set of effective lines. 
    ///
    /// Each element in this list corresponds to some *effective* line
    /// in the source file, along with the location in the original file.
    pub lines: Vec<FirrtlLine>,
}
impl FirrtlFile {
    fn char_is_indent_whitespace(c: &char) -> bool {
        c == &' ' || c == &'\t'
    }

    fn char_is_whitespace(c: &char) -> bool {
        c == &' ' || c == &'\t' || c == &','
    }

    /// Given some string containing the contents of a .fir file, produce a 
    /// list of lines ([FirrtlLine]) that contain meaningful data.
    fn read_lines(content: &str) -> Vec<FirrtlLine> {
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

    pub fn new(filename: &str, contents: &str) -> Self {
        let lines = Self::read_lines(contents);
        Self { 
            filename: filename.to_string(),
            lines
        }
    }
}



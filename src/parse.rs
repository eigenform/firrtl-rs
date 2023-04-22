
use crate::lex::*;

pub enum ParserError {
}

pub struct FirrtlParser<'a> {
    stream: &'a mut FirrtlStream<'a>
}
impl <'a> FirrtlParser<'a> {
    pub fn new(stream: &'a mut FirrtlStream<'a>) -> Self { 
        Self { 
            stream
        }
    }
}

impl <'a> FirrtlParser<'a> {
    pub fn parse(stream: &mut FirrtlStream<'a>) -> Result<(), FirrtlStreamErr> {

        println!("{:?}", stream.line());
        assert!(stream.indent_level() == 0);
        stream.match_identkw("FIRRTL")?;
        stream.next_token();
        stream.match_identkw("version")?;
        stream.next_token();
        stream.next_line();

        println!("{:?}", stream.line());
        assert!(stream.indent_level() == 0);
        stream.match_identkw("circuit")?;
        stream.next_token();
        let circuit_id = stream.get_identkw()?;
        stream.next_token();
        stream.match_punc(":")?;
        stream.next_token();

        // This should be the indentation level for all module declarations
        let module_indent = stream.indent_level();
        assert!(module_indent > 0);

        loop {
            println!("{:?}", stream.line());
            let m_indent = stream.indent_level();
            assert!(m_indent == module_indent);

            if stream.match_identkw("module").is_ok() {
                stream.next_token();
                let module_id = stream.get_identkw()?;
                stream.next_token();
                stream.match_punc(":")?;
                stream.next_token();
                assert!(stream.indent_level() > m_indent);
                let module = FirrtlParser::parse_module(stream)?;
            } 
            else if stream.match_identkw("extmodule").is_ok() {
                stream.next_token();
                unimplemented!("extmodule");
            } 
            else if stream.match_identkw("intmodule").is_ok() {
                stream.next_token();
                unimplemented!("intmodule");
            } else {
                return Err(FirrtlStreamErr::Other("bad module keyword?"));
            }

            break;
        }
        Ok(())
    }

    fn parse_module(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr> 
    {
        let body_indent_level = stream.indent_level();
        println!("{:?}", stream.line());

        if stream.match_identkw("input").is_ok() {
        } 
        else if stream.match_identkw("output").is_ok() {
        } 
        else { 
            return Err(FirrtlStreamErr::Other("expected input or output"));
        }
        stream.next_token();
        let port_id = stream.get_identkw()?;
        stream.next_token();
        stream.match_punc(":")?;
        stream.next_token();

        panic!();
        Ok(())
    }
}



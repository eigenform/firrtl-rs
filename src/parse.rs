
use crate::lex::*;
use crate::ast::*;

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
        assert!(stream.is_sol());

        // This should be the indentation level for all module declarations
        let module_indent = stream.indent_level();
        assert!(module_indent > 0);

        loop {
            assert!(stream.is_sol());
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
        assert!(stream.is_sol());
        let port_list = FirrtlParser::parse_portlist(stream)?;
        let stmt_line = FirrtlParser::parse_statements(stream)?;

        Ok(())
    }
    fn parse_statements(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        panic!("statements");
        Ok(())
    }

    fn parse_portlist(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        let body_indent_level = stream.indent_level();

        loop {
            assert!(stream.is_sol());
            println!("{:?}", stream.line());

            // FIXME: This must be a module without ports?
            if stream.indent_level() < body_indent_level {
                break;
            }


            let has_dir  = (
                stream.token().match_identkw("input").unwrap_or(false) || 
                stream.token().match_identkw("output").unwrap_or(false)
            );
            let has_id   = stream.peekn_token(1).is_identkw();
            let has_col  = stream.peekn_token(2).match_punc(":")
                .unwrap_or(false);

            // There are no more port declarations to handle
            if !(has_dir && has_id && has_col) {
                break;
            }

            let port_dir = if stream.match_identkw("input").is_ok() {
                Direction::Input
            } else if stream.match_identkw("output").is_ok() {
                Direction::Output
            } else { 
                return Err(FirrtlStreamErr::Other("how did we get here?"));
            };
            stream.next_token();

            let port_id = stream.get_identkw()?;
            stream.next_token();
            stream.match_punc(":")?;
            stream.next_token();
            let port_type = FirrtlParser::parse_type(stream)?;
        }


        Ok(())
    }

    fn parse_type(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr>
    {
        // This is a bundle type
        if stream.match_punc("{").is_ok() {
            unimplemented!("bundle");
        } 
        // This is some ground type (or an array of ground type)
        else {
            let ground_type = stream.get_identkw()?;
            let maybe_width = match ground_type {
                "UInt" | "SInt" | "Analog" => true,
                "Clock" | "Reset" | "AsyncReset" => false,
                _ => return Err(FirrtlStreamErr::Other("bad ground type?")),
            };
            stream.next_token();

            // A width is optional for some ground types
            let ls  = stream.token().match_punc("<").is_some();
            let lit = stream.peekn_token(1).is_lit_int();
            let gt  = stream.peekn_token(2).match_punc(">").is_some();
            let width = if maybe_width && ls && lit && gt {
                stream.match_punc("<")?;
                stream.next_token();
                let w = stream.get_lit_int()?;
                stream.next_token();
                stream.match_punc(">")?;
                stream.next_token();
                Some(w.parse::<usize>().unwrap())
            } else { 
                None
            };

            // Optionally indicates an array type
            let ls  = stream.token().match_punc("[").is_some();
            let lit = stream.peekn_token(1).is_lit_int();
            let gt  = stream.peekn_token(2).match_punc("]").is_some();
            let arr_width = if maybe_width && ls && lit && gt {
                stream.match_punc("[")?;
                stream.next_token();
                let w = stream.get_lit_int()?;
                stream.next_token();
                stream.match_punc("]")?;
                stream.next_token();
                Some(w.parse::<usize>().unwrap())
            } else { 
                None
            };
        }

        Ok(())
    }


}




use crate::lex::*;
use crate::ast::*;
use crate::parse::FirrtlParser;
use std::collections::HashMap;

impl <'a> FirrtlParser {
    pub fn parse_module(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr> 
    {
        stream.match_identkw("module")?;
        stream.next_token();
        let module_id = stream.get_identkw()?;
        stream.next_token();
        stream.match_punc(":")?;
        stream.next_token();

        let body_indent_level = stream.indent_level();

        assert!(stream.is_sol());
        let port_list = FirrtlParser::parse_portlist(stream)?;
        assert!(stream.is_sol());

        // There are no statements
        if stream.indent_level() < body_indent_level {
            return Ok(());
        }

        let stmt_list = FirrtlParser::parse_statements_block(stream)?;
        assert!(stream.is_sol());

        Ok(())
    }


    pub fn parse_intmodule(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr> 
    {
        stream.match_identkw("intmodule")?;
        stream.next_token();
        let intmodule_id = stream.get_identkw()?;
        stream.next_token();
        stream.match_punc(":")?;
        stream.next_token();

        let body_indent_level = stream.indent_level();
        assert!(stream.is_sol());
        let port_list = FirrtlParser::parse_portlist(stream)?;
        assert!(stream.is_sol());

        stream.match_identkw("intrinsic")?;
        stream.next_token();
        stream.match_punc("=")?;
        stream.next_token();
        let intrin_id = stream.get_identkw()?;
        stream.next_token();

        loop {
            if stream.indent_level() < body_indent_level {
                break;
            }
            let parameter = FirrtlParser::parse_parameter(stream)?;
        }
        Ok(())

    }

    pub fn parse_extmodule(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr> 
    {
        stream.match_identkw("extmodule")?;
        stream.next_token();
        let extmodule_id = stream.get_identkw()?;
        stream.next_token();
        stream.match_punc(":")?;
        stream.next_token();

        let body_indent_level = stream.indent_level();
        assert!(stream.is_sol());
        let port_list = FirrtlParser::parse_portlist(stream)?;
        assert!(stream.is_sol());

        // FIXME: These have a definite order in the spec
        loop { 
            // End of extmodule
            if stream.indent_level() < body_indent_level {
                break;
            }

            // Start of parameters
            if stream.match_identkw("parameter").is_ok() {
                let parameter = FirrtlParser::parse_parameter(stream)?;
            } 
            else if stream.match_identkw("defname").is_ok() {
                let defname = FirrtlParser::parse_defname(stream)?;
            } 
            // FIXME: Skip 'ref' declarations for now
            else if stream.match_identkw("ref").is_ok() {
                stream.next_line();
            }
        }
        Ok(())
    }

    pub fn parse_defname(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr> 
    {
        stream.match_identkw("defname")?;
        stream.next_token();
        stream.match_punc("=")?;
        stream.next_token();
        let id = stream.get_identkw()?;
        stream.next_token();
        assert!(stream.is_sol());
        Ok(())
    }

    pub fn parse_parameter(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr> 
    {
        stream.match_identkw("parameter")?;
        stream.next_token();
        let param_id = stream.get_identkw()?;
        stream.next_token();
        stream.match_punc("=")?;
        stream.next_token();

        if let Ok(lit) = stream.get_lit_int() {
            stream.next_token();
            assert!(stream.is_sol());
        } 
        else if let Ok(lit) = stream.get_lit_sint() {
            stream.next_token();
            assert!(stream.is_sol());
        }
        else if let Ok(lit) = stream.get_lit_float() {
            stream.next_token();
            assert!(stream.is_sol());
        } 
        else if let Ok(lit) = stream.get_lit_str() {
            stream.next_token();
            assert!(stream.is_sol());
        }
        else if let Ok(lit) = stream.get_lit_raw_str() {
            stream.next_token();
            assert!(stream.is_sol());
        } else {
            unimplemented!("unimpl parameter literal?")
        }
        Ok(())
    }

    pub fn parse_portlist(stream: &mut FirrtlStream<'a>)
        -> Result<Portlist, FirrtlStreamErr>
    {
        let mut res: Portlist = Vec::new();

        let body_indent_level = stream.indent_level();

        loop {
            assert!(stream.is_sol());
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
                return Err(FirrtlStreamErr::Other("invalid port direction"));
            };
            stream.next_token();

            let port_id = stream.get_identkw()?;
            stream.next_token();
            stream.add_module_ctx(port_id);

            stream.match_punc(":")?;
            stream.next_token();
            let port_type = FirrtlParser::parse_type(stream)?;

            res.push(PortDeclaration::new(port_id, port_dir, port_type));

        }

        Ok(res)
    }


}



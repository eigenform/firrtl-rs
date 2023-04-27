
use crate::lex::*;
use crate::ast::*;
use crate::parse::FirrtlParser;
use std::collections::HashMap;

impl <'a> FirrtlParser {
    pub fn parse_module(stream: &mut FirrtlStream<'a>) 
        -> Result<Module, FirrtlStreamErr> 
    {
        stream.match_identkw("module")?;
        stream.next_token();
        let id = stream.get_identkw()?;
        stream.next_token();
        stream.match_punc(":")?;
        stream.next_token();

        let body_indent_level = stream.indent_level();

        assert!(stream.is_sol());
        let ports = FirrtlParser::parse_portlist(stream)?;
        assert!(stream.is_sol());

        // There are no statements
        if stream.indent_level() < body_indent_level {
            return Ok(Module::new(id.to_string(), ports, Vec::new()));
        }

        let statements = FirrtlParser::parse_statements_block(stream)?;
        assert!(stream.is_sol());

        Ok(Module::new(id.to_string(), ports, statements))
    }


    pub fn parse_intmodule(stream: &mut FirrtlStream<'a>) 
        -> Result<IntModule, FirrtlStreamErr> 
    {
        stream.match_identkw("intmodule")?;
        stream.next_token();
        let id = stream.get_identkw()?;
        stream.next_token();
        stream.match_punc(":")?;
        stream.next_token();

        let body_indent_level = stream.indent_level();
        assert!(stream.is_sol());
        let ports = FirrtlParser::parse_portlist(stream)?;
        assert!(stream.is_sol());
        let intmodule = IntModule::new(id, ports);

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
        Ok(intmodule)

    }

    pub fn parse_extmodule(stream: &mut FirrtlStream<'a>) 
        -> Result<ExtModule, FirrtlStreamErr> 
    {
        stream.match_identkw("extmodule")?;
        stream.next_token();
        let id = stream.get_identkw()?;
        stream.next_token();
        stream.match_punc(":")?;
        stream.next_token();

        let body_indent_level = stream.indent_level();
        assert!(stream.is_sol());
        let ports = FirrtlParser::parse_portlist(stream)?;
        assert!(stream.is_sol());
        let extmodule = ExtModule::new(id, ports);

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
        Ok(extmodule)
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

    pub fn parse_port(stream: &mut FirrtlStream<'a>)
        -> Result<PortDecl, FirrtlStreamErr>
    {
        let dir = match stream.get_identkw()? {
            "input"  => Direction::Input,
            "output" => Direction::Output,
            _ => { 
                return Err(FirrtlStreamErr::Other("invalid port direction"));
            },
        };
        stream.next_token();
        let id = stream.get_identkw()?;
        stream.next_token();
        stream.add_module_ctx(id);
        stream.match_punc(":")?;
        stream.next_token();
        let ty = FirrtlParser::parse_type(stream)?;
        Ok(PortDecl::new(id, dir, ty))
    }

    /// Returns 'true' if the current input qualifies as a port declaration.
    pub fn check_port(stream: &mut FirrtlStream<'a>) -> bool {
        let has_dir  = (
            stream.token().match_identkw("input").unwrap_or(false) || 
            stream.token().match_identkw("output").unwrap_or(false)
        );
        let has_id  = stream.peekn_token(1).is_identkw();
        let has_col = stream.peekn_token(2).match_punc(":")
            .unwrap_or(false);
        has_dir && has_id && has_col
    }

    pub fn parse_portlist(stream: &mut FirrtlStream<'a>)
        -> Result<Vec<PortDecl>, FirrtlStreamErr>
    {
        let mut portlist = Vec::new();
        let body_indent_level = stream.indent_level();
        loop {
            assert!(stream.is_sol());
            // There are no more port declarations to handle
            if stream.indent_level() < body_indent_level {
                break;
            }
            // There are no more port declarations to handle
            if !FirrtlParser::check_port(stream) {
                break;
            }
            let port = FirrtlParser::parse_port(stream)?;
            portlist.push(port);
        }
        Ok(portlist)
    }
}



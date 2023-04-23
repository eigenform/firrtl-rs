
use crate::lex::*;
use crate::ast::*;
use crate::token::Token;

pub mod module;
pub mod statement;
pub mod expr;
pub mod datatype;

pub struct FirrtlParser;
impl <'a> FirrtlParser {
    pub fn parse_firrtl_version(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        assert!(stream.indent_level() == 0);
        stream.match_identkw("FIRRTL")?;
        stream.next_token();
        stream.match_identkw("version")?;
        stream.next_token();
        stream.next_line();
        Ok(())
    }

    pub fn parse_circuit(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
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
            let m_indent = stream.indent_level();
            assert!(m_indent == module_indent);

            if stream.match_identkw("module").is_ok() {
                stream.clear_module_ctx();
                stream.next_token();
                let module_id = stream.get_identkw()?;
                stream.next_token();
                stream.match_punc(":")?;
                stream.next_token();
                assert!(stream.indent_level() > m_indent);
                let module = FirrtlParser::parse_module(stream)?;
            } 
            else if stream.match_identkw("extmodule").is_ok() {
                stream.clear_module_ctx();
                stream.next_token();
                let extmodule_id = stream.get_identkw()?;
                stream.next_token();
                stream.match_punc(":")?;
                stream.next_token();
                assert!(stream.indent_level() > m_indent);
                let extmodule = FirrtlParser::parse_extmodule(stream)?;
            } 
            else if stream.match_identkw("intmodule").is_ok() {
                stream.clear_module_ctx();
                stream.next_token();
                let intmodule_id = stream.get_identkw()?;
                stream.next_token();
                stream.match_punc(":")?;
                stream.next_token();
                assert!(stream.indent_level() > m_indent);
                let intmodule = FirrtlParser::parse_intmodule(stream)?;
            } else {
                return Err(FirrtlStreamErr::Other("bad module keyword?"));
            }
        }
    }

    /// Convert a [FirrtlStream] into an AST
    pub fn parse(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr> 
    {
        FirrtlParser::parse_firrtl_version(stream)?;
        FirrtlParser::parse_circuit(stream)?;
        Ok(())
    }
}



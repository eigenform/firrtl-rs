/// FIRRTL parser implementation.

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
        -> Result<(), FirrtlParseError>
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
        -> Result<Circuit, FirrtlParseError>
    {
        assert!(stream.indent_level() == 0);
        stream.match_identkw("circuit")?;
        stream.next_token();
        let circuit_id = stream.get_identkw()?;
        stream.next_token();
        stream.match_punc(":")?;
        stream.next_token();
        assert!(stream.is_sol());

        let mut circuit = Circuit::new(circuit_id);

        // This should be the indentation level for all module declarations
        let module_indent = stream.indent_level();
        assert!(module_indent > 0);

        loop {
            // There are no more module declarations left
            if stream.indent_level() == 0 {
                break;
            }

            assert!(stream.is_sol());
            let m_indent = stream.indent_level();
            assert!(m_indent == module_indent);
            stream.clear_module_ctx();

            match stream.get_identkw()? {
                "module" => {
                    let m = FirrtlParser::parse_module(stream)?;
                    circuit.add_module(m);
                },
                "extmodule" => {
                    let m = FirrtlParser::parse_extmodule(stream)?;
                    circuit.add_extmodule(m);
                },
                "intmodule" => {
                    let m = FirrtlParser::parse_intmodule(stream)?;
                    circuit.add_intmodule(m);
                },
                _ => {
                    panic!("bad module keyword?");
                }
            }
        }

        Ok(circuit)
    }

    /// Convert a [FirrtlStream] into an AST
    pub fn parse(stream: &mut FirrtlStream<'a>) 
        -> Result<Circuit, FirrtlParseError> 
    {
        // This is *optional* for now, I guess
        if let Ok(v) = FirrtlParser::parse_firrtl_version(stream) {
        } else { 
        }

        let circuit = FirrtlParser::parse_circuit(stream)?;
        Ok(circuit)
    }
}



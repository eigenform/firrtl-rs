
use crate::lex::*;
use crate::ast::*;
use crate::token;

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

    pub fn parse(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr> 
    {
        FirrtlParser::parse_firrtl_version(stream)?;
        
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
            println!("{:?}", stream.line().content());
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
                let extmodule_id = stream.get_identkw()?;
                stream.next_token();
                stream.match_punc(":")?;
                stream.next_token();
                assert!(stream.indent_level() > m_indent);
                let extmodule = FirrtlParser::parse_extmodule(stream)?;
                //unimplemented!("extmodule");
            } 
            else if stream.match_identkw("intmodule").is_ok() {
                stream.next_token();
                let intmodule_id = stream.get_identkw()?;
                stream.next_token();
                stream.match_punc(":")?;
                stream.next_token();
                assert!(stream.indent_level() > m_indent);
                let intmodule = FirrtlParser::parse_intmodule(stream)?;
                //unimplemented!("intmodule");
            } else {
                return Err(FirrtlStreamErr::Other("bad module keyword?"));
            }
            //break;
        }
        Ok(())
    }

    fn parse_intmodule(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr> 
    {
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

    fn parse_extmodule(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr> 
    {
        let body_indent_level = stream.indent_level();
        assert!(stream.is_sol());
        let port_list = FirrtlParser::parse_portlist(stream)?;
        assert!(stream.is_sol());

        loop { 
            // End of extmodule
            if stream.indent_level() < body_indent_level {
                break;
            }
            // Start of parameters
            if stream.match_identkw("parameter").is_ok() {
                break;
            }
            let defname = FirrtlParser::parse_defname(stream)?;
        }

        loop { 
            // End of extmodule
            if stream.indent_level() < body_indent_level {
                break;
            }
            let parameter = FirrtlParser::parse_parameter(stream)?;
        }
        Ok(())
    }

    fn parse_defname(stream: &mut FirrtlStream<'a>) 
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

    fn parse_parameter(stream: &mut FirrtlStream<'a>) 
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
        else if let Ok(lit) = stream.get_lit_float() {
            stream.next_token();
            assert!(stream.is_sol());
        } 
        else if let Ok(lit) = stream.get_lit_str() {
            stream.next_token();
            assert!(stream.is_sol());
        } else {
            unimplemented!("unimpl parameter literal?")
        }
        Ok(())
    }


    fn parse_module(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr> 
    {
        let body_indent_level = stream.indent_level();

        assert!(stream.is_sol());
        let port_list = FirrtlParser::parse_portlist(stream)?;
        assert!(stream.is_sol());

        // There are no statements
        if stream.indent_level() < body_indent_level {
            return Ok(());
        }

        let stmt_line = FirrtlParser::parse_statements(stream)?;
        assert!(stream.is_sol());

        Ok(())
    }

    fn parse_statements(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        let body_indent_level = stream.indent_level();
        loop {
            assert!(stream.is_sol());
            println!("[*] Parsing statement: {:?}", stream.line().content());

            if stream.indent_level() < body_indent_level {
                break;
            }

            // FIXME: Not clear if this is sufficient to disambiguate 
            // statements that start with a reference?
            let is_reference_stmt = (
                stream.remaining_tokens().contains(&token::Token::LessEqual) ||
                stream.remaining_tokens().contains(&token::Token::LessMinus) ||
                stream.remaining_tokens().windows(2).position(|w| { w == 
                    &[
                        token::Token::IdentKw("is".to_string()), 
                        token::Token::IdentKw("invalid".to_string())
                    ]
                }).is_some()
            );

            if is_reference_stmt {
                println!("[*] Parsing reference statement: {:?}", stream.line().content());
                let reference = FirrtlParser::parse_reference(stream)?;
                // Must be an assignment '<=', this is an identifier
                if stream.match_punc("<=").is_ok() {
                    stream.next_token();
                    let expr = FirrtlParser::parse_expr(stream)?;
                } 
                // Must be a partial assignment '<-'?,
                else if stream.match_punc("<-").is_ok() {
                    stream.next_token();
                    let expr = FirrtlParser::parse_expr(stream)?;
                }
                // Must be 'is invalid', this is an identifier
                else if stream.match_identkw("is").is_ok() {
                    stream.next_token();
                    stream.match_identkw("invalid")?;
                    stream.next_token();
                } else { 
                    panic!("unexpected keyword in reference statement?");
                }
                assert!(stream.is_sol(), "{:?}", stream.remaining_tokens());
            } 
            // Otherwise, this is a "simple" statement where we can just 
            // match on some keyword
            else {
                match stream.get_identkw()? {
                    "wire" => {
                        stream.next_token();
                        let id = stream.get_identkw()?;
                        stream.next_token();
                        stream.match_punc(":")?;
                        stream.next_token();
                        let ty = FirrtlParser::parse_type(stream)?;
                    },
                    "reg"  => { unimplemented!("reg"); },
                    "inst" => { unimplemented!("inst"); },
                    "node" => { 
                        stream.next_token();
                        let id = stream.get_identkw()?;
                        stream.next_token();
                        stream.match_punc("=")?;
                        stream.next_token();
                        let expr = FirrtlParser::parse_expr(stream)?;
                    },
                    "attach" => { unimplemented!("attach"); },
                    "when" => { unimplemented!("when"); },
                    "stop" => { unimplemented!("stop"); },
                    "printf" => { unimplemented!("printf"); },
                    "skip" => { 
                        stream.next_token();
                    },
                    "define" => { unimplemented!("define"); },
                    "force_release" => { unimplemented!("force_release"); },
                    "connect" => { unimplemented!("connect"); },
                    "invalidate" => { unimplemented!("invalidate"); },
                    identkw @ _ => {
                        panic!("unexpected statement keyword {}", identkw);
                    },
                }
                assert!(stream.is_sol());
            }

        }
        //panic!("statements end?");
        Ok(())
    }

    fn parse_expr(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        // NOTE: Careful with lookahead behavior in this function ..

        println!("parse_expr @ {:?}", stream.remaining_tokens());

        // This must be a static_reference (a single identifier)
        if stream.remaining_tokens().len() == 1 {
            let ident = stream.get_identkw()?;
            stream.next_token();
            return Ok(());
        }

        // FIXME: Does this properly disambiguate between "an operation" 
        // and "a reference?"
        // Is this expression a primitive operation?
        let is_primop_expr = {
            if stream.remaining_tokens().len() >= 2 {
                let kw = stream.get_identkw()?;
                let kw_ok = (
                    PrimOp2Expr::from_str(kw).is_some() ||
                    PrimOp1Expr::from_str(kw).is_some() ||
                    PrimOp1Expr1Int::from_str(kw).is_some() ||
                    PrimOp1Expr2Int::from_str(kw).is_some()
                );
                let has_lparen = stream.peekn_token(1).match_punc("(")
                    .unwrap_or(false);
                kw_ok && has_lparen
            } else {
                false
            }
        };
        // Is this expression a mux() operation?
        let is_mux_expr = {
            if stream.remaining_tokens().len() >= 2 {
                stream.match_identkw("mux").is_ok() &&
                    stream.peekn_token(1).match_punc("(").unwrap_or(false)
            } else {
                false
            }
        };
        // Is this expression a read() operation?
        let is_read_expr = {
            if stream.remaining_tokens().len() >= 2 {
                stream.match_identkw("read").is_ok() &&
                    stream.peekn_token(1).match_punc("(").unwrap_or(false)
            } else {
                false
            }
        };
        // Is this expression a literal SInt or UInt?
        let is_lit_expr = {
            if stream.remaining_tokens().len() >= 2 {
                let maybe_keyword = stream.get_identkw()?;
                let kw_ok = (
                    (maybe_keyword == "UInt" || maybe_keyword == "SInt")
                );
                let has_width = (
                    if stream.peekn_token(1).match_punc("<").unwrap_or(false) {
                        if stream.peekn_token(2).is_lit_int() {
                            if stream.peekn_token(3).match_punc(">")
                                .unwrap_or(false) { true }
                            else { false }
                        } else { false }
                    } else { false }
                );
                let has_lparen = if has_width {
                    stream.peekn_token(4).match_punc("(").unwrap_or(false)
                } else { 
                    stream.peekn_token(1).match_punc("(").unwrap_or(false)
                };
                kw_ok && has_lparen
            } else {
                false
            }
        };

        // This must be a reference expression
        if !(is_lit_expr || is_read_expr || is_mux_expr || is_primop_expr) {
            println!("parse ref expr {:?}", stream.remaining_tokens());
            let reference = FirrtlParser::parse_reference(stream)?;
            return Ok(());
        } 

        if is_lit_expr {
            let sint_or_uint = stream.get_identkw()?;
            stream.next_token();
            let width = FirrtlParser::parse_optional_typewidth(stream)?;
            stream.match_punc("(")?;
            unimplemented!("expr literal");
        }
        else if is_read_expr {
            unimplemented!("expr read");
        } 
        else if is_mux_expr {
            unimplemented!("expr mux");
        }
        else if is_primop_expr {
            let primop_kw = stream.get_identkw()?;
            if PrimOp2Expr::from_str(primop_kw).is_some() {
                stream.next_token();
                stream.match_punc("(")?;
                stream.next_token();
                let e1 = FirrtlParser::parse_expr(stream)?;
                let e2 = FirrtlParser::parse_expr(stream)?;
                stream.match_punc(")")?;
                stream.next_token();
            } 
            else if PrimOp1Expr::from_str(primop_kw).is_some() {
                stream.next_token();
                stream.match_punc("(")?;
                stream.next_token();
                let e1 = FirrtlParser::parse_expr(stream)?;
                stream.match_punc(")")?;
                stream.next_token();
            } 
            else if PrimOp1Expr1Int::from_str(primop_kw).is_some() {
                stream.next_token();
                stream.match_punc("(")?;
                stream.next_token();
                unimplemented!("PrimOp1Expr1Int");
            } 
            else if PrimOp1Expr2Int::from_str(primop_kw).is_some() {
                stream.next_token();
                stream.match_punc("(")?;
                stream.next_token();
                unimplemented!("PrimOp1Expr2Int");
            } else {
                panic!("eh?");
            }
        }
        else {
            panic!("how did we get here?")
        }

        Ok(())
        //unimplemented!("expr");
    }

    fn parse_reference(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        let static_ref = FirrtlParser::parse_static_reference(stream)?;
        if stream.match_punc("[").is_ok() {
            stream.next_token();
            let expr = FirrtlParser::parse_expr(stream)?;
            stream.match_punc("]")?;
            stream.next_token();
            unimplemented!("dynamic ref?");
        }
        Ok(())

    }


    fn parse_static_reference(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {

        // Static references *must* begin with an identifier
        let ref_ident = stream.get_identkw()?;
        stream.next_token();

        // ... followed by some arbitrary list of subfield/subindex
        loop {
            println!("Parsing static reference {:?}", stream.remaining_tokens());

            // Must be a subfield access
            if stream.match_punc(".").is_ok() {
                stream.next_token();
                // FIXME: SFC behavior allows unsigned integer subfield names
                if let Ok(lit) = stream.get_lit_int() {
                    stream.next_token();
                } 
                else if let Ok(ident) = stream.get_identkw() {
                    stream.next_token();
                } 
            } 
            // Must be a subindex access
            else if stream.match_punc("[").is_ok() {
                stream.next_token();
                let subindex = stream.get_lit_int()?;
                stream.next_token();
                stream.match_punc("]")?;
                stream.next_token();
            } else {
                break;
            }
        }
        Ok(())
    }



    fn parse_portlist(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        let body_indent_level = stream.indent_level();

        loop {
            assert!(stream.is_sol());
            println!("{:?}", stream.line().content());

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

    fn parse_optional_typewidth(stream: &mut FirrtlStream<'a>)
        -> Result<Option<usize>, FirrtlStreamErr>
    {
        let width = if stream.match_punc("<").is_ok() {
            stream.next_token();
            let w = stream.get_lit_int()?;
            stream.next_token();
            stream.match_punc(">")?;
            stream.next_token();
            Some(w.parse::<usize>().unwrap())
        } else { 
            None
        };
        Ok(width)
    }

    fn parse_bundle_field(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr>
    {
        println!("{:?}", stream.remaining_tokens());
        let flip = if stream.match_identkw("flip").is_ok() {
            stream.next_token();
            true
        } else {
            false
        };

        // FIXME: SFC defines 'fieldId' which *includes* unsigned integers.
        // I'm only doing this to pass thru 'parse-basic.fir' from llvm/CIRCT. 
        let field_id = if let Ok(lit) = stream.get_lit_int() {
            lit
        } else {
            stream.get_identkw()?
        };
        stream.next_token();

        stream.match_punc(":")?;
        stream.next_token();

        println!("{:?}", stream.remaining_tokens());
        let field_type = FirrtlParser::parse_type(stream)?;
        println!("{:?}", stream.remaining_tokens());
        Ok(())
    }

    fn parse_bundle(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr>
    {
        stream.match_punc("{")?;
        stream.next_token();

        // NOTE: Always at least one field?
        let field = FirrtlParser::parse_bundle_field(stream)?;
        loop { 
            println!("{:?}", stream.remaining_tokens());
            // End of fields
            if stream.match_punc("}").is_ok() {
                stream.next_token();
                break;
            }

            println!("{:?}", stream.remaining_tokens());
            let f = FirrtlParser::parse_bundle_field(stream)?;
            println!("{:?}", stream.remaining_tokens());

        }

        Ok(())
    }




    fn parse_type(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr>
    {
        // This is a bundle type
        if stream.match_punc("{").is_ok() {
            let bundle_type = FirrtlParser::parse_bundle(stream)?;
            //unimplemented!("bundle");
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

            let width = if maybe_width {
                FirrtlParser::parse_optional_typewidth(stream)?
            } else {
                None
            };

            // Optionally indicates an array type
            let arrwidth = if stream.match_punc("[").is_ok() {
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



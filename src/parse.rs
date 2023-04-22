
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

        println!("{:?}", stream.line().content());
        assert!(stream.indent_level() == 0);
        stream.match_identkw("FIRRTL")?;
        stream.next_token();
        stream.match_identkw("version")?;
        stream.next_token();
        stream.next_line();

        println!("{:?}", stream.line().content());
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
                unimplemented!("extmodule");
            } 
            else if stream.match_identkw("intmodule").is_ok() {
                stream.next_token();
                unimplemented!("intmodule");
            } else {
                return Err(FirrtlStreamErr::Other("bad module keyword?"));
            }
            //break;
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
            println!("{:?}", stream.line().content());

            // FIXME: This must be a module without statements?
            if stream.indent_level() < body_indent_level {
                break;
            }

            let st_first = stream.get_identkw()?;
            stream.next_token();
            // reference <= expr
            if stream.match_punc("<=").is_ok() {
                stream.next_token();
                let expr = FirrtlParser::parse_expr(stream)?;
                assert!(stream.is_sol());
            } 
            // reference 'is invalid'
            else if stream.match_identkw("is").is_ok() {
                stream.next_token();
                stream.match_identkw("invalid")?;
                stream.next_token();
                assert!(stream.is_sol());
                continue;
            } 
            // keyword
            else {
                println!("{:?}", stream.remaining_tokens());
                unimplemented!("statement keyword");
            }
        }
        //panic!("statements end?");
        Ok(())
    }

    fn parse_expr(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        println!("{:?}", stream.remaining_tokens());

        let expr_st = stream.get_identkw()?;
            
        if (expr_st == "UInt" || expr_st == "SInt") {
            stream.next_token();
            let width = FirrtlParser::parse_optional_typewidth(stream)?;
            unimplemented!("expr literal");
        }


        // This must mean the expression is some op (or mux/read)
        if stream.match_punc("(").is_ok() {
            stream.next_token(); // expr_st
            stream.next_token(); // "("
            if PrimOp2Expr::from_str(expr_st).is_some() {
                let e1 = FirrtlParser::parse_expr(stream)?;
                let e2 = FirrtlParser::parse_expr(stream)?;
                stream.match_punc(")")?;
                stream.next_token();
            } 
            else if PrimOp1Expr::from_str(expr_st).is_some() {
                let e1 = FirrtlParser::parse_expr(stream)?;
                stream.match_punc(")")?;
                stream.next_token();
            } 
            else if PrimOp1Expr1Int::from_str(expr_st).is_some() {
                unimplemented!("PrimOp1Expr1Int");
            } 
            else if PrimOp1Expr2Int::from_str(expr_st).is_some() {
                unimplemented!("PrimOp1Expr2Int");
            } 
            else if expr_st == "mux" {
                unimplemented!("mux");
            }
            else if expr_st == "read" {
                unimplemented!("read");
            }
            else {
                return Err(FirrtlStreamErr::Other(
                    "Invalid keyword in statement?"
                ));
            }
        } else {
            let ref_stmt = FirrtlParser::parse_reference(stream)?;
            //unimplemented!("ref?");
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
        let ref_ident = stream.get_identkw()?;
        stream.next_token();

        loop {
            println!("{:?}", stream.remaining_tokens());
            // Must be a subfield access
            if stream.match_punc(".").is_ok() {
                stream.next_token();
                let subfield_ident = stream.get_identkw()?;
                stream.next_token();
                continue;
            }
            // Must be a subindex access
            if stream.match_punc("[").is_ok() {
                stream.next_token();
                let subindex = stream.get_lit_int()?;
                stream.next_token();
                stream.match_punc("]")?;
                stream.next_token();
                continue;
            }
            break;
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

    fn parse_optional_typewidth(stream: &mut FirrtlStream<'a>)
        -> Result<Option<usize>, FirrtlStreamErr>
    {
        let ls  = stream.token().match_punc("<").is_some();
        let lit = stream.peekn_token(1).is_lit_int();
        let gt  = stream.peekn_token(2).match_punc(">").is_some();
        let width = if ls && lit && gt {
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
        Ok(width)
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

            let width = if maybe_width {
                FirrtlParser::parse_optional_typewidth(stream)?
            } else {
                None
            };
            // A width is optional for some ground types
            //let ls  = stream.token().match_punc("<").is_some();
            //let lit = stream.peekn_token(1).is_lit_int();
            //let gt  = stream.peekn_token(2).match_punc(">").is_some();
            //let width = if maybe_width && ls && lit && gt {
            //    stream.match_punc("<")?;
            //    stream.next_token();
            //    let w = stream.get_lit_int()?;
            //    stream.next_token();
            //    stream.match_punc(">")?;
            //    stream.next_token();
            //    Some(w.parse::<usize>().unwrap())
            //} else { 
            //    None
            //};

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



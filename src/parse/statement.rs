
use crate::lex::*;
use crate::parse::FirrtlParser;

impl <'a> FirrtlParser {
    pub fn parse_statements_block(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        let body_indent_level = stream.indent_level();
        loop {
            println!("handling statement in block: {:?}", stream.line().content());
            assert!(stream.is_sol());
            if stream.indent_level() < body_indent_level {
                break;
            }
            let statement = FirrtlParser::parse_statement(stream)?;
        }
        Ok(())
    }

    pub fn parse_statement(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        println!("parsing statement @ {:?}", stream.remaining_tokens());
        // We have to check for statements that begin with a 'reference'
        // first. Otherwise, this is a "simple" statement where we can 
        // just match on some keyword
        if FirrtlParser::check_reference(stream) {
            let ref_stmt = FirrtlParser::parse_reference_stmt(stream)?;
            Ok(())
        } else {
            match stream.get_identkw()? {
                "wire"
                    => FirrtlParser::parse_wire_stmt(stream)?,
                "reg"
                    => FirrtlParser::parse_reg_stmt(stream)?,
                "inst"
                    => FirrtlParser::parse_inst_stmt(stream)?,
                "node"
                    => FirrtlParser::parse_node_stmt(stream)?,
                "attach"
                    => FirrtlParser::parse_attach_stmt(stream)?,
                "when"
                    => FirrtlParser::parse_when_stmt(stream)?,
                "stop"
                    => FirrtlParser::parse_stop_stmt(stream)?,
                "printf" 
                    => FirrtlParser::parse_printf_stmt(stream)?,
                "skip"
                    => stream.next_token(),
                "define"
                    => FirrtlParser::parse_define_stmt(stream)?,
                "force_release"
                    => FirrtlParser::parse_force_release_stmt(stream)?,
                "connect"
                    => FirrtlParser::parse_connect_stmt(stream)?,
                "invalidate"
                    => FirrtlParser::parse_invalidate_stmt(stream)?,
                identkw @ _ => {
                    panic!("unexpected statement keyword {}", identkw);
                },
            }
            Ok(())
        }
    }

    pub fn parse_reg_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        unimplemented!("reg");
    }
    pub fn parse_inst_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        unimplemented!("inst");
    }
    pub fn parse_define_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        unimplemented!("define");
    }
    pub fn parse_attach_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        unimplemented!("attach");
    }
    pub fn parse_force_release_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        unimplemented!("force_release");
    }
    pub fn parse_connect_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        unimplemented!("connect");
    }
    pub fn parse_invalidate_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        unimplemented!("invalidate");
    }

    pub fn parse_printf_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        stream.match_identkw("printf")?;
        stream.next_token();
        stream.match_punc("(")?;
        stream.next_token();

        let e1 = FirrtlParser::parse_expr(stream)?;
        let e2 = FirrtlParser::parse_expr(stream)?;
        let fmtstr = stream.get_lit_str()?;
        stream.next_token();

        // Parse expressions until we reach the closing paren
        loop {
            if stream.match_punc(")").is_ok() {
                stream.next_token();
                break;
            }
            let fmt_expr = FirrtlParser::parse_expr(stream)?;
        }

        if stream.match_punc(":").is_ok() {
            stream.next_token();
            let _ = stream.get_identkw()?;
            stream.next_token();
        }

        Ok(())
    }

    pub fn parse_stop_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        stream.match_identkw("stop")?;
        stream.next_token();
        stream.match_punc("(")?;
        stream.next_token();
        let e1 = FirrtlParser::parse_expr(stream)?;
        let e2 = FirrtlParser::parse_expr(stream)?;
        let lit = stream.get_lit_int()?;
        stream.next_token();
        stream.match_punc(")")?;
        stream.next_token();

        if stream.match_punc(":").is_ok() {
            stream.next_token();
            let _ = stream.get_identkw()?;
            stream.next_token();
        }
        Ok(())
    }


    pub fn parse_reference_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        println!("parsing reference stmt @ {:?}", stream.remaining_tokens());
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
        Ok(())
    }

    pub fn parse_wire_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        stream.match_identkw("wire")?;
        stream.next_token();
        let wire_id = stream.get_identkw()?;
        stream.add_module_ctx(wire_id);
        stream.next_token();
        stream.match_punc(":")?;
        stream.next_token();
        let ty = FirrtlParser::parse_type(stream)?;
        Ok(())
    }

    pub fn parse_node_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        stream.match_identkw("node")?;
        stream.next_token();
        let node_id = stream.get_identkw()?;
        stream.add_module_ctx(node_id);
        stream.next_token();
        stream.match_punc("=")?;
        stream.next_token();
        let expr = FirrtlParser::parse_expr(stream)?;
        Ok(())
    }


    pub fn parse_when_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        let current_indent = stream.indent_level();

        stream.match_identkw("when")?;
        stream.next_token();
        let expr_cond = FirrtlParser::parse_expr(stream)?;
        stream.match_punc(":")?;
        stream.next_token();

        // Have we reached the start of a new line?
        if stream.is_sol() {
            // This must be a block of statements
            assert!(stream.indent_level() > current_indent);
            let statements = FirrtlParser::parse_statements_block(stream)?;
            println!("Finished when block");

            assert!(stream.is_sol());
            println!("{:?}", stream.remaining_tokens());
            loop {
                if stream.indent_level() < current_indent {
                    panic!("uhhh");
                }

                println!("{:?}", stream.remaining_tokens());

                if stream.match_identkw("else").is_ok() {
                    stream.next_token();

                    // This terminates the list of conditional blocks
                    if stream.match_punc(":").is_ok() {
                        stream.next_token();
                        let blk = FirrtlParser::parse_statements_block(stream)?;
                        break;
                    }

                    // Add to the list of conditional blocks
                    if stream.match_identkw("when").is_ok() {
                        stream.next_token();
                        let expr_elsewhen = FirrtlParser::parse_expr(stream)?;
                        stream.match_punc(":")?;
                        stream.next_token();
                        let blk = FirrtlParser::parse_statements_block(stream)?;
                        continue;
                    }
                } else { 
                    break;
                }
            }
            Ok(())

            //if stream.match_identkw("else").is_ok() {
            //    stream.next_token();

            //    if stream.match_identkw("when").is_ok() { 
            //        stream.next_token();
            //        let expr_elsewhen = FirrtlParser::parse_expr(stream)?;
            //    }
            //    stream.match_punc(":")?;
            //    stream.next_token();
            //    let statements = FirrtlParser::parse_statements_block(stream)?;
            //    Ok(())
            //} 
            //else {
            //    Ok(())
            //}

        } 
        // If there are still tokens on this line, this must be the case 
        // with a single statement on the same line
        else {
            println!("{:?}", stream.remaining_tokens());
            let stmt = FirrtlParser::parse_statement(stream)?;
            println!("{:?}", stream.remaining_tokens());

            if stream.is_sol() {
                unimplemented!();
                Ok(())
            } else { 
                // This must be 'else' with a single statement
                if stream.match_identkw("else").is_ok() {
                    stream.next_token();
                    stream.match_punc(":")?;
                    stream.next_token();
                    let else_stmt = FirrtlParser::parse_statement(stream)?;
                    Ok(())
                } 
                else {
                    Ok(())
                }
            }
        }
    }
}


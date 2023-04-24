
use crate::lex::*;
use crate::parse::FirrtlParser;

impl <'a> FirrtlParser {
    pub fn parse_statements_block(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        let body_indent_level = stream.indent_level();
        assert!(stream.is_sol());
        loop {
            println!("handling statement in block: {:?}", stream.line().content());
            if stream.indent_level() < body_indent_level {
                break;
            }
            let statement = FirrtlParser::parse_statement(stream)?;
            //println!("finished statement, {:?}", stream.remaining_tokens());

            // FIXME: Dynamic field access after a statement?
            if !stream.is_sol() {
                if stream.match_punc(".").is_ok() {
                    stream.next_token();
                    let dynamic_subfield = stream.get_identkw()?;
                    stream.next_token();
                }
            }

            assert!(stream.is_sol());
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
        } else {
            match stream.get_identkw()? {
                "wire"
                    => FirrtlParser::parse_wire_stmt(stream)?,
                "reg"
                    => FirrtlParser::parse_reg_stmt(stream)?,
                "mem"
                    => FirrtlParser::parse_mem_stmt(stream)?,
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
                "force_initial"
                    => FirrtlParser::parse_force_initial_stmt(stream)?,
                "release_initial"
                    => FirrtlParser::parse_release_initial_stmt(stream)?,
                "force"
                    => FirrtlParser::parse_force_stmt(stream)?,
                "release"
                    => FirrtlParser::parse_release_stmt(stream)?,
                "connect"
                    => FirrtlParser::parse_connect_stmt(stream)?,
                "invalidate"
                    => FirrtlParser::parse_invalidate_stmt(stream)?,

                // FIXME: These are old SFC statements that I don't want
                // to deal with right now
                "cmem" | "smem" => { 
                    stream.next_line(); 
                },
                "infer" | "read" | "write" | "rdwr" => {
                    stream.next_line();
                },

                // FIXME: These are verification statements that aren't
                // properly in the spec yet
                "assert" | "assume" | "cover" => {
                    stream.next_line();
                },

                identkw @ _ => {
                    panic!("unexpected statement keyword {}", identkw);
                },
            }
        }
        Ok(())
    }

    pub fn parse_mem_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        let stmt_blk_level = stream.indent_level();
        stream.match_identkw("mem")?;
        stream.next_token();
        let mem_id = stream.get_identkw()?;
        stream.next_token();
        stream.match_punc(":")?;
        stream.next_token();
        assert!(stream.is_sol() && stream.indent_level() > stmt_blk_level);

        // FIXME: In 'parse-basic.fir' from llvm/circt, there are some examples
        // where the order of declarations here does not match the following
        // language in the spec:
        //
        //      data-type =>
        //      depth =>
        //      read-latency =>
        //      write-latency =>
        //      read-under-write =>
        //      { reader ... }
        //      { writer ... }
        //      { readwriter ... }
        //
        //
        // Additionally, there are also examples where 'reader' and 'writer'
        // may have more than one identifier in the same line; in the spec,
        // 'reader' and 'writer' are defined as having a single id per line.
        //
        // Are those test cases valid FIRRTL (ie. old SFC behavior?)
        //
        // For now, just do this the messy way. 
        // The following code is *incorrect*. 

        loop {
            if stream.indent_level() <= stmt_blk_level {
                break;
            }

            if stream.match_identkw("data-type").is_ok() {
                stream.next_token();
                stream.match_punc("=>")?;
                stream.next_token();
                let data_type = FirrtlParser::parse_type(stream)?;
            }

            if stream.match_identkw("depth").is_ok() {
                stream.next_token();
                stream.match_punc("=>")?;
                stream.next_token();
                let depth = stream.get_lit_int()?;
                stream.next_token();
            }

            if stream.match_identkw("read-latency").is_ok() {
                stream.next_token();
                stream.match_punc("=>")?;
                stream.next_token();
                let read_lat = stream.get_lit_int()?;
                stream.next_token();
            }

            if stream.match_identkw("write-latency").is_ok() {
                stream.next_token();
                stream.match_punc("=>")?;
                stream.next_token();
                let write_lat = stream.get_lit_int()?;
                stream.next_token();
            }

            if stream.match_identkw("read-under-write").is_ok() {
                stream.next_token();
                stream.match_punc("=>")?;
                stream.next_token();
                match stream.get_identkw()? {
                    "old" => {},
                    "new" => {},
                    "undefined" => {},
                    _ => panic!("unexpected read-under-write keyword"),
                }
                stream.next_token();
            }
            if stream.match_identkw("reader").is_ok() {
                stream.next_token();
                stream.match_punc("=>")?;
                stream.next_token();
                while !stream.is_sol() {
                    let rp_id = stream.get_identkw()?;
                    stream.next_token();
                }
            }
            if stream.match_identkw("writer").is_ok() {
                stream.next_token();
                stream.match_punc("=>")?;
                stream.next_token();
                while !stream.is_sol() {
                    let wp_id = stream.get_identkw()?;
                    stream.next_token();
                }
            }
            if stream.match_identkw("readwriter").is_ok() {
                stream.next_token();
                stream.match_punc("=>")?;
                stream.next_token();
                while !stream.is_sol() {
                    let rwp_id = stream.get_identkw()?;
                    stream.next_token();
                }
            }
        }

        Ok(())
    }

    pub fn parse_reg_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        stream.match_identkw("reg")?;
        stream.next_token();
        let id = stream.get_identkw()?;
        stream.add_module_ctx(id);
        stream.next_token();
        stream.match_punc(":")?;
        stream.next_token();

        let reg_type = FirrtlParser::parse_type(stream)?;
        let clk_expr = FirrtlParser::parse_expr(stream)?;

        if stream.match_identkw("with").is_ok() {
            stream.next_token();
            stream.match_punc(":")?;
            stream.next_token();

            // Apparently optional parenthesis?
            if stream.match_punc("(").is_ok() {
                stream.next_token();
            }

            stream.match_identkw("reset")?;
            stream.next_token();
            stream.match_punc("=>")?;
            stream.next_token();
            stream.match_punc("(")?;
            stream.next_token();
            let e1 = FirrtlParser::parse_expr(stream)?;
            let e2 = FirrtlParser::parse_expr(stream)?;
            stream.match_punc(")")?;
            stream.next_token();

            // Apparently optional parenthesis?
            if stream.match_punc(")").is_ok() {
                stream.next_token();
            }
        }
        Ok(())
    }
    pub fn parse_inst_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        stream.match_identkw("inst")?;
        stream.next_token();

        let inst_id = stream.get_identkw()?;
        stream.add_module_ctx(inst_id);
        stream.next_token();

        stream.match_identkw("of")?;
        stream.next_token();

        // FIXME: legalize module identifiers
        let module_id = stream.get_identkw()?;
        stream.next_token();

        Ok(())
    }

    pub fn parse_define_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        stream.match_identkw("define")?;
        stream.next_token();
        let static_ref = FirrtlParser::parse_static_reference(stream)?;
        stream.match_punc("=")?;
        stream.next_token();
        let ref_expr = FirrtlParser::parse_ref_expr(stream)?;
        Ok(())
    }

    pub fn parse_attach_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        stream.match_identkw("attach")?;
        stream.next_token();
        stream.match_punc("(")?;
        stream.next_token();
        while !stream.match_punc(")").is_ok() {
            let reference = FirrtlParser::parse_reference(stream)?;
        }
        stream.match_punc(")")?;
        stream.next_token();
        Ok(())
    }

    pub fn parse_force_initial_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        stream.match_identkw("force_initial")?;
        stream.next_token();
        stream.match_punc("(")?;
        stream.next_token();

        let ref_expr = FirrtlParser::parse_ref_expr(stream)?;
        let expr = FirrtlParser::parse_expr(stream)?;
        stream.match_punc(")")?;
        stream.next_token();
        Ok(())
    }

    pub fn parse_release_initial_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        stream.match_identkw("release_initial")?;
        stream.next_token();
        stream.match_punc("(")?;
        stream.next_token();
        let ref_expr = FirrtlParser::parse_ref_expr(stream)?;
        stream.match_punc(")")?;
        stream.next_token();
        Ok(())
    }

    pub fn parse_force_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        stream.match_identkw("force")?;
        stream.next_token();
        stream.match_punc("(")?;
        stream.next_token();

        let e1 = FirrtlParser::parse_expr(stream)?;
        let e2 = FirrtlParser::parse_expr(stream)?;
        let ref_expr = FirrtlParser::parse_ref_expr(stream)?;
        let e3 = FirrtlParser::parse_expr(stream)?;
        stream.match_punc(")")?;
        stream.next_token();
        Ok(())
    }

    pub fn parse_release_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        stream.match_identkw("release")?;
        stream.next_token();
        stream.match_punc("(")?;
        stream.next_token();

        let e1 = FirrtlParser::parse_expr(stream)?;
        let e2 = FirrtlParser::parse_expr(stream)?;
        let ref_expr = FirrtlParser::parse_ref_expr(stream)?;
        stream.match_punc(")")?;
        stream.next_token();
        Ok(())
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

        // Consume 'when <expr> :'
        stream.match_identkw("when")?;
        stream.next_token();
        let expr_cond = FirrtlParser::parse_expr(stream)?;
        stream.match_punc(":")?;
        stream.next_token();

        // This must be 'when <expr> : <statement>'
        if !stream.is_sol() {
            let stmt = FirrtlParser::parse_statement(stream)?;
            // This must be 'when <expr> : <statement> else : <statement>'
            if stream.match_identkw("else").is_ok() {
                stream.next_token();
                stream.match_punc(":")?;
                stream.next_token();
                let else_stmt = FirrtlParser::parse_statement(stream)?;
            }
            assert!(stream.is_sol());
            return Ok(())
        }

        // Otherwise, this must be 
        // 'when <expr> : { statements }'
        assert!(stream.indent_level() > current_indent);
        let statements = FirrtlParser::parse_statements_block(stream)?;

        loop {
            if stream.indent_level() < current_indent {
                break;
            }

            // This must be a following 'else' or 'else when' block
            if stream.match_identkw("else").is_ok() {
                stream.next_token();

                // Must be 'else when <expr> : { statements }'
                if stream.match_identkw("when").is_ok() {
                    stream.next_token();
                    let expr_elsewhen = FirrtlParser::parse_expr(stream)?;
                }
                stream.match_punc(":")?;
                stream.next_token();
                let statements = FirrtlParser::parse_statements_block(stream)?;
            } 
            // Otherwise, we're done parsing this set of conditionals
            else {
                break;
            }
        }

        Ok(())
    }
}



//        // This must be 'when <expr> : <statement> else :'
//        if !stream.is_sol() {
//        }
//
//
//        // Have we reached the start of a new line?
//        if stream.is_sol() {
//            // This must be a block of statements
//            assert!(stream.indent_level() > current_indent);
//            let statements = FirrtlParser::parse_statements_block(stream)?;
//            println!("Finished when block");
//
//            assert!(stream.is_sol());
//            println!("{:?}", stream.remaining_tokens());
//            loop {
//
//                println!("{:?}", stream.remaining_tokens());
//
//                if stream.indent_level() < current_indent {
//                    panic!("uhhh");
//                }
//
//
//                if stream.match_identkw("else").is_ok() {
//                    stream.next_token();
//
//                    // This terminates the list of conditional blocks
//                    if stream.match_punc(":").is_ok() {
//                        stream.next_token();
//                        let blk = FirrtlParser::parse_statements_block(stream)?;
//                        break;
//                    }
//
//                    // Add to the list of conditional blocks
//                    if stream.match_identkw("when").is_ok() {
//                        stream.next_token();
//                        let expr_elsewhen = FirrtlParser::parse_expr(stream)?;
//                        stream.match_punc(":")?;
//                        stream.next_token();
//                        let blk = FirrtlParser::parse_statements_block(stream)?;
//                        continue;
//                    }
//                } else { 
//                    break;
//                }
//            }
//            Ok(())
//        } 
//        // If there are still tokens on this line, this must be the case 
//        // with a single statement on the same line
//        else {
//            println!("{:?}", stream.remaining_tokens());
//            let stmt = FirrtlParser::parse_statement(stream)?;
//            println!("{:?}", stream.remaining_tokens());
//
//            if stream.is_sol() {
//                unimplemented!();
//                Ok(())
//            } else { 
//                // This must be 'else' with a single statement
//                if stream.match_identkw("else").is_ok() {
//                    stream.next_token();
//                    stream.match_punc(":")?;
//                    stream.next_token();
//                    let else_stmt = FirrtlParser::parse_statement(stream)?;
//                    Ok(())
//                } 
//                else {
//                    Ok(())
//                }
//            }
//        }
//    }
//}


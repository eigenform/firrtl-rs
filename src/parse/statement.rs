
use crate::ast::*;
use crate::lex::*;
use crate::parse::FirrtlParser;

impl <'a> FirrtlParser {
    pub fn parse_statements_block(stream: &mut FirrtlStream<'a>)
        -> Result<Vec<Statement>, FirrtlStreamErr>
    {
        let mut statements = Vec::new();
        let body_indent_level = stream.indent_level();
        assert!(stream.is_sol());
        loop {
            if stream.indent_level() < body_indent_level {
                break;
            }
            let statement = FirrtlParser::parse_statement(stream)?;
            statements.push(statement);

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
        Ok(statements)
    }

    pub fn parse_statement(stream: &mut FirrtlStream<'a>)
        -> Result<Statement, FirrtlStreamErr>
    {
        // We have to check for statements that begin with a 'reference'
        // first. Otherwise, this is a "simple" statement where we can 
        // just match on some keyword.
        //
        // NOTE: This syntax ('<='/'<-'/'is invalid') is apparently deprecated 
        // in new versions of FIRRTL
        if FirrtlParser::check_reference(stream) {
            let ref_stmt = FirrtlParser::parse_reference_stmt(stream)?;
            return Ok(ref_stmt);
        } 

        match stream.get_identkw()? {
            "wire" => {
                let (id, ty) = FirrtlParser::parse_wire_stmt(stream)?;
                return Ok(Statement::Wire(id, ty));
            },
            "reg" => {
                let (id, ty, clkexpr, rvexpr) = 
                    FirrtlParser::parse_reg_stmt(stream)?;
                return Ok(Statement::Reg(id, ty, clkexpr, rvexpr));
            },
            "mem" => {
                let mem_decl = FirrtlParser::parse_mem_stmt(stream)?;
                return Ok(Statement::Mem(mem_decl));
            }
            "inst" => { 
                let (id, mid) = FirrtlParser::parse_inst_stmt(stream)?;
                return Ok(Statement::Inst(id, mid));
            },
            "node" => { 
                let (id, expr) = FirrtlParser::parse_node_stmt(stream)?;
                return Ok(Statement::Node(id, expr));
            },
            "when" => { 
                let (expr, ws, es) = FirrtlParser::parse_when_stmt(stream)?;
                let s = Statement::When(expr, ws, es);
                return Ok(s);
            },

            "connect" => { 
                let (refr, expr) = FirrtlParser::parse_connect_stmt(stream)?;
                return Ok(Statement::Connect(refr, expr));
            },
            "invalidate" => { 
                let refr = FirrtlParser::parse_invalidate_stmt(stream)?;
                return Ok(Statement::Invalidate(refr));
            },
            "attach" => { 
                let refs = FirrtlParser::parse_attach_stmt(stream)?;
                return Ok(Statement::Attach(refs));
            },

            // NOTE: These are all only relevant to simulation?
            "define" => { 
                let (sref, rexpr) = FirrtlParser::parse_define_stmt(stream)?;
                return Ok(Statement::Define(sref, rexpr));
            },
            "force_initial" => { 
                let (rexpr, expr) = 
                    FirrtlParser::parse_force_initial_stmt(stream)?;
                return Ok(Statement::ForceInitial(rexpr, expr));
            },
            "release_initial" => { 
                let rexpr = FirrtlParser::parse_release_initial_stmt(stream)?;
                return Ok(Statement::ReleaseInitial(rexpr));
            },
            "force" => { 
                let (e1, e2, rexpr, e3) = 
                    FirrtlParser::parse_force_stmt(stream)?;
                return Ok(Statement::Force(e1, e2, rexpr, e3));
            },
            "release" => { 
                let (e1, e2, rexpr) = FirrtlParser::parse_release_stmt(stream)?;
                return Ok(Statement::Release(e1, e2, rexpr));
            },
            "stop" => { 
                let (e1, e2, lit) = FirrtlParser::parse_stop_stmt(stream)?;
                return Ok(Statement::Stop(e1, e2, lit));
            },
            "printf" => { 
                let (clk, cond, fmtstr, args) = 
                    FirrtlParser::parse_printf_stmt(stream)?;
                return Ok(Statement::Printf(clk, cond, fmtstr, args));
            },


            // FIXME: These are old SFC statements that I don't want
            // to deal with right now (is this CHIRRTL?)
            "cmem" => {
                stream.next_line(); 
                return Ok(Statement::Unimplemented("cmem".to_string()));
            },
            "smem" => { 
                stream.next_line(); 
                return Ok(Statement::Unimplemented("smem".to_string()));
            },
            "infer" => {
                stream.next_line();
                return Ok(Statement::Unimplemented("infer".to_string()));
            },
            "read" => {
                stream.next_line();
                return Ok(Statement::Unimplemented("read".to_string()));
            },
            "write" => {
                stream.next_line();
                return Ok(Statement::Unimplemented("write".to_string()));
            },
            "rdwr" => {
                stream.next_line();
                return Ok(Statement::Unimplemented("rdwr".to_string()));
            },

            // FIXME: These are verification statements that aren't
            // properly in the spec yet
            "assert" => {
                stream.next_line();
                return Ok(Statement::Unimplemented("assert".to_string()));
            },
            "assume" => {
                stream.next_line();
                return Ok(Statement::Unimplemented("assume".to_string()));
            }, 
            "cover" => {
                stream.next_line();
                return Ok(Statement::Unimplemented("cover".to_string()));
            },

            // FIXME: Should probably ignore these in a different way..
            "skip" => { 
                stream.next_token();
                return Ok(Statement::Skip);
            },
            // Otherwise, this is an invalid statement
            identkw @ _ => {
                panic!("unexpected statement keyword {}", identkw);
            },
        }
    }

    pub fn parse_mem_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<MemDecl, FirrtlStreamErr>
    {
        let stmt_blk_level = stream.indent_level();
        stream.match_identkw("mem")?;
        stream.next_token();
        let id = stream.get_identkw()?;
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
        // For now, just do this the messy way: collect everything in a loop
        // and avoid assuming the order.

        let mut datatype = None;
        let mut depth = None;
        let mut read_latency = None;
        let mut write_latency = None;
        let mut read_under_write = None;
        let mut rp_list = Vec::new();
        let mut wp_list = Vec::new();
        let mut rwp_list = Vec::new();
        loop {
            if stream.indent_level() <= stmt_blk_level {
                break;
            }

            if stream.match_identkw("data-type").is_ok() {
                stream.next_token();
                stream.match_punc("=>")?;
                stream.next_token();
                let ty = FirrtlParser::parse_type(stream)?;
                if datatype.is_some() {
                    panic!("Multiple data-type statements?");
                } else {
                    datatype = Some(ty);
                }
            }

            if stream.match_identkw("depth").is_ok() {
                stream.next_token();
                stream.match_punc("=>")?;
                stream.next_token();
                let val = stream.get_lit_int()?;
                stream.next_token();
                if depth.is_some() {
                    panic!("Multiple depth statements?");
                } else {
                    let val = val.parse::<usize>().unwrap();
                    depth = Some(val);
                }
            }

            if stream.match_identkw("read-latency").is_ok() {
                stream.next_token();
                stream.match_punc("=>")?;
                stream.next_token();
                let val = stream.get_lit_int()?;
                stream.next_token();
                if read_latency.is_some() {
                    panic!("Multiple read-latency statements?");
                } else {
                    let val = val.parse::<usize>().unwrap();
                    read_latency = Some(val);
                }
            }

            if stream.match_identkw("write-latency").is_ok() {
                stream.next_token();
                stream.match_punc("=>")?;
                stream.next_token();
                let val = stream.get_lit_int()?;
                stream.next_token();
                if write_latency.is_some() {
                    panic!("Multiple write-latency statements?");
                } else {
                    let val = val.parse::<usize>().unwrap();
                    write_latency = Some(val);
                }

            }

            if stream.match_identkw("read-under-write").is_ok() {
                stream.next_token();
                stream.match_punc("=>")?;
                stream.next_token();
                let ruw = match stream.get_identkw()? {
                    "old" => ReadUnderWrite::Old,
                    "new" => ReadUnderWrite::New,
                    "undefined" => ReadUnderWrite::Undefined,
                    _ => panic!("unexpected read-under-write keyword"),
                };
                stream.next_token();
                if read_under_write.is_some() {
                    panic!("Multiple read-under-write statements?");
                } else {
                    read_under_write = Some(ruw);
                }
            }

            if stream.match_identkw("reader").is_ok() {
                stream.next_token();
                stream.match_punc("=>")?;
                stream.next_token();
                while !stream.is_sol() {
                    let rp_id = stream.get_identkw()?;
                    rp_list.push(rp_id.to_string());
                    stream.next_token();
                }
            }
            if stream.match_identkw("writer").is_ok() {
                stream.next_token();
                stream.match_punc("=>")?;
                stream.next_token();
                while !stream.is_sol() {
                    let wp_id = stream.get_identkw()?;
                    wp_list.push(wp_id.to_string());
                    stream.next_token();
                }
            }
            if stream.match_identkw("readwriter").is_ok() {
                stream.next_token();
                stream.match_punc("=>")?;
                stream.next_token();
                while !stream.is_sol() {
                    let rwp_id = stream.get_identkw()?;
                    rwp_list.push(rwp_id.to_string());
                    stream.next_token();
                }
            }
        }

        let Some(datatype) = datatype else { 
            panic!("Missing data-type declaration");
        };
        let Some(depth) = depth else { 
            panic!("Missing depth declaration");
        };
        let Some(read_latency) = read_latency else { 
            panic!("Missing read-latency declaration");
        };
        let Some(write_latency) = write_latency else { 
            panic!("Missing write-latency declaration");
        };
        let Some(read_under_write) = read_under_write else { 
            panic!("Missing read-under-write declaration");
        };

        Ok(MemDecl::new(id, datatype, depth, read_latency, write_latency,
            read_under_write,
            rp_list,
            wp_list,
            rwp_list
        ))
    }

    pub fn parse_reg_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(String, FirrtlType, Expr, Option<(Expr, Expr)>), 
                    FirrtlStreamErr>
    {
        stream.match_identkw("reg")?;
        stream.next_token();
        let id = stream.get_identkw()?;
        stream.add_module_ctx(id);
        stream.next_token();
        stream.match_punc(":")?;
        stream.next_token();

        let ty = FirrtlParser::parse_type(stream)?;
        let clk_expr = FirrtlParser::parse_expr(stream)?;

        let mut reset_val_expr = None;
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
            let reset_expr = FirrtlParser::parse_expr(stream)?;
            let val_expr = FirrtlParser::parse_expr(stream)?;
            reset_val_expr = Some((reset_expr, val_expr));
            stream.match_punc(")")?;
            stream.next_token();

            // Apparently optional parenthesis?
            if stream.match_punc(")").is_ok() {
                stream.next_token();
            }
        }
        Ok((id.to_string(), ty, clk_expr, reset_val_expr))
    }

    pub fn parse_inst_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(String, String), FirrtlStreamErr>
    {
        // FIXME: legalize module identifiers
        stream.match_identkw("inst")?;
        stream.next_token();
        let id = stream.get_identkw()?;
        stream.add_module_ctx(id);
        stream.next_token();
        stream.match_identkw("of")?;
        stream.next_token();
        let module_id = stream.get_identkw()?;
        stream.next_token();

        Ok((id.to_string(), module_id.to_string()))
    }

    pub fn parse_define_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(StaticReference, RefExpr), FirrtlStreamErr>
    {
        stream.match_identkw("define")?;
        stream.next_token();
        let static_ref = FirrtlParser::parse_static_reference(stream)?;
        stream.match_punc("=")?;
        stream.next_token();
        let ref_expr = FirrtlParser::parse_ref_expr(stream)?;
        Ok((static_ref, ref_expr))
    }

    pub fn parse_attach_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<Vec<Reference>, FirrtlStreamErr>
    {
        stream.match_identkw("attach")?;
        stream.next_token();
        stream.match_punc("(")?;
        stream.next_token();

        let mut refs = Vec::new();
        while !stream.match_punc(")").is_ok() {
            let reference = FirrtlParser::parse_reference(stream)?;
            refs.push(reference);
        }
        stream.match_punc(")")?;
        stream.next_token();
        Ok(refs)
    }

    pub fn parse_force_initial_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(RefExpr, Expr), FirrtlStreamErr>
    {
        stream.match_identkw("force_initial")?;
        stream.next_token();
        stream.match_punc("(")?;
        stream.next_token();

        let ref_expr = FirrtlParser::parse_ref_expr(stream)?;
        let expr = FirrtlParser::parse_expr(stream)?;
        stream.match_punc(")")?;
        stream.next_token();
        Ok((ref_expr, expr))
    }

    pub fn parse_release_initial_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<RefExpr, FirrtlStreamErr>
    {
        stream.match_identkw("release_initial")?;
        stream.next_token();
        stream.match_punc("(")?;
        stream.next_token();
        let ref_expr = FirrtlParser::parse_ref_expr(stream)?;
        stream.match_punc(")")?;
        stream.next_token();
        Ok(ref_expr)
    }

    pub fn parse_force_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(Expr, Expr, RefExpr, Expr), FirrtlStreamErr>
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
        Ok((e1, e2, ref_expr, e3))
    }

    pub fn parse_release_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(Expr, Expr, RefExpr), FirrtlStreamErr>
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
        Ok((e1, e2, ref_expr))
    }

    pub fn parse_connect_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(Reference, Expr), FirrtlStreamErr>
    {
        stream.match_identkw("connect")?;
        stream.next_token();
        let reference = FirrtlParser::parse_reference(stream)?;
        let expr = FirrtlParser::parse_expr(stream)?;
        Ok((reference, expr))
    }
    pub fn parse_invalidate_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<Reference, FirrtlStreamErr>
    {
        stream.match_identkw("invalidate")?;
        stream.next_token();
        let reference = FirrtlParser::parse_reference(stream)?;
        Ok(reference)
    }

    pub fn parse_printf_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(Expr, Expr, String, Vec<Expr>), FirrtlStreamErr>
    {
        stream.match_identkw("printf")?;
        stream.next_token();
        stream.match_punc("(")?;
        stream.next_token();

        let clk_expr = FirrtlParser::parse_expr(stream)?;
        let cond_expr = FirrtlParser::parse_expr(stream)?;
        let fmtstr = stream.get_lit_str()?;
        stream.next_token();

        // Parse expressions until we reach the closing paren
        let mut arg_exprs = Vec::new();
        loop {
            if stream.match_punc(")").is_ok() {
                stream.next_token();
                break;
            }
            let e = FirrtlParser::parse_expr(stream)?;
            arg_exprs.push(e);
        }

        if stream.match_punc(":").is_ok() {
            stream.next_token();
            let _ = stream.get_identkw()?;
            stream.next_token();
        }
        Ok((clk_expr, cond_expr, fmtstr.to_string(), arg_exprs))
    }

    pub fn parse_stop_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(Expr, Expr, usize), FirrtlStreamErr>
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
        Ok((e1, e2, lit.parse().unwrap()))
    }


    pub fn parse_reference_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<Statement, FirrtlStreamErr>
    {
        //println!("parsing reference stmt @ {:?}", stream.remaining_tokens());
        let reference = FirrtlParser::parse_reference(stream)?;
        // Must be an assignment '<=', this is an identifier
        if stream.match_punc("<=").is_ok() {
            stream.next_token();
            let expr = FirrtlParser::parse_expr(stream)?;
            Ok(Statement::Connect(reference, expr))
        } 
        // Must be a partial assignment '<-'?,
        else if stream.match_punc("<-").is_ok() {
            stream.next_token();
            let expr = FirrtlParser::parse_expr(stream)?;
            Ok(Statement::PartialConnect(reference, expr))
        }
        // Must be 'is invalid', this is an identifier
        else if stream.match_identkw("is").is_ok() {
            stream.next_token();
            stream.match_identkw("invalid")?;
            stream.next_token();
            Ok(Statement::Invalidate(reference))
        } else { 
            panic!("unexpected keyword in reference statement?");
        }
    }

    pub fn parse_wire_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(String, FirrtlType), FirrtlStreamErr>
    {
        stream.match_identkw("wire")?;
        stream.next_token();
        let id = stream.get_identkw()?;
        stream.add_module_ctx(id);
        stream.next_token();
        stream.match_punc(":")?;
        stream.next_token();
        let ty = FirrtlParser::parse_type(stream)?;
        Ok((id.to_string(), ty))
    }

    pub fn parse_node_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(String, Expr), FirrtlStreamErr>
    {
        stream.match_identkw("node")?;
        stream.next_token();
        let id = stream.get_identkw()?;
        stream.add_module_ctx(id);
        stream.next_token();
        stream.match_punc("=")?;
        stream.next_token();
        let expr = FirrtlParser::parse_expr(stream)?;
        Ok((id.to_string(), expr))
    }


    pub fn parse_when_stmt(stream: &mut FirrtlStream<'a>)
        -> Result<(Expr, Vec<Statement>, Vec<Statement>), FirrtlStreamErr>
    {
        let current_indent = stream.indent_level();

        // Consume 'when <expr> :'
        stream.match_identkw("when")?;
        stream.next_token();
        let cond_expr = FirrtlParser::parse_expr(stream)?;
        stream.match_punc(":")?;
        stream.next_token();

        // This must be 'when <expr> : <statement>'
        if !stream.is_sol() {
            let mut when_statements = Vec::new();
            let mut else_statements = Vec::new();
            let stmt = FirrtlParser::parse_statement(stream)?;
            when_statements.push(stmt);

            // This must be 'when <expr> : <statement> else : <statement>'
            if stream.match_identkw("else").is_ok() {
                stream.next_token();
                stream.match_punc(":")?;
                stream.next_token();
                let else_stmt = FirrtlParser::parse_statement(stream)?;
                else_statements.push(else_stmt);
            }
            assert!(stream.is_sol());
            return Ok((cond_expr, when_statements, else_statements));
        }

        // Otherwise, this must be 'when <expr> : { statements }'
        assert!(stream.indent_level() > current_indent);
        let when_block = FirrtlParser::parse_statements_block(stream)?;

        // There are no 'else' statements
        if stream.indent_level() < current_indent {
            return Ok((cond_expr, when_block, Vec::new()));
        }

        if stream.match_identkw("else").is_ok() {
            stream.next_token();
            // This must be a block of 'else: { statements }'
            if stream.match_punc(":").is_ok() {
                stream.next_token();
                let else_block = FirrtlParser::parse_statements_block(stream)?;
                return Ok((cond_expr, when_block, else_block));
            } 
            // Otherwise, this is a single 'else { statement }'
            else {
                let mut else_statements = Vec::new();
                let else_stmt = FirrtlParser::parse_statement(stream)?;
                else_statements.push(else_stmt);
                return Ok((cond_expr, when_block, else_statements));
            }
        } 
        else {
            return Ok((cond_expr, when_block, Vec::new()));
        }
    }
}


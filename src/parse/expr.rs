

use crate::ast::*;
use crate::lex::*;
use crate::token::Token;
use crate::parse::FirrtlParser;

/// Parsing for statements and expressions.
impl <'a> FirrtlParser {

    pub fn check_primop_expr(stream: &FirrtlStream<'a>) -> bool {
        let Ok(kw) = stream.get_identkw() else { 
            return false;
        };
        if stream.remaining_tokens().len() >= 2 {
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
    }

    pub fn check_mux_expr(stream: &FirrtlStream<'a>) -> bool {
        if stream.remaining_tokens().len() >= 2 {
            stream.match_identkw("mux").is_ok() &&
                stream.peekn_token(1).match_punc("(").unwrap_or(false)
        } else {
            false
        }
    }

    pub fn check_read_expr(stream: &FirrtlStream<'a>) -> bool {
        if stream.remaining_tokens().len() >= 2 {
            stream.match_identkw("read").is_ok() &&
                stream.peekn_token(1).match_punc("(").unwrap_or(false)
        } else {
            false
        }
    }

    pub fn check_const_expr(stream: &FirrtlStream<'a>) -> bool {
        let Ok(maybe_keyword) = stream.get_identkw() else {
            return false;
        };
        if stream.remaining_tokens().len() >= 2 {
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
    }

    // FIXME: Do we need to handle postfix operations (indexing/subfields)
    // for arbitrary expressions? I don't think this is in the spec, but
    // there's at least one case in CIRCT tests, ie. 
    //
    //  wire agg2 : { a : UInt, flip b : UInt<1> }
    //  ...
    //  out2 <= read(probe(agg2)).b
    //
    pub fn parse_expr(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        // This must be a static_reference (a single identifier)
        // FIXME: Can we actually assume this?
        if stream.remaining_tokens().len() == 1 {
            let ident = stream.get_identkw()?;
            stream.next_token();
            return Ok(());
        }

        if FirrtlParser::check_primop_expr(stream) {
            let primop_expr = FirrtlParser::parse_primop_expr(stream)?;
        }
        else if FirrtlParser::check_const_expr(stream) {
            let const_expr = FirrtlParser::parse_const_expr(stream)?;
        }
        else if FirrtlParser::check_mux_expr(stream) {
            let mux_expr = FirrtlParser::parse_mux_expr(stream)?;
        }
        else if FirrtlParser::check_read_expr(stream) {
            let read_expr = FirrtlParser::parse_read_expr(stream)?;
        } 
        else if FirrtlParser::check_reference(stream) {
            let ref_expr = FirrtlParser::parse_reference(stream)?;
        } 
        else {
            panic!("unable to disambiguate expression? {:?}", 
                stream.remaining_tokens());
        }
        println!("finished expr, {:?}", stream.remaining_tokens());
        Ok(())
    }

    pub fn parse_mux_expr(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr>
    {
        stream.match_identkw("mux")?;
        stream.next_token();
        stream.match_punc("(")?;
        stream.next_token();

        let e1 = FirrtlParser::parse_expr(stream)?;
        let e2 = FirrtlParser::parse_expr(stream)?;
        let e3 = FirrtlParser::parse_expr(stream)?;

        stream.match_punc(")")?;
        stream.next_token();
        Ok(())
    }

    pub fn parse_read_expr(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr>
    {
        stream.match_identkw("read")?;
        stream.next_token();
        stream.match_punc("(")?;
        stream.next_token();
        let ref_expr = FirrtlParser::parse_ref_expr(stream)?;
        stream.match_punc(")")?;
        stream.next_token();
        Ok(())
    }


    pub fn parse_primop_expr(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr>
    {
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
            let e1 = FirrtlParser::parse_expr(stream)?;
            let lit1 = stream.get_lit_int()?;
            stream.next_token();
            stream.match_punc(")")?;
            stream.next_token();
        } 
        else if PrimOp1Expr2Int::from_str(primop_kw).is_some() {
            stream.next_token();
            stream.match_punc("(")?;
            stream.next_token();
            let e1 = FirrtlParser::parse_expr(stream)?;
            let lit1 = stream.get_lit_int()?;
            stream.next_token();
            let lit2 = stream.get_lit_int()?;
            stream.next_token();
            stream.match_punc(")")?;
            stream.next_token();
        } else {
            panic!("eh?");
        }
        Ok(())
    }

    pub fn parse_const_expr(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr>
    {
        let sint_or_uint = stream.get_identkw()?;
        stream.next_token();
        let width = FirrtlParser::parse_optional_typewidth(stream)?;
        stream.match_punc("(")?;
        stream.next_token();
        let lit_val = match sint_or_uint {
            "SInt" => stream.token().get_signed_numeric_literal(),
            "UInt" => stream.token().get_unsigned_numeric_literal(),
            _ => unreachable!(),
        };
        stream.next_token();
        stream.match_punc(")")?;
        stream.next_token();
        Ok(())
    }


    /// Determine if the following tokens qualify as a "reference"
    pub fn check_reference(stream: &mut FirrtlStream<'a>) -> bool {
        // References always start with Token::IdentKw
        let Ok(symbol) = stream.get_identkw() else { 
            return false;
        };

        let matches = &[
            &[Token::Period],
            &[Token::LSquare],
            &[Token::LessEqual],
            &[Token::LessMinus],
            &[Token::IdentKw("is".to_string())],
        ];
        let rem = &stream.remaining_tokens()[1..];
        let ctx_valid = matches.iter().any(|m| rem.starts_with(*m));

        // Matching context should indicate that we have a reference.
        // Otherwise, fall back on checking for a previously-declared symbol
        if ctx_valid {
            return true;
        } 
        else {
            return stream.check_module_ctx(symbol);
        }
    }

    pub fn parse_static_reference(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        // References *must* begin with an identifier
        let ref_ident = stream.get_identkw()?;
        stream.next_token();

        // ... followed by some arbitrary list of subfield/subindex
        loop {
            // Must be a subfield access
            if stream.match_punc(".").is_ok() {
                stream.next_token();
                // FIXME: SFC behavior allows unsigned integer subfield names?
                if let Ok(lit) = stream.get_lit_int() {
                    stream.next_token();
                } 
                else if let Ok(ident) = stream.get_identkw() {
                    stream.next_token();
                } 
            } 
            // Must be a subindex access with an integer literal
            else if stream.match_punc("[").is_ok() {
                // Dynamic indexing always terminates a list of postfix ops,
                // so we should handle this outside the loop?
                if !stream.peekn_token(1).is_lit_int() {
                    break;
                }
                stream.next_token(); // consume '['

                let subindex = stream.get_lit_int()?;
                stream.next_token();
                stream.match_punc("]")?;
                stream.next_token();
            } 
            else {
                break;
            }
        }
        Ok(())
    }


    pub fn parse_reference(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {
        println!("parsing reference @ {:?}", stream.remaining_tokens());

        // All references are *at least* composed of a static reference
        let static_ref = FirrtlParser::parse_static_reference(stream)?;

        // Optional dynamic indexing with some expression
        if stream.match_punc("[").is_ok() {
            stream.next_token();
            let index_expr = FirrtlParser::parse_expr(stream)?;
            stream.match_punc("]")?;
            stream.next_token();
        }

        Ok(())
    }

    pub fn parse_ref_expr(stream: &mut FirrtlStream<'a>)
        -> Result<(), FirrtlStreamErr>
    {

        // This must be 'probe(<static_ref>)' or 'rwprobe(<static_ref>)'
        if stream.peekn_token(1).match_punc("(").unwrap_or(false) {
            let kw = stream.match_identkw_multi(&["probe", "rwprobe"])?;
            stream.next_token();
            stream.match_punc("(")?;
            stream.next_token();
            let static_ref = FirrtlParser::parse_static_reference(stream)?;
            stream.match_punc(")")?;
            stream.next_token();
            Ok(())
        } 
        // Otherwise this is just a static reference
        else {
            let static_ref = FirrtlParser::parse_static_reference(stream)?;
            Ok(())
        }
    }


}




use crate::ast::*;
use crate::lex::*;
use crate::parse::FirrtlParser;

impl <'a> FirrtlParser {
    pub fn parse_optional_typewidth(stream: &mut FirrtlStream<'a>)
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

    pub fn parse_bundle_field(stream: &mut FirrtlStream<'a>) 
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

    pub fn parse_bundle(stream: &mut FirrtlStream<'a>) 
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




    pub fn parse_type(stream: &mut FirrtlStream<'a>) 
        -> Result<(), FirrtlStreamErr>
    {

        // Probe/RWProbe
        if stream.match_identkw("Probe").is_ok() {
            stream.next_token();
            stream.match_punc("<")?;
            stream.next_token();
            let prb_type = FirrtlParser::parse_type(stream)?;
            stream.match_punc(">")?;
            stream.next_token();
            return Ok(());

        } else if stream.match_identkw("RWProbe").is_ok() {
            stream.next_token();
            stream.match_punc("<")?;
            stream.next_token();
            let prb_type = FirrtlParser::parse_type(stream)?;
            stream.match_punc(">")?;
            stream.next_token();
            return Ok(());
        } 
        // Otherwise, this is a ground/aggregate type
        else {
            let is_const = if stream.match_identkw("const").is_ok() {
                stream.next_token();
                true
            } else {
                false
            };

            // This is a bundle type
            if stream.match_punc("{").is_ok() {
                let bundle_type = FirrtlParser::parse_bundle(stream)?;
                //unimplemented!("bundle");
            } 
            // This is some ground type
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
            }

            // Optionally indicates an array type.
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



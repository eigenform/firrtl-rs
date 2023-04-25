
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
        -> Result<BundleField, FirrtlStreamErr>
    {
        //println!("{:?}", stream.remaining_tokens());
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
        let field_type = FirrtlParser::parse_type(stream)?;

        Ok(BundleField::new(flip, field_id, field_type))
    }

    pub fn parse_bundle(stream: &mut FirrtlStream<'a>) 
        -> Result<FirrtlType, FirrtlStreamErr>
    {
        stream.match_punc("{")?;
        stream.next_token();

        let mut fields = Vec::new();

        // NOTE: Always at least one field?
        let field = FirrtlParser::parse_bundle_field(stream)?;
        fields.push(field);
        loop { 
            if stream.match_punc("}").is_ok() {
                stream.next_token();
                break;
            }
            let field = FirrtlParser::parse_bundle_field(stream)?;
            fields.push(field);
        }
        Ok(FirrtlType::Bundle(fields))
    }




    pub fn parse_type(stream: &mut FirrtlStream<'a>) 
        -> Result<FirrtlType, FirrtlStreamErr>
    {

        // Probe/RWProbe
        if stream.match_identkw("Probe").is_ok() {
            stream.next_token();
            stream.match_punc("<")?;
            stream.next_token();
            let prb_type = FirrtlParser::parse_type(stream)?;
            stream.match_punc(">")?;
            stream.next_token();
            Ok(FirrtlType::Ref(FirrtlTypeRef::Probe(Box::new(prb_type))))

        } else if stream.match_identkw("RWProbe").is_ok() {
            stream.next_token();
            stream.match_punc("<")?;
            stream.next_token();
            let prb_type = FirrtlParser::parse_type(stream)?;
            stream.match_punc(">")?;
            stream.next_token();
            Ok(FirrtlType::Ref(FirrtlTypeRef::RWProbe(Box::new(prb_type))))
        } 
        // Otherwise, this is a ground/aggregate type
        else {
            let is_const = if stream.match_identkw("const").is_ok() {
                stream.next_token();
                true
            } else {
                false
            };

            // Either a bundle, or a ground type
            let res_type = if stream.match_punc("{").is_ok() {
                FirrtlParser::parse_bundle(stream)?
            } else {
                let ground_type = stream.get_identkw()?;
                stream.next_token();
                let width = FirrtlParser::parse_optional_typewidth(stream)?;
                match ground_type { 
                    "UInt" => 
                        FirrtlType::Ground(FirrtlTypeGround::UInt(width)),
                    "SInt" => 
                        FirrtlType::Ground(FirrtlTypeGround::SInt(width)),
                    "Analog" => 
                        FirrtlType::Ground(FirrtlTypeGround::Analog(width)),
                    "Clock" => 
                        FirrtlType::Ground(FirrtlTypeGround::Clock),
                    "Reset" => 
                        FirrtlType::Ground(FirrtlTypeGround::Reset),
                    "AsyncReset" => 
                        FirrtlType::Ground(FirrtlTypeGround::AsyncReset),
                    _ => panic!("unknown ground type?"),
                }
            };

            // This is an array of 'res_type'
            if stream.match_punc("[").is_ok() {
                stream.next_token();
                let width = stream.get_lit_int()?;
                stream.next_token();
                stream.match_punc("]")?;
                stream.next_token();
                Ok(FirrtlType::Vector(Box::new(res_type), width.parse().unwrap()))
            } else {
                Ok(res_type)
            }
        }
    }
}



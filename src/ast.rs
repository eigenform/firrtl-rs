

pub enum Direction { Input, Output }

pub enum LiteralNumeric {
    UInt(usize),
    SInt(isize),
}


// NOTE: 'dshlw' only occurs in SFC output?
pub enum PrimOp2Expr {
    Add, Sub, Mul, Div, Mod,
    Lt, Leq, Gt, Geq, Eq, Neq,
    Dshl, Dshlw, Dshr,
    And, Or, Xor, Cat
}
impl PrimOp2Expr {
    pub fn from_str(s: &str) -> Option<Self> { 
        match s {
            "add"  => Some(Self::Add),
            "sub"  => Some(Self::Sub),
            "mul"  => Some(Self::Mul),
            "div"  => Some(Self::Div),
            "mod"  => Some(Self::Mod),
            "lt"   => Some(Self::Lt),
            "leq"  => Some(Self::Leq),
            "gt"   => Some(Self::Gt),
            "geq"  => Some(Self::Geq),
            "eq"   => Some(Self::Eq),
            "neq"  => Some(Self::Neq),
            "dshl" => Some(Self::Dshl),
            "dshlw"=> Some(Self::Dshlw),
            "dshr" => Some(Self::Dshr),
            "and"  => Some(Self::And),
            "or"   => Some(Self::Or),
            "xor"  => Some(Self::Xor),
            "cat"  => Some(Self::Cat),
            _ => None,
        }
    }
}

pub enum PrimOp1Expr {
    AsUInt, AsSInt, AsClock, AsAsyncReset, Cvt,
    Neg, Not,
    Andr, Orr, Xorr
}
impl PrimOp1Expr {
    pub fn from_str(s: &str) -> Option<Self> { 
        match s {
            "asUInt"  => Some(Self::AsUInt),
            "asSInt"  => Some(Self::AsSInt),
            "asClock" => Some(Self::AsClock),
            "asAsyncReset" => Some(Self::AsAsyncReset),
            "cvt"     => Some(Self::Cvt),
            "neg"     => Some(Self::Neg),
            "not"     => Some(Self::Not),
            "andr"    => Some(Self::Andr),
            "orr"     => Some(Self::Orr),
            "xorr"    => Some(Self::Xorr),
            _ => None,
        }
    }
}

pub enum PrimOp1Expr1Int {
    Pad, Shl, Shr, Head, Tail
}
impl PrimOp1Expr1Int {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pad"  => Some(Self::Pad),
            "shl"  => Some(Self::Shl),
            "shr"  => Some(Self::Shr),
            "head" => Some(Self::Head),
            "tail" => Some(Self::Tail),
            _ => None,
        }
    }
}


pub enum PrimOp1Expr2Int {
    Bits
}
impl PrimOp1Expr2Int {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "bits" => Some(Self::Bits),
            _ => None,
        }
    }
}



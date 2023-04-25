
pub struct Circuit { 
    id: String,
    modules:    Vec<Module>,
    intmodules: Vec<Module>,
    extmodules: Vec<Module>,
}
impl Circuit {
    pub fn new(id: impl ToString) -> Self { 
        Self {
            id: id.to_string(),
            modules: Vec::new(),
            intmodules: Vec::new(),
            extmodules: Vec::new(),
        }
    }
}

pub struct Module {
}
pub struct IntModule {
}
pub struct ExtModule {
}

pub type Portlist = Vec<PortDeclaration>;
pub struct PortDeclaration {
    id: String,
    dir: Direction,
    ty: FirrtlType,
}
impl PortDeclaration {
    pub fn new(id: impl ToString, dir: Direction, ty: FirrtlType) -> Self { 
        Self { id: id.to_string(), dir, ty }
    }
}

#[derive(Debug)]
pub enum FirrtlTypeGround {
    Clock, Reset, AsyncReset, 
    UInt(Option<usize>), SInt(Option<usize>), Analog(Option<usize>),
}

#[derive(Debug)]
pub enum FirrtlTypeRef {
    Probe(Box<FirrtlType>),
    RWProbe(Box<FirrtlType>),
}

#[derive(Debug)]
pub enum FirrtlType {
    Ground(FirrtlTypeGround),
    Vector(Box<Self>, usize),
    Bundle(Vec<BundleField>),
    Ref(FirrtlTypeRef),
    None,
}

#[derive(Debug)]
pub struct BundleField {
    flip: bool,
    id: String,
    ty: FirrtlType,
}
impl BundleField {
    pub fn new(flip: bool, id: impl ToString, ty: FirrtlType) -> Self {
        Self { flip, id: id.to_string(), ty }
    }
}


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




use std::ops::RangeInclusive;
use std::collections::*;

pub type Ident = String;

#[derive(Debug)]
pub struct Circuit {
    pub id: Ident,
    pub modules: Vec<Module>,
}
impl Circuit {
    pub fn new(id: impl ToString) -> Self { 
        Self { 
            id: id.to_string(),
            modules: Vec::new(),
        }
    }
    pub fn add_module(&mut self, m: Module) {
        self.modules.push(m);
    }
}

#[derive(Debug)]
pub struct Module { 
    pub id: Ident,
    pub ports: Vec<Port>,
    pub statements: Vec<Statement>,
}
impl Module {
    pub fn new(id: impl ToString) -> Self { 
        Self { 
            id: id.to_string(),
            ports: Vec::new(), 
            statements: Vec::new(), 
        }
    }
    pub fn add_port(&mut self, p: Port) { 
        self.ports.push(p); 
    }
    pub fn add_statement(&mut self, s: Statement) { 
        self.statements.push(s); 
    }
}


#[derive(Debug)]
pub struct Port { 
    pub id: Ident,
    pub ty: Type,
    pub dir: Direction,
}
impl Port { 
    pub fn new(id: impl ToString, ty: Type, dir: Direction) -> Self { 
        Self { id: id.to_string(), ty, dir }
    }
}

#[derive(Debug)]
pub enum Direction { 
    Input, 
    Output, 
}

#[derive(Debug)]
pub enum Type { 
    Clock, 
    Reset,
    AsyncReset,
    UInt(usize),
    SInt(usize),
    Analog(usize),

    Vector(Box<Type>, usize),
    Bundle(Vec<Field>),
}

#[derive(Debug)]
pub struct Field { 
    pub id: Ident,
    pub ty: Type,
    pub flipped: bool,
}
impl Field {
    pub fn new(id: impl ToString, ty: Type, flipped: bool) -> Self {
        Self { id: id.to_string(), ty, flipped }
    }
}

#[derive(Debug)]
pub enum Statement { 
    /// Wire declaration
    Wire { id: Ident, ty: Type, },
    /// Intermediate value declaration
    Node { id: Ident, expr: Expr, },
    /// Register declaration
    Reg { id: Ident, ty: Type, clk: Expr, reset: Expr, init: Expr, },

    /// Module instance declaration
    Inst { id: Ident, module: Ident, },

    Connect { lhs: Reference, rhs: Expr, },

    When { 
        cond: Expr, 
        when_blk: Vec<Statement>, 
        else_blk: Vec<Statement>,
    }
}

// FIXME: Hack until we have a uniform way of representing literals
#[derive(Debug)]
pub enum Literal {
    Dec(usize),
    Hex(String),
    Bin(String),
    Oct(String),
}

#[derive(Debug)]
pub enum Expr { 
    LiteralUInt { width: usize, value: Literal, },
    LiteralSInt { width: usize, value: Literal, },
    Reference(Box<Reference>),
    Mux { cond: Box<Self>, t: Box<Self>, f: Box<Self> },
    PrimOp(PrimOp),
    Read(RefExpr),
}

#[derive(Debug)]
pub enum Reference {
    Static(StaticReference),
    DynamicIndex(Box<Self>, Box<Expr>),
}
#[derive(Debug)]
pub enum StaticReference {
    Id(String),
    Subfield(Box<Self>, String),
    Subindex(Box<Self>, usize),
}

#[derive(Debug)]
pub enum RefExpr {
    Static(StaticReference),
    Probe(StaticReference),
    RwProbe(StaticReference),
}


#[derive(Debug)]
pub enum PrimOp {
    Bits(Box<Expr>, Literal, Literal),

    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Leq(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Geq(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Neq(Box<Expr>, Box<Expr>),
    Dshl(Box<Expr>,Box<Expr>),
    Dshr(Box<Expr>,Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Xor(Box<Expr>, Box<Expr>),
    Cat(Box<Expr>, Box<Expr>),

    Pad(Box<Expr>, Literal),
    Shl(Box<Expr>, Literal),
    Shr(Box<Expr>, Literal),
    Head(Box<Expr>, Literal),
    Tail(Box<Expr>, Literal),

    AsUInt(Box<Expr>),
    AsSInt(Box<Expr>),
    AsClock(Box<Expr>),
    Cvt(Box<Expr>),
    Neg(Box<Expr>),
    Not(Box<Expr>),
    Andr(Box<Expr>),
    Orr(Box<Expr>),
    Xorr(Box<Expr>),
}



//#[cfg(test)]
//mod test {
//    use super::*;
//    #[test]
//    fn foo() {
//        let mut top = Circuit::new("top");
//        let mut m_top = Module::new("top");
//        m_top.add_port(Port::new("in", Type::UInt(1), Direction::In));
//        m_top.add_port(Port::new("out", Type::UInt(1), Direction::Out));
//        m_top.add_statement(Statement::Connect {
//            lhs: Expr::Ref { id: "out".to_string(), },
//            rhs: Expr::Ref { id: "in".to_string(), },
//        });
//    }
//}





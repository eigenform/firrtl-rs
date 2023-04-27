//! Elements in the FIRRTL abstract syntax tree. 

use std::fmt;

/// FIRRTL circuit (`circuit`)
#[derive(Debug)]
pub struct Circuit { 
    pub id: String,
    pub modules: Vec<Module>,
    pub intmodules: Vec<IntModule>,
    pub extmodules: Vec<ExtModule>,
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
    pub fn add_module(&mut self, m: Module) {
        self.modules.push(m);
    }
    pub fn add_intmodule(&mut self, m: IntModule) {
        self.intmodules.push(m);
    }
    pub fn add_extmodule(&mut self, m: ExtModule) {
        self.extmodules.push(m);
    }

    /// Print a FIRRTL statement with some indentation level
    fn dump_indent_stmt(indent: usize, statement: &Statement) {
        match statement {
            Statement::Reg(id, ty, clkexpr, rvexpr) => {
                if let Some((reset_expr, val_expr)) = rvexpr { 
                    println!("{:idt$}reg {}: {}, {} with:", "", 
                             id, ty, clkexpr, idt=indent);
                    println!("{:idt$}(reset => ({}, {}))", "", 
                             reset_expr, val_expr, idt=indent+2);
                } else { 
                    println!("{:idt$}reg {}: {}, {}", "", 
                             id, ty, clkexpr, idt=indent);
                }
            },
            Statement::Wire(id, ty) => {
                println!("{:idt$}wire {}: {}", "", id, ty, idt=indent);
            },
            Statement::Inst(id, mid) => {
                println!("{:idt$}inst {} of {}", "", id, mid, idt=indent);
            },
            Statement::Node(id, expr) => {
                println!("{:idt$}node {} = {}", "", id, expr, idt=indent);
            },

            // FIXME: We're *always* expanding single-line 'when' and 'else'
            Statement::When(condexpr, wblk, eblk) => {
                println!("{:idt$}when {} :", "", condexpr, idt=indent);
                for s in wblk {
                    Self::dump_indent_stmt(indent+2, s);
                }
                if !eblk.is_empty() {
                    println!("{:idt$}else :", "", idt=indent);
                    for s in eblk {
                        Self::dump_indent_stmt(indent+2, s);
                    }
                }
            },

            Statement::Connect(r, e) => {
                println!("{:idt$}connect {}, {}", "", r, e, idt=indent);
            },
            Statement::PartialConnect(r, e) => {
                println!("{:idt$}{} <- {}", "", r, e, idt=indent);
            },
            Statement::Invalidate(r) => {
                println!("{:idt$}invalidate {}", "", r, idt=indent);
            },
            Statement::Skip => {
                println!("{:idt$}skip", "", idt=indent);
            },
            Statement::Printf(e1, e2, s, args) => {
                if args.is_empty() {
                    println!("{:idt$}printf({}, {}, {})", "", 
                            e1, e2, s, idt=indent);
                } else {
                    let arglist: String = args.iter()
                        .map(|x| x.to_string() + ", ").collect::<String>();
                    let argstr = arglist.trim_end_matches(", ");
                    println!("{:idt$}printf({}, {}, {}, {})", "", 
                            e1, e2, s, argstr, idt=indent);
                }
            },
            Statement::Stop(e1, e2, val) => {
                println!("{:idt$}stop({}, {}, {})", "", e1, e2, val, idt=indent);
            },
            Statement::Unimplemented(s) => {
                println!("{:idt$}unimpl_{}()", "", s, idt=indent);
            }
            Statement::Mem(decl) => {
                println!("{:idt$}mem {} :", "", decl.id, idt=indent);
                println!("{:idt$}data-type => {}", "", decl.ty, idt=indent+2);
                println!("{:idt$}depth => {}", "", decl.depth, idt=indent+2);
                println!("{:idt$}read-latency => {}", "", 
                    decl.read_latency, idt=indent+2
                );
                println!("{:idt$}write-latency => {}", "", 
                    decl.write_latency, idt=indent+2
                );
                println!("{:idt$}read-under-write => {}", "", 
                    decl.read_under_write, idt=indent+2
                );
                for rp in &decl.rp_list {
                    println!("{:idt$}reader => {}", "", rp, idt=indent+2);
                }
                for wp in &decl.wp_list {
                    println!("{:idt$}writer => {}", "", wp, idt=indent+2);
                }
                for rwp in &decl.rwp_list {
                    println!("{:idt$}readwriter => {}", "", rwp, idt=indent+2);
                }
            }
            Statement::Attach(refs) => {
                let reflist: String = refs.iter().map(|x| x.to_string() + ", ")
                    .collect::<String>();
                let s = reflist.trim_end_matches(", ");
                println!("{:idt$}attach({})", "", s, idt=indent);
            },
            Statement::Define(sr, re) => {
                println!("{:idt$}define {} = {}", "", sr, re, idt=indent);
            },
            Statement::ForceInitial(re, e) => {
                println!("{:idt$}force_initial({}, {})", "", re, e, idt=indent);
            },
            Statement::Force(e1, e2, re, e3) => {
                println!("{:idt$}force({}, {}, {}, {})", "", 
                    e1, e2, re, e3, idt=indent);
            },
            Statement::Release(e1, e2, re) => {
                println!("{:idt$}release({}, {}, {})", "", 
                    e1, e2, re, idt=indent);
            },
            Statement::ReleaseInitial(re) => {
                println!("{:idt$}release_initial({})", "", re, idt=indent);
            },
            _ => panic!("{:?}", statement),
        }
    }

    /// Write the FIRRTL for this [Circuit] to `stdout`.
    pub fn dump(&self) {
        println!("circuit {}:", self.id);
        for m in &self.modules {
            println!("{:idt$}module {}:", "", m.id, idt=2);
            for port in &m.ports {
                println!("{:idt$}{}", "", port, idt=4);
            }
            for s in &m.statements {
                Self::dump_indent_stmt(4, s);
            }
        }
    }
}


/// FIRRTL module (`module`)
#[derive(Debug)]
pub struct Module {
    pub id: String,
    pub ports: Vec<PortDecl>,
    pub statements: Vec<Statement>,
}
impl Module {
    pub fn new(
        id: impl ToString, 
        ports: Vec<PortDecl>, 
        statements: Vec<Statement>
    ) -> Self 
    {
        Self { id: id.to_string(), ports, statements }
    }
}

/// FIRRTL intrinsic module (`intmodule`)
#[derive(Debug)]
pub struct IntModule {
    pub id: String,
    pub ports: Vec<PortDecl>,
}
impl IntModule {
    pub fn new(id: impl ToString, ports: Vec<PortDecl>) -> Self {
        Self {
            id: id.to_string(),
            ports
        }
    }
}


/// FIRRTL external module (`extmodule`)
#[derive(Debug)]
pub struct ExtModule {
    pub id: String,
    pub ports: Vec<PortDecl>,
}
impl ExtModule {
    pub fn new(id: impl ToString, ports: Vec<PortDecl>) -> Self {
        Self {
            id: id.to_string(),
            ports,
        }
    }
}


/// FIRRTL module port declaration
#[derive(Debug)]
pub struct PortDecl {
    pub id: String,
    pub dir: Direction,
    pub ty: FirrtlType,
}
impl PortDecl {
    pub fn new(id: impl ToString, dir: Direction, ty: FirrtlType) -> Self { 
        Self { id: id.to_string(), dir, ty }
    }
}
impl fmt::Display for PortDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{} {} : {}", self.dir, self.id, self.ty)
    }
}

/// FIRRTL ground datatypes
#[derive(Debug)]
pub enum FirrtlTypeGround {
    Clock, Reset, AsyncReset, 
    UInt(Option<usize>), SInt(Option<usize>), Analog(Option<usize>),
}
impl fmt::Display for FirrtlTypeGround {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let s = match self {
            Self::Clock => "Clock".to_string(),
            Self::Reset => "Reset".to_string(),
            Self::AsyncReset => "AsyncReset".to_string(),
            Self::UInt(None) => "UInt".to_string(),
            Self::SInt(None) => "SInt".to_string(),
            Self::Analog(None) => "Analog".to_string(),
            Self::UInt(Some(w)) => format!("UInt<{}>", w),
            Self::SInt(Some(w)) => format!("SInt<{}>", w),
            Self::Analog(Some(w)) => format!("Analog<{}>", w),
        };
        write!(f, "{}", s)
    }
}


#[derive(Debug)]
pub enum FirrtlTypeRef {
    Probe(Box<FirrtlType>),
    RWProbe(Box<FirrtlType>),
}
impl fmt::Display for FirrtlTypeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self { 
            Self::Probe(ty) => write!(f, "Probe<{}>", ty),
            Self::RWProbe(ty) => write!(f, "RWProbe<{}>", ty),
        }
    }
}

/// FIRRTL datatypes
#[derive(Debug)]
pub enum FirrtlType {
    Ground(FirrtlTypeGround),
    Vector(Box<Self>, usize),
    Bundle(Vec<BundleField>),
    Ref(FirrtlTypeRef),
    None,
}
impl fmt::Display for FirrtlType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self { 
            Self::Ground(g) => write!(f, "{}", g),
            Self::Vector(ty, sz) => write!(f, "{}[{}]", ty, sz),
            Self::Bundle(fields) => {
                let flist: String = fields.iter().map(|x| x.to_string() + ", ")
                    .collect::<String>();
                let s = flist.trim_end_matches(", ");
                write!(f, "{{ {} }}", s)
            },
            Self::Ref(rty) => write!(f, "{}", rty),
            Self::None => { 
                panic!("cant format nonetype?");
            },
        }
    }
}

#[derive(Debug)]
pub struct BundleField {
    pub flip: bool,
    pub id: String,
    pub ty: FirrtlType,
}
impl BundleField {
    pub fn new(flip: bool, id: impl ToString, ty: FirrtlType) -> Self {
        Self { flip, id: id.to_string(), ty }
    }
}
impl fmt::Display for BundleField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let flip = if self.flip {"flip"} else { "" };
        if self.flip {
            write!(f, "flip {} : {}", self.id, self.ty)
        } else { 
            write!(f, "{} : {}", self.id, self.ty)
        }
    }
}

#[derive(Debug)]
pub enum Reference {
    Static(StaticReference),
    DynamicIndex(StaticReference, Box<Expr>),
}
impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self { 
            Self::Static(r) => write!(f, "{}", r),
            Self::DynamicIndex(r, expr) => write!(f, "{}[{}]", r, expr),
        }
    }
}

#[derive(Debug)]
pub enum StaticReference { 
    Static(String),
    Subfield(Box<Self>, String),
    Subindex(Box<Self>, usize),
}
impl StaticReference {
    pub fn new_static(s: impl ToString) -> Self {
        Self::Static(s.to_string())
    }
}
impl fmt::Display for StaticReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self { 
            Self::Static(r) => write!(f, "{}", r),
            Self::Subfield(r, field) => write!(f, "{}.{}", r, field),
            Self::Subindex(r, idx) => write!(f, "{}[{}]", r, idx),
        }
    }
}


/// FIRRTL memory declaration (`mem` statement)
#[derive(Debug)]
pub struct MemDecl {
    pub id: String,
    pub ty: FirrtlType,
    pub depth: usize,
    pub write_latency: usize,
    pub read_latency: usize,
    pub rp_list: Vec<String>,
    pub wp_list: Vec<String>,
    pub rwp_list: Vec<String>,
    pub read_under_write: ReadUnderWrite,
}
impl MemDecl {
    pub fn new(
        id: impl ToString, 
        ty: FirrtlType,
        depth: usize,
        read_latency: usize,
        write_latency: usize,
        read_under_write: ReadUnderWrite,
        rp_list: Vec<String>,
        wp_list: Vec<String>,
        rwp_list: Vec<String>,
    ) -> Self {
        Self { 
            id: id.to_string(), 
            depth, 
            ty, 
            read_latency, 
            write_latency, 
            read_under_write,
            rp_list, 
            wp_list, 
            rwp_list
        }
    }
}


/// FIRRTL statements
#[derive(Debug)]
pub enum Statement {
    Wire(String, FirrtlType),
    Reg(String, FirrtlType, Expr, Option<(Expr, Expr)>),
    Inst(String, String),
    Mem(MemDecl),
    Node(String, Expr),

    Attach(Vec<Reference>),
    PartialConnect(Reference, Expr),
    Connect(Reference, Expr),
    Invalidate(Reference),
    When(Expr, Vec<Self>, Vec<Self>),

    Stop(Expr, Expr, usize),
    Force(Expr, Expr, RefExpr, Expr),
    Release(Expr, Expr, RefExpr),
    ForceInitial(RefExpr, Expr),
    ReleaseInitial(RefExpr),
    Define(StaticReference, RefExpr),
    Printf(Expr, Expr, String, Vec<Expr>),

    Skip,
    Unimplemented(String),
}

/// FIRRTL expressions
#[derive(Debug)]
pub enum Expr {
    Ref(Reference),
    Const(FirrtlType, LiteralNumeric),
    Read(RefExpr),
    Mux(Box<Self>, Box<Self>, Box<Self>),
    PrimOp2Expr(PrimOp2Expr, Box<Self>, Box<Self>),
    PrimOp1Expr(PrimOp1Expr, Box<Self>),
    PrimOp1Expr1Int(PrimOp1Expr1Int, Box<Self>, usize),
    PrimOp1Expr2Int(PrimOp1Expr2Int, Box<Self>, usize, usize),
    None,
}
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Ref(r) => write!(f, "{}", r),
            Self::Const(ty, lit) => write!(f, "{}({})", ty, lit),
            Self::Read(rexpr) => write!(f, "read({})", rexpr),
            Self::Mux(e1,e2,e3) => write!(f, "mux({}, {}, {})", e1, e2, e3),
            Self::PrimOp2Expr(op, e1, e2) => {
                write!(f, "{}({}, {})", op, e1, e2)
            },
            Self::PrimOp1Expr(op, e1) => write!(f, "{}({})", op, e1),
            Self::PrimOp1Expr1Int(op, e1, lit) => {
                write!(f, "{}({}, {})", op, e1, lit)
            },
            Self::PrimOp1Expr2Int(op, e1, lit1, lit2) => {
                write!(f, "{}({}, {}, {})", op, e1, lit1, lit2)
            },
            Self::None => panic!("format none expr?"),

        }
    }
}

/// FIRRTL reference expressions
#[derive(Debug)]
pub enum RefExpr {
    Static(StaticReference),
    RwProbe(StaticReference),
    Probe(StaticReference),
}
impl fmt::Display for RefExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self { 
            Self::Static(r) => write!(f, "{}", r),
            Self::RwProbe(r) => write!(f, "rwprobe({})", r),
            Self::Probe(r) => write!(f, "probe({})", r),
        }
    }
}

/// FIRRTL numeric literals
#[derive(Debug)]
pub enum LiteralNumeric {
    UInt(usize), SInt(isize),
}
impl fmt::Display for LiteralNumeric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self { 
            Self::UInt(v) => write!(f, "{}", v),
            Self::SInt(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug)]
pub enum ReadUnderWrite { 
    Old, New, Undefined
}
impl ReadUnderWrite {
    pub fn from_str(s: &str) -> Option<Self> {
        match s { 
            "old" => Some(Self::Old),
            "new" => Some(Self::New),
            "undefined" => Some(Self::Undefined),
            _ => None,
        }
    }
    pub fn to_str(&self) -> &'static str {
        match self { 
            Self::Old => "old",
            Self::New => "new",
            Self::Undefined => "undefined",
        }
    }
}
impl fmt::Display for ReadUnderWrite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", Self::to_str(self))
    }
}

/// FIRRTL port direction
#[derive(Debug)]
pub enum Direction { 
    Input, Output 
}
impl Direction {
    pub fn from_str(s: &str) -> Option<Self> {
        match s { 
            "input" => Some(Self::Input),
            "output" => Some(Self::Output),
            _ => None,
        }
    }
    pub fn to_str(&self) -> &'static str {
        match self { 
            Self::Input => "input",
            Self::Output => "output",
        }
    }
}
impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", Self::to_str(self))
    }
}



/// Primitive operations (2 expressions)
///
/// NOTE: 'dshlw' only occurs in SFC output?
#[derive(Debug)]
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
    pub fn to_str(&self) -> &'static str {
        match self { 
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Div => "div",
            Self::Mod => "mod",
            Self::Lt => "lt",
            Self::Leq => "leq",
            Self::Gt => "gt",
            Self::Geq => "geq",
            Self::Eq => "eq",
            Self::Neq => "ne",
            Self::Dshl => "dshl",
            Self::Dshlw => "dshlw",
            Self::Dshr => "dsh",
            Self::And => "and",
            Self::Or => "or",
            Self::Xor => "xor",
            Self::Cat => "cat",
        }
    }
}
impl fmt::Display for PrimOp2Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", Self::to_str(self))
    }
}


/// Primitive operations (1 expression)
#[derive(Debug)]
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
    pub fn to_str(&self) -> &'static str {
        match self { 
            Self::AsUInt => "asUInt",
            Self::AsSInt => "asSInt",
            Self::AsClock => "asClock",
            Self::AsAsyncReset => "asAsyncReset",
            Self::Cvt => "cvt",
            Self::Neg => "neg",
            Self::Not => "not",
            Self::Andr => "andr",
            Self::Orr => "orr",
            Self::Xorr => "xorr",
        }
    }
}
impl fmt::Display for PrimOp1Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", Self::to_str(self))
    }
}


/// Primitive operations (1 expression, 1 integer literal)
#[derive(Debug)]
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
    pub fn to_str(&self) -> &'static str {
        match self { 
            Self::Pad => "pad",
            Self::Shl => "shl",
            Self::Shr => "shr",
            Self::Head => "head",
            Self::Tail => "tail",
        }
    }
}
impl fmt::Display for PrimOp1Expr1Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", Self::to_str(self))
    }
}


/// Primitive operations (1 expression, 2 integer literals)
#[derive(Debug)]
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
    pub fn to_str(&self) -> &'static str { 
        match self { 
            Self::Bits => "bits",
        }
    }
}
impl fmt::Display for PrimOp1Expr2Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", Self::to_str(self))
    }
}





use std::collections::*;
use num::{ BigUint, Zero };

use firrtl::ast::*;

/// An indentifier for a signal.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SignalId(usize);
impl SignalId {
    pub fn new(x: usize) -> Self { Self(x) }
    pub fn val(&self) -> usize { self.0 }
}

/// The state of a signal during simulation. 
#[derive(Debug)]
pub struct Signal {
    pub ty: FirrtlType,
    pub val: SignalValue,
    pub reg: bool,
}
impl Signal {
    pub fn new(ty: &FirrtlType) -> Self {
        let val = SignalValue::new(ty);
        Self { 
            ty: ty.clone(),
            val: SignalValue::Undefined,
            reg: false,
        }
    }
    pub fn new_reg(ty: &FirrtlType) -> Self {
        let val = SignalValue::new(ty);
        Self { 
            ty: ty.clone(),
            val: SignalValue::Undefined,
            reg: true,
        }

    }
}

/// Container for the simulated value of a signal.
///
/// NOTE: The distinction between "instantaneous" and "registered" values
/// is probably made somewhere else.
#[derive(Debug)]
pub enum SignalValue {
    Bundle(HashMap<String, Self>),
    Vector(Vec<Self>, usize),
    UInt(BigUint, usize),
    Bool(bool),
    Undefined,
}
impl SignalValue {

    /// Resolve a [FirrtlType] into a [SignalValue].
    pub fn new(ty: &FirrtlType) -> Self { 
        match ty {
            FirrtlType::Bundle(fields) => {
                let mut map = HashMap::new();
                for field in fields {
                    let s = Self::new(&field.ty);
                    map.insert(field.id.clone(), s);
                }
                Self::Bundle(map)
            },
            FirrtlType::Vector(ty, size) => {
                let mut data = Vec::new();
                for _ in 0..*size { 
                    let v = Self::new(ty);
                    data.push(v); 
                }
                Self::Vector(data, *size)
            },
            FirrtlType::Ground(gt) => match gt {
                FirrtlTypeGround::Clock => Self::Bool(false),
                FirrtlTypeGround::Reset => Self::Bool(false),
                FirrtlTypeGround::AsyncReset => Self::Bool(false),
                FirrtlTypeGround::UInt(w) |
                FirrtlTypeGround::SInt(w) |
                FirrtlTypeGround::Analog(w) => {
                    let width = w.unwrap_or(0);
                    Self::UInt(BigUint::zero(), width)
                },
            },
            FirrtlType::Ref(r) => Self::Undefined,
            FirrtlType::None => Self::Undefined,
        }
    }
}

/// Storage for tracking the state of signals
pub struct SignalTable {
    map: HashMap<String, SignalId>,
    signals: Vec<Signal>,
}
impl SignalTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            signals: Vec::new(),
        }
    }

    pub fn dump(&self) {
        for (ident, id) in self.map.iter() {
            println!("{:?} '{}': {:?}", id, ident, self[*id]);
        }
    }

    /// Get a reference to a signal from a FIRRTL identifier
    pub fn signal_from_ident(&self, ident: &str) -> &Signal {
        let id = self.map.get(ident).unwrap();
        &self.signals[id.val()]
    }

    pub fn signal_from_ident_mut(&mut self, ident: &str) -> &mut Signal {
        if let Some(id) = self.map.get(ident) {
            &mut self.signals[id.val()]
        } else {
            panic!("no ident {}", ident);
        }
    }


    /// Get a reference to a signal from a FIRRTL 'reference' 
    pub fn signal_from_ref(&self, r: &Reference) -> &Signal {
        let ident = r.get_ident();
        self.signal_from_ident(ident)
    }

    pub fn signal_from_ref_mut(&mut self, r: &Reference) -> &mut Signal {
        let ident = r.get_ident();
        self.signal_from_ident_mut(ident)
    }



    /// Add a new signal
    pub fn alloc(&mut self, ident: impl ToString, ty: &FirrtlType)
        -> SignalId 
    {
        let next_id = SignalId::new(self.signals.len());
        self.map.insert(ident.to_string(), next_id);
        let s = Signal::new(ty);
        self.signals.push(s);
        next_id
    }

}

impl std::ops::Index<SignalId> for SignalTable {
    type Output = Signal;
    fn index(&self, id: SignalId) -> &Self::Output {
        &self.signals[id.val()]
    }
}
impl std::ops::IndexMut<SignalId> for SignalTable {
    fn index_mut(&mut self, id: SignalId) -> &mut Self::Output {
        &mut self.signals[id.val()]
    }
}



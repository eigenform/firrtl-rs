//! Sketching out some kind of FIRRTL simulator
//!
//! Miscellania/Notes
//! =================
//!
//! Before building up the simulator state, we probably have to implement a 
//! bunch of passes for lowering the input somewhat, ie. 
//!
//! - Some types/widths may be inferred (and we need to infer them)
//! - Ideally all of the combinational paths are SSA'd
//! - etc
//!
//! AFAICT, there isn't a nice way to get CIRCT to do this for us yet (while
//! simultaneously re-exporting the corresponding specification FIRRTL)?
//!
//! Now that I'm thinking about this, it would be really nice if FIRRTL 
//! supported bundle declarations (ie. giving them names). Hopefully this
//! is on the roadmap. 
//!

pub mod signal;

use std::collections::*;
use num::{ BigUint, Zero };

use firrtl::{ FirrtlFile, FirrtlParseError };
use firrtl::ast::*;

use crate::signal::*;


/// For walking the FIRRTL AST and turning everything into simulator state.
pub struct FirrtlVisitor {
    tbl: SignalTable,
}
impl FirrtlVisitor {
    pub fn new() -> Self {
        Self {
            tbl: SignalTable::new(),
        }
    }

    pub fn run(&mut self, m: &Module) {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        for port in &m.ports {
            let sid = self.tbl.alloc(&port.id, &port.ty);
            if port.dir == Direction::Input {
                inputs.push(sid);
            } else {
                outputs.push(sid);
            }
        }
        self.walk_statements(&m.statements);

        self.tbl.dump();
    }

    /// Resolve the FIRRTL type of the provided FIRRTL 'reference'
    fn resolve_ref_type(&self, refr: &Reference) -> FirrtlType {
        let s = self.tbl.signal_from_ref(refr);
        match refr { 
            Reference::Static(sr) => match sr {
                StaticReference::Static(_) => { s.ty.clone() },
                StaticReference::Subfield(_, field_name) => {
                    s.ty.bundle_field_type(field_name).unwrap().clone()
                },
                StaticReference::Subindex(_, _) => { s.ty.clone() },
            },
            Reference::DynamicIndex(_, _) => { s.ty.clone() },
        }
    }

    /// Resolve the FIRRTL type of the provided FIRRTL expression. 
    fn resolve_expr_type(&self, expr: &Expr) -> FirrtlType {
        match expr {
            Expr::None => FirrtlType::None,
            Expr::Const(ty, _) => ty.clone(),
            Expr::Ref(refr) => {
                self.resolve_ref_type(refr)
            },
            Expr::PrimOp2Expr(_, e1, e2) => {
                let t1 = self.resolve_expr_type(e1);
                let t2 = self.resolve_expr_type(e2);
                println!("t1 {:?}", t1);
                println!("t2 {:?}", t2);
                FirrtlType::None
            },
            Expr::PrimOp1Expr(_, e1) => {
                self.resolve_expr_type(e1)
            },
            Expr::PrimOp1Expr1Int(_, e1, lit1) => {
                self.resolve_expr_type(e1)
            },
            Expr::PrimOp1Expr2Int(_, e1, lit1, lit2) => {
                self.resolve_expr_type(e1)
            },
            _ => unimplemented!("{:?}", expr),
        }
    }

    /// Walk a block of FIRRTL statements
    fn walk_statements(&mut self, statements: &Vec<Statement>) {
        for s in statements { 
            match s {
                Statement::Wire(id, ty) => {
                    self.tbl.alloc(id, ty);
                },
                Statement::Connect(refr, expr) => {
                    let tgt = self.tbl.signal_from_ref_mut(refr);
                    if tgt.ty.width().is_none() {
                    }

                    let expr_ty = self.resolve_expr_type(expr);
                },
                Statement::Node(id, expr) => {
                    let ty = self.resolve_expr_type(expr);
                    self.tbl.alloc(id, &ty);
                },
                Statement::Reg(id, ty, clkexpr, rstexpr) => {
                    self.tbl.alloc(id, &ty);
                },
                Statement::When(cond_expr, when_stmt, else_stmt) => {
                    let ty = self.resolve_expr_type(cond_expr);
                    self.walk_statements(when_stmt);
                    self.walk_statements(else_stmt);
                },
                Statement::Inst(id, module_id) => {
                },
                _ => unimplemented!("{:?}", s),
            }
        }
    }
}


#[cfg(test)]
mod test {
    use crate::*;
    use firrtl::*;
    use firrtl::ast::*;

    #[test]
    fn foo() -> Result<(), String> {
        let f = FirrtlFile::from_file("../chisel-tests/firrtl/MyAlu.fir");
        //let f = FirrtlFile::from_file("/tmp/foo.fir");
        let c = f.parse().map_err(|e| e.kind.message())?;
        c.dump();
        println!();

        let mut builder = FirrtlVisitor::new();
        builder.run(c.top_module().unwrap());

        //let mut sim = Simulator::new();
        //sim.load_circuit(&c);

        Ok(())

    }
}

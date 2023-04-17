package ft

import chisel3._

// FIXME: Uhhh is this actually emitting what I want? 
object FirrtlEmitter {
  def apply(gen: => RawModule) = {
    (new circt.stage.ChiselStage).execute(
      Array("--target-dir", "firrtl"),
      Seq(
        chisel3.stage.ChiselGeneratorAnnotation(() => gen),
        circt.stage.CIRCTTargetAnnotation(circt.stage.CIRCTTarget.CHIRRTL),
      ),
    )
  }
}



class MyNestedModule extends Module {

  class Module1 extends Module { 
    val io = IO(new Bundle {
      val in  = Input(UInt(32.W))
      val out = Output(UInt(32.W))
    })
    io.out := io.in + 0x5a5a5a5a.U(32.W)
  }

  class Module2 extends Module { 
    val io = IO(new Bundle {
      val in  = Input(UInt(32.W))
      val out = Output(UInt(32.W))
    })
    io.out := io.in & 0x5a5a5a5a.U(32.W)
  }

  val io = IO(new Bundle { 
    val a = Input(UInt(32.W))
    val b = Input(UInt(32.W))
    val c = Output(UInt(32.W))
    val d = Output(UInt(32.W))
  })

  val mod1 = Module(new Module1())
  val mod2 = Module(new Module2())
  mod1.io.in := io.a
  mod2.io.in := io.b
  io.c := mod1.io.out
  io.d := mod2.io.out

}


object Elaborate extends App {
  FirrtlEmitter(new ft.alu.MyAlu)
  FirrtlEmitter(new ft.alu.GCD)
  FirrtlEmitter(new MyNestedModule)
}

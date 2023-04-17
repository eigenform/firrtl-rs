package ft.alu

import chisel3._
import chisel3.util._

object AluOp extends ChiselEnum {
  val ADD = Value
  val SUB = Value
  val AND = Value
  val OR  = Value
  val XOR = Value
}

class MyAlu extends Module { 
  val io = IO(new Bundle { 
    val op  = Input(AluOp())
    val x   = Input(UInt(32.W))
    val y   = Input(UInt(32.W))
    val res = Output(UInt(32.W))
  })

  val x = io.x
  val y = io.y
  val res = WireDefault(0.U(32.W))

  switch (io.op) {
    is (AluOp.ADD) { res := x + y }
    is (AluOp.SUB) { res := x - y }
    is (AluOp.AND) { res := x & y }
    is (AluOp.OR)  { res := x | y }
    is (AluOp.XOR) { res := x ^ y }
  }
  io.res := res
}

class GCD extends Module {
  val io = IO(new Bundle {
    val a     = Input(UInt(16.W))
    val b     = Input(UInt(16.W))
    val load  = Input(Bool())
    val out   = Output(UInt(16.W))
    val valid = Output(Bool())
  })
  val x = Reg(UInt())
  val y = Reg(UInt())

  when (io.load) {
    x := io.a; y := io.b
  } .otherwise {
    when (x > y) {
      x := x - y
    } .elsewhen (x <= y) {
      y := y - x
    }
  }

  io.out := x
  io.valid := y === 0.U
}


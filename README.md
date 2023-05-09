
A crate for parsing [FIRRTL](https://github.com/chipsalliance/firrtl-spec).

> :warning: This is currently **unfinished** and probably **incorrect** in 
> many places.

# Notes

I'm writing this with the intention of experimenting with designs 
in [Chisel](https://github.com/chipsalliance/chisel) (ie. for simulation). 
For now, it seems like the `circt.stage.CIRCTTarget.CHIRRTL` target is the 
easiest way to emit something like specification FIRRTL. 

# Usage

You can run some simple tests with the following:

```
$ make -C chisel-tests
...
$ cargo test -- --nocapture 
```


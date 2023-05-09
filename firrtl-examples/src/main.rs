//! Print the parsed contents of a FIRRTL file

use std::env;
use firrtl::{ FirrtlParseError, FirrtlFile };

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    if args.len() < 2 {
        return Err(format!("usage: {} <.fir file>", args[0]));
    }

    let ff = FirrtlFile::from_file(&args[1]);
    let circuit = ff.parse().map_err(|e: FirrtlParseError| {
        e.kind.message()
    })?;
                                            
    circuit.dump();
    Ok(())
}

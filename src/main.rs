pub mod err;
pub mod parser;
pub mod reader;
pub mod s_time;
pub mod tokenize;
use err::CanErr;
use std::io::Read;
//use tokenize::*;
use parser::*;

fn main() -> Result<(), err::BoxErr> {
    let mut input = std::io::stdin();
    let mut s = String::new();
    input.read_to_string(&mut s).as_err()?;
    let mut tk = Parser::new(&s);
    loop {
        let ac = tk.next_action().as_err()?;
        if ac.ad == ActionData::End {
            break;
        }
        println!("Action :: {:?}", ac);
    }

    Ok(())
}

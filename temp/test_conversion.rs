use std::fs;
use pgf2json::{parse_pgf, pgf_to_json};
use bytes::Bytes;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read("grammars/compare/generated_Zero.pgf")?;
    let bytes = Bytes::from(data);
    
    let pgf = parse_pgf(&bytes)?;
    let json_output = pgf_to_json(&pgf)?;
    
    println!("{}", json_output);
    
    Ok(())
}
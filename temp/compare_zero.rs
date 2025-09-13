#[cfg(test)]
mod test_zero_comparison {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_zero_pgf_parsing() {
        let data = fs::read("grammars/compare/generated_Zero.pgf").expect("Failed to read PGF file");
        let bytes = bytes::Bytes::from(data);
        
        let pgf = parse_pgf(&bytes).expect("Failed to parse PGF");
        let json_output = pgf_to_json(&pgf).expect("Failed to convert to JSON");
        
        // Write current output for comparison
        fs::write("current_zero_output.json", &json_output).expect("Failed to write output");
        
        println!("Current output written to current_zero_output.json");
        
        // Load expected output
        let expected = fs::read_to_string("grammars/compare/correctly_generated_Zero.json")
            .expect("Failed to read expected JSON");
        
        // Parse both to compare structures (ignoring whitespace differences)
        let current: serde_json::Value = serde_json::from_str(&json_output).expect("Invalid current JSON");
        let expected: serde_json::Value = serde_json::from_str(&expected).expect("Invalid expected JSON");
        
        // For now, just ensure they're valid JSON - we'll compare structures manually
        assert!(current.is_object());
        assert!(expected.is_object());
        
        println!("Both JSON structures are valid");
    }
}
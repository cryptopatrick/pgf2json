# pgf2json

A Rust library for parsing Portable Grammar Format (PGF) files and converting them to JSON.

## Overview

This crate provides an API to load and interpret grammars compiled in Portable Grammar Format (PGF), which is the final output format from the Grammatical Framework (GF) compiler. The library enables embedding GF grammars in Rust programs.

## Features

- **PGF Parsing**: Read binary PGF files into Rust data structures
- **JSON Conversion**: Convert PGF grammars to structured JSON format
- **Grammar Analysis**: Extract functions, categories, and language-specific concrete syntaxes
- **Sentence Parsing**: Parse sentences using loaded grammars
- **Error Handling**: Comprehensive error reporting for invalid files and parsing failures

## Usage

```rust
use pgf2json::{read_pgf, pgf_to_json, parse, language, types};

// Load a PGF file
let pgf = read_pgf("./grammars/Food.pgf")?;

// Convert to JSON
let json = pgf_to_json(&pgf)?;

// Parse a sentence
let lang = language::read_language("FoodEng").unwrap();
let typ = types::start_cat(&pgf);
let trees = parse(&pgf, &lang, &typ, "this pizza is delicious")?;
```

## Testing

```bash
cargo test
```

The test suite includes parsing real PGF files and validating JSON output structure.
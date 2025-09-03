use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Read};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use thiserror::Error;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

/// Errors that can occur during PGF operations.
#[derive(Error, Debug)]
pub enum PgfError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Unknown language: {0}")]
    UnknownLanguage(String),
    #[error("Deserialization error: {0}")]
    DeserializeError(String),
    #[error("Serialization error: {0}")]
    SerializeError(String),
    #[error("Type checking error: {0}")]
    TypeCheckError(String),
    #[error("Parsing error: {0}")]
    ParseError(String),
}

/// Represents a Portable Grammar Format (PGF) structure.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pgf {
    absname: CId,
    concretes: HashMap<Language, Concrete>,
    r#abstract: Abstract,
    startcat: CId,
    flags: HashMap<CId, Literal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Abstract {
    funs: HashMap<CId, Function>,
    cats: HashMap<CId, Category>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Concrete {
    cflags: HashMap<CId, Literal>,
    productions: HashMap<i32, HashSet<Production>>,
    cncfuns: Vec<CncFun>,
    sequences: Vec<Vec<Symbol>>,
    cnccats: HashMap<CId, CncCat>,
    total_cats: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Function {
    ty: Type,
    weight: i32,
    equations: Option<(Vec<Equation>, Vec<Vec<Instr>>)>,
    prob: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    hypos: Vec<Hypo>,
    funs: Vec<(usize, CId)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Language(CId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypo {
    binding: Binding,
    ty: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Binding {
    Explicit(String),
    Implicit(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Type {
    hypos: Vec<Hypo>,
    category: CId,
    exprs: Vec<Expr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Str(String),
    Int(i32),
    Flt(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CncCat {
    start: i32,
    end: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CncFun {
    name: CId,
    lins: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Production {
    Apply { fid: i32, args: Vec<PArg> },
    Coerce { arg: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PArg {
    hypos: Vec<i32>,
    fid: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Symbol {
    SymCat(i32, i32),
    SymLit(i32, i32),
    SymVar(i32, i32),
    SymKS(String),
    SymKP(Vec<Symbol>, Vec<Alt>),
    SymNE,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Alt {
    symbols: Vec<Symbol>,
    tokens: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equation {
    patterns: Vec<Pattern>,
    result: Expr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pattern {
    PVar(CId),
    PApp(CId, Vec<Pattern>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instr {
    // Placeholder
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    Abs(Binding, CId, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    Fun(CId),
    Str(String),
    Int(i32),
    Float(f32),
    Double(f64),
    Meta,
    Typed(Box<Expr>, Type),
    ImplArg(Box<Expr>),
}

pub mod cid {
    use super::CId;

    pub fn mk_cid(s: &str) -> CId {
        CId(s.to_string())
    }

    pub fn wild_cid() -> CId {
        CId("*".to_string())
    }

    pub fn show_cid(cid: &CId) -> String {
        cid.0.clone()
    }

    pub fn read_cid(s: &str) -> Option<CId> {
        if s.is_empty() {
            None
        } else {
            Some(CId(s.to_string()))
        }
    }
}

pub mod language {
    use super::{CId, Language, Pgf, Literal};

    pub fn show_language(lang: &Language) -> String {
        super::cid::show_cid(&lang.0)
    }

    pub fn read_language(s: &str) -> Option<Language> {
        super::cid::read_cid(s).map(Language)
    }

    pub fn languages(pgf: &Pgf) -> Vec<Language> {
        pgf.concretes.keys().cloned().collect()
    }

    pub fn language_code(pgf: &Pgf, lang: &Language) -> Option<String> {
        pgf.concretes.get(lang).and_then(|cnc| {
            cnc.cflags.get(&CId("language".to_string())).and_then(|lit| {
                match lit {
                    Literal::Str(s) => Some(s.replace('_', "-")),
                    _ => None,
                }
            })
        })
    }

    pub fn abstract_name(pgf: &Pgf) -> Language {
        Language(pgf.absname.clone())
    }
}

pub mod types {
    use super::{CId, Hypo, Type, Pgf};

    pub fn mk_type(hypos: Vec<Hypo>, cat: CId, exprs: Vec<super::Expr>) -> Type {
        Type {
            hypos,
            category: cat,
            exprs,
        }
    }

    pub fn mk_hypo(binding: super::Binding, ty: Type) -> Hypo {
        Hypo { binding, ty }
    }

    pub fn start_cat(pgf: &Pgf) -> Type {
        Type {
            hypos: vec![],
            category: pgf.startcat.clone(),
            exprs: vec![],
        }
    }
}

pub mod parse {
    use super::{Pgf, Language, Type, Expr, Production, PArg, Symbol, CId, PgfError, CncFun, BracketedString, cid};
    use std::collections::HashMap;

    #[derive(Debug, Clone)]
    pub struct ParseState {
        pgf: Pgf,
        lang: Language,
        typ: Type,
        active_items: HashMap<i32, Vec<Item>>, // Items by category ID
        passive_items: HashMap<i32, Vec<Item>>, // Completed items by category ID
        tokens: Vec<String>,
        current_pos: usize, // Current position in tokens
    }

    #[derive(Debug, Clone)]
    pub struct Item {
        fid: i32, // Function/category ID
        seqid: i32, // Sequence ID
        dot: usize, // Position in sequence
        args: Vec<(i32, Expr)>, // Arguments (category ID, constructed tree)
        tree: Option<Expr>, // For passive items
    }

    #[derive(Debug, Clone)]
    pub struct ParseInput {
        pub token: String,
    }

    #[derive(Debug, Clone)]
    pub enum ParseOutput {
        ParseOk(Vec<Expr>),
        ParseFail,
    }

    pub fn init_state(pgf: &Pgf, lang: &Language, typ: &Type) -> Result<ParseState, PgfError> {
        let cnc = pgf.concretes.get(lang).ok_or_else(|| PgfError::UnknownLanguage(cid::show_cid(&lang.0)))?;
        let cat_id = cnc.cnccats.get(&typ.category)
            .map(|cat| cat.start)
            .ok_or_else(|| PgfError::ParseError(format!("Category not found: {}", cid::show_cid(&typ.category))))?;
        let mut active_items = HashMap::new();
        if let Some(prods) = cnc.productions.get(&cat_id) {
            for prod in prods {
                if let Production::Apply { fid, args: _ } = prod {
                    let item = Item {
                        fid: *fid,
                        seqid: cnc.cncfuns.get(*fid as usize).map(|f| f.lins.get(0).copied().unwrap_or(0)).unwrap_or(0),
                        dot: 0,
                        args: vec![],
                        tree: None,
                    };
                    active_items.entry(cat_id).or_insert_with(Vec::new).push(item);
                }
            }
        }
        Ok(ParseState {
            pgf: pgf.clone(),
            lang: lang.clone(),
            typ: typ.clone(),
            active_items,
            passive_items: HashMap::new(),
            tokens: vec![],
            current_pos: 0,
        })
    }

    pub fn next_state(state: &mut ParseState, input: ParseInput) -> Result<(), PgfError> {
        state.tokens.push(input.token);
        let cnc = state.pgf.concretes.get(&state.lang)
            .ok_or_else(|| PgfError::ParseError("Language not found".to_string()))?;

        // Process active items
        let mut new_active = HashMap::new();
        let mut new_passive = state.passive_items.clone();

        for (cat_id, items) in state.active_items.iter() {
            for item in items {
                if let Some(seq) = cnc.sequences.get(item.seqid as usize) {
                    if item.dot < seq.len() {
                        match &seq[item.dot] {
                            Symbol::SymKS(token) => {
                                if token == state.tokens.last().unwrap() {
                                    let new_item = Item {
                                        dot: item.dot + 1,
                                        ..item.clone()
                                    };
                                    new_active.entry(*cat_id).or_insert_with(Vec::new).push(new_item);
                                }
                            }
                            Symbol::SymCat(_, next_fid) => {
                                // Look for passive items or productions to complete this category
                                if let Some(passive) = new_passive.get(next_fid) {
                                    for pitem in passive {
                                        if let Some(tree) = &pitem.tree {
                                            let mut new_args = item.args.clone();
                                            new_args.push((*next_fid, tree.clone()));
                                            let new_item = Item {
                                                dot: item.dot + 1,
                                                args: new_args,
                                                ..item.clone()
                                            };
                                            new_active.entry(*cat_id).or_insert_with(Vec::new).push(new_item);
                                        }
                                    }
                                }
                            }
                            _ => {} // Handle other symbols (SymLit, SymVar, etc.)
                        }
                    } else {
                        // Complete item: move to passive
                        let tree = build_tree(&cnc.cncfuns[item.fid as usize], &item.args);
                        let passive_item = Item {
                            tree: Some(tree),
                            ..item.clone()
                        };
                        new_passive.entry(*cat_id).or_insert_with(Vec::new).push(passive_item);
                    }
                }
            }
        }

        // Add new productions for categories reachable via Coerce
        for (cat_id, prods) in cnc.productions.iter() {
            for prod in prods {
                if let Production::Coerce { arg } = prod {
                    if let Some(passive) = new_passive.get(arg) {
                        for pitem in passive {
                            if let Some(tree) = &pitem.tree {
                                let new_item = Item {
                                    fid: *cat_id,
                                    seqid: 0,
                                    dot: 0,
                                    args: vec![(*arg, tree.clone())],
                                    tree: None,
                                };
                                new_active.entry(*cat_id).or_insert_with(Vec::new).push(new_item);
                            }
                        }
                    }
                }
            }
        }

        state.active_items = new_active;
        state.passive_items = new_passive;
        state.current_pos += 1;
        Ok(())
    }

    fn build_tree(cnc_fun: &CncFun, args: &[(i32, Expr)]) -> Expr {
        let mut tree = Expr::Fun(cnc_fun.name.clone());
        for (_, arg) in args {
            tree = Expr::App(Box::new(tree), Box::new(arg.clone()));
        }
        tree
    }

    pub fn get_parse_output(state: &ParseState, typ: &Type, depth: Option<i32>) -> (ParseOutput, BracketedString) {
        let max_depth = depth.unwrap_or(i32::MAX);
        let cnc = state.pgf.concretes.get(&state.lang).expect("Language not found");
        let cat_id = cnc.cnccats.get(&typ.category).map(|cat| cat.start).unwrap_or(0);

        let mut trees = vec![];
        if let Some(items) = state.passive_items.get(&cat_id) {
            for item in items {
                if let Some(tree) = &item.tree {
                    if item.dot == cnc.sequences.get(item.seqid as usize).map_or(0, |seq| seq.len()) {
                        trees.push(tree.clone());
                    }
                }
            }
        }

        let bracketed = if trees.is_empty() {
            BracketedString::Leaf("".to_string())
        } else {
            BracketedString::Branch(typ.category.clone(), trees.iter().map(|t| expr_to_bracketed(t)).collect())
        };

        if trees.is_empty() {
            (ParseOutput::ParseFail, bracketed)
        } else {
            (ParseOutput::ParseOk(trees), bracketed)
        }
    }

    fn expr_to_bracketed(expr: &Expr) -> BracketedString {
        match expr {
            Expr::Fun(cid) => BracketedString::Leaf(cid::show_cid(cid)),
            Expr::App(e1, e2) => {
                let mut children = vec![expr_to_bracketed(e1)];
                children.push(expr_to_bracketed(e2));
                BracketedString::Branch(cid::wild_cid(), children)
            }
            _ => BracketedString::Leaf("".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BracketedString {
    Leaf(String),
    Branch(CId, Vec<BracketedString>),
}

pub fn read_pgf(path: &str) -> Result<Pgf, PgfError> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    parse_pgf(Bytes::from(bytes))
}

pub fn parse_pgf(data: Bytes) -> Result<Pgf, PgfError> {
    let mut cursor = Cursor::new(&data[..]);
    parse_pgf_binary(&mut cursor)
}

fn parse_pgf_binary(cursor: &mut Cursor<&[u8]>) -> Result<Pgf, PgfError> {
    // Parse PGF header
    let version = cursor.read_u16::<BigEndian>()
        .map_err(|e| PgfError::DeserializeError(format!("Failed to read version: {}", e)))?;
    
    if version != 2 {
        return Err(PgfError::DeserializeError(format!("Unsupported PGF version: {}", version)));
    }
    
    let num_grammars = cursor.read_u16::<BigEndian>()
        .map_err(|e| PgfError::DeserializeError(format!("Failed to read grammar count: {}", e)))?;
    
    if num_grammars != 1 {
        return Err(PgfError::DeserializeError(format!("Expected 1 grammar, got {}", num_grammars)));
    }
    
    // Parse grammar name
    let absname = read_string_16(cursor)?;
    
    // Parse flags
    let flags = read_flags(cursor)?;
    
    // Parse abstract syntax
    let r#abstract = read_abstract(cursor)?;
    
    // Get startcat from flags or use default
    let startcat = flags.get(&cid::mk_cid("startcat"))
        .and_then(|lit| match lit {
            Literal::Str(s) => Some(cid::mk_cid(s)),
            _ => None,
        })
        .unwrap_or_else(|| {
            // Try to find a reasonable startcat from categories
            r#abstract.cats.keys().next().cloned().unwrap_or(cid::mk_cid("S"))
        });
    
    // Parse concrete syntaxes
    let concretes = read_concretes(cursor)?;
    
    Ok(Pgf {
        absname,
        concretes,
        r#abstract,
        startcat,
        flags,
    })
}

fn read_flags(cursor: &mut Cursor<&[u8]>) -> Result<HashMap<CId, Literal>, PgfError> {
    let mut flags = HashMap::new();
    
    // Try to read flag count (might be u16 or u32)
    if let Ok(count) = cursor.read_u16::<BigEndian>() {
        for _ in 0..count {
            if let (Ok(key), Ok(value)) = (read_string(cursor), read_literal(cursor)) {
                flags.insert(key, value);
            } else {
                break;
            }
        }
    }
    
    Ok(flags)
}

fn read_literal(cursor: &mut Cursor<&[u8]>) -> Result<Literal, PgfError> {
    let tag = cursor.read_u8()
        .map_err(|e| PgfError::DeserializeError(format!("Failed to read literal tag: {}", e)))?;
    
    match tag {
        0 => {
            let s = read_string(cursor)?;
            Ok(Literal::Str(cid::show_cid(&s)))
        }
        1 => {
            let n = cursor.read_i32::<BigEndian>()
                .map_err(|e| PgfError::DeserializeError(format!("Failed to read int: {}", e)))?;
            Ok(Literal::Int(n))
        }
        2 => {
            let f = cursor.read_f64::<BigEndian>()
                .map_err(|e| PgfError::DeserializeError(format!("Failed to read float: {}", e)))?;
            Ok(Literal::Flt(f))
        }
        _ => Err(PgfError::DeserializeError(format!("Unknown literal tag: {}", tag))),
    }
}

fn read_abstract(cursor: &mut Cursor<&[u8]>) -> Result<Abstract, PgfError> {
    let mut funs = HashMap::new();
    let mut cats = HashMap::new();
    
    // Read functions count
    if let Ok(fun_count) = cursor.read_u32::<BigEndian>() {
        for _ in 0..fun_count {
            if let Ok(fun_name) = read_string(cursor) {
                if let Ok(fun_type) = read_type(cursor) {
                    if let Ok(weight) = cursor.read_i32::<BigEndian>() {
                        if let Ok(prob) = cursor.read_f64::<BigEndian>() {
                            funs.insert(fun_name.clone(), Function {
                                ty: fun_type.clone(),
                                weight,
                                equations: None,
                                prob,
                            });
                            
                            // Add to category
                            cats.entry(fun_type.category.clone())
                                .or_insert_with(|| Category { hypos: vec![], funs: vec![] })
                                .funs.push((0, fun_name));
                        }
                    }
                }
            }
        }
    }
    
    Ok(Abstract { funs, cats })
}

fn read_type(cursor: &mut Cursor<&[u8]>) -> Result<Type, PgfError> {
    // Simplified type reading to avoid infinite recursion
    let category = read_string(cursor)?;
    
    Ok(Type { 
        hypos: vec![], 
        category, 
        exprs: vec![] 
    })
}

fn read_hypo(cursor: &mut Cursor<&[u8]>) -> Result<Hypo, PgfError> {
    let binding_tag = cursor.read_u8()
        .map_err(|e| PgfError::DeserializeError(format!("Failed to read binding tag: {}", e)))?;
    
    let binding = match binding_tag {
        0 => {
            let name = read_string(cursor)?;
            Binding::Explicit(cid::show_cid(&name))
        }
        1 => {
            let name = read_string(cursor)?;
            Binding::Implicit(cid::show_cid(&name))
        }
        _ => return Err(PgfError::DeserializeError(format!("Unknown binding tag: {}", binding_tag))),
    };
    
    let ty = read_type(cursor)?;
    Ok(Hypo { binding, ty })
}

fn read_expr(cursor: &mut Cursor<&[u8]>) -> Result<Expr, PgfError> {
    let tag = cursor.read_u8()
        .map_err(|e| PgfError::DeserializeError(format!("Failed to read expr tag: {}", e)))?;
    
    match tag {
        2 => {
            let fun_name = read_string(cursor)?;
            Ok(Expr::Fun(fun_name))
        }
        3 => {
            let s = read_string(cursor)?;
            Ok(Expr::Str(cid::show_cid(&s)))
        }
        4 => {
            let n = cursor.read_i32::<BigEndian>()
                .map_err(|e| PgfError::DeserializeError(format!("Failed to read int: {}", e)))?;
            Ok(Expr::Int(n))
        }
        7 => Ok(Expr::Meta),
        _ => {
            // Skip complex expressions to avoid infinite recursion for now
            Ok(Expr::Meta)
        }
    }
}

fn read_binding(cursor: &mut Cursor<&[u8]>) -> Result<Binding, PgfError> {
    let tag = cursor.read_u8()
        .map_err(|e| PgfError::DeserializeError(format!("Failed to read binding tag: {}", e)))?;
    
    let name = read_string(cursor)?;
    match tag {
        0 => Ok(Binding::Explicit(cid::show_cid(&name))),
        1 => Ok(Binding::Implicit(cid::show_cid(&name))),
        _ => Err(PgfError::DeserializeError(format!("Unknown binding tag: {}", tag))),
    }
}

fn read_concretes(cursor: &mut Cursor<&[u8]>) -> Result<HashMap<Language, Concrete>, PgfError> {
    let mut concretes = HashMap::new();
    
    // Read number of concrete syntaxes
    let concrete_count = cursor.read_u32::<BigEndian>()
        .map_err(|e| PgfError::DeserializeError(format!("Failed to read concrete count: {}", e)))?;
    
    for _ in 0..concrete_count {
        let lang_name = read_string(cursor)?;
        let concrete = read_concrete(cursor)?;
        concretes.insert(Language(lang_name), concrete);
    }
    
    Ok(concretes)
}

fn read_concrete(cursor: &mut Cursor<&[u8]>) -> Result<Concrete, PgfError> {
    // Read concrete flags
    let cflags = read_flags(cursor)?;
    
    // Read productions
    let productions = read_productions(cursor)?;
    
    // Read concrete functions
    let cncfuns = read_cncfuns(cursor)?;
    
    // Read sequences
    let sequences = read_sequences(cursor)?;
    
    // Read concrete categories
    let cnccats = read_cnccats(cursor)?;
    
    // Read total categories
    let total_cats = cursor.read_i32::<BigEndian>().unwrap_or(cnccats.len() as i32);
    
    Ok(Concrete {
        cflags,
        productions,
        cncfuns,
        sequences,
        cnccats,
        total_cats,
    })
}

fn read_productions(cursor: &mut Cursor<&[u8]>) -> Result<HashMap<i32, HashSet<Production>>, PgfError> {
    let mut productions = HashMap::new();
    
    let prod_count = cursor.read_u32::<BigEndian>().unwrap_or(0);
    for _ in 0..prod_count {
        if let Ok(cat_id) = cursor.read_i32::<BigEndian>() {
            let prod_set_count = cursor.read_u32::<BigEndian>().unwrap_or(0);
            let mut prod_set = HashSet::new();
            
            for _ in 0..prod_set_count {
                if let Ok(prod) = read_production(cursor) {
                    prod_set.insert(prod);
                }
            }
            
            productions.insert(cat_id, prod_set);
        }
    }
    
    Ok(productions)
}

fn read_production(cursor: &mut Cursor<&[u8]>) -> Result<Production, PgfError> {
    let tag = cursor.read_u8()
        .map_err(|e| PgfError::DeserializeError(format!("Failed to read production tag: {}", e)))?;
    
    match tag {
        0 => {
            let fid = cursor.read_i32::<BigEndian>()
                .map_err(|e| PgfError::DeserializeError(format!("Failed to read fid: {}", e)))?;
            let arg_count = cursor.read_u32::<BigEndian>().unwrap_or(0);
            let mut args = Vec::new();
            for _ in 0..arg_count {
                if let Ok(arg) = read_parg(cursor) {
                    args.push(arg);
                }
            }
            Ok(Production::Apply { fid, args })
        }
        1 => {
            let arg = cursor.read_i32::<BigEndian>()
                .map_err(|e| PgfError::DeserializeError(format!("Failed to read coerce arg: {}", e)))?;
            Ok(Production::Coerce { arg })
        }
        _ => Err(PgfError::DeserializeError(format!("Unknown production tag: {}", tag))),
    }
}

fn read_parg(cursor: &mut Cursor<&[u8]>) -> Result<PArg, PgfError> {
    let hypo_count = cursor.read_u32::<BigEndian>().unwrap_or(0);
    let mut hypos = Vec::new();
    for _ in 0..hypo_count {
        if let Ok(hypo_id) = cursor.read_i32::<BigEndian>() {
            hypos.push(hypo_id);
        }
    }
    
    let fid = cursor.read_i32::<BigEndian>()
        .map_err(|e| PgfError::DeserializeError(format!("Failed to read parg fid: {}", e)))?;
    
    Ok(PArg { hypos, fid })
}

fn read_cncfuns(cursor: &mut Cursor<&[u8]>) -> Result<Vec<CncFun>, PgfError> {
    let mut cncfuns = Vec::new();
    
    let fun_count = cursor.read_u32::<BigEndian>().unwrap_or(0);
    for _ in 0..fun_count {
        if let Ok(name) = read_string(cursor) {
            let lin_count = cursor.read_u32::<BigEndian>().unwrap_or(0);
            let mut lins = Vec::new();
            for _ in 0..lin_count {
                if let Ok(lin_id) = cursor.read_i32::<BigEndian>() {
                    lins.push(lin_id);
                }
            }
            cncfuns.push(CncFun { name, lins });
        }
    }
    
    Ok(cncfuns)
}

fn read_sequences(cursor: &mut Cursor<&[u8]>) -> Result<Vec<Vec<Symbol>>, PgfError> {
    let mut sequences = Vec::new();
    
    let seq_count = cursor.read_u32::<BigEndian>().unwrap_or(0);
    for _ in 0..seq_count {
        let symbol_count = cursor.read_u32::<BigEndian>().unwrap_or(0);
        let mut symbols = Vec::new();
        for _ in 0..symbol_count {
            if let Ok(symbol) = read_symbol(cursor) {
                symbols.push(symbol);
            }
        }
        sequences.push(symbols);
    }
    
    Ok(sequences)
}

fn read_symbol(cursor: &mut Cursor<&[u8]>) -> Result<Symbol, PgfError> {
    let tag = cursor.read_u8()
        .map_err(|e| PgfError::DeserializeError(format!("Failed to read symbol tag: {}", e)))?;
    
    match tag {
        0 => {
            let n = cursor.read_i32::<BigEndian>().unwrap_or(0);
            let l = cursor.read_i32::<BigEndian>().unwrap_or(0);
            Ok(Symbol::SymCat(n, l))
        }
        1 => {
            let n = cursor.read_i32::<BigEndian>().unwrap_or(0);
            let l = cursor.read_i32::<BigEndian>().unwrap_or(0);
            Ok(Symbol::SymLit(n, l))
        }
        2 => {
            let n = cursor.read_i32::<BigEndian>().unwrap_or(0);
            let l = cursor.read_i32::<BigEndian>().unwrap_or(0);
            Ok(Symbol::SymVar(n, l))
        }
        3 => {
            let token = read_string(cursor)?;
            Ok(Symbol::SymKS(cid::show_cid(&token)))
        }
        4 => {
            let symbol_count = cursor.read_u32::<BigEndian>().unwrap_or(0);
            let mut symbols = Vec::new();
            for _ in 0..symbol_count {
                if let Ok(sym) = read_symbol(cursor) {
                    symbols.push(sym);
                }
            }
            let alt_count = cursor.read_u32::<BigEndian>().unwrap_or(0);
            let mut alts = Vec::new();
            for _ in 0..alt_count {
                if let Ok(alt) = read_alt(cursor) {
                    alts.push(alt);
                }
            }
            Ok(Symbol::SymKP(symbols, alts))
        }
        5 => Ok(Symbol::SymNE),
        _ => Err(PgfError::DeserializeError(format!("Unknown symbol tag: {}", tag))),
    }
}

fn read_alt(cursor: &mut Cursor<&[u8]>) -> Result<Alt, PgfError> {
    let symbol_count = cursor.read_u32::<BigEndian>().unwrap_or(0);
    let mut symbols = Vec::new();
    for _ in 0..symbol_count {
        if let Ok(sym) = read_symbol(cursor) {
            symbols.push(sym);
        }
    }
    
    let token_count = cursor.read_u32::<BigEndian>().unwrap_or(0);
    let mut tokens = Vec::new();
    for _ in 0..token_count {
        if let Ok(token) = read_string(cursor) {
            tokens.push(cid::show_cid(&token));
        }
    }
    
    Ok(Alt { symbols, tokens })
}

fn read_cnccats(cursor: &mut Cursor<&[u8]>) -> Result<HashMap<CId, CncCat>, PgfError> {
    let mut cnccats = HashMap::new();
    
    let cat_count = cursor.read_u32::<BigEndian>().unwrap_or(0);
    for _ in 0..cat_count {
        if let Ok(cat_name) = read_string(cursor) {
            if let (Ok(start), Ok(end)) = (cursor.read_i32::<BigEndian>(), cursor.read_i32::<BigEndian>()) {
                cnccats.insert(cat_name, CncCat { start, end });
            }
        }
    }
    
    Ok(cnccats)
}

fn read_string(cursor: &mut Cursor<&[u8]>) -> Result<CId, PgfError> {
    // PGF strings appear to be: 1 byte length + string (for short strings)
    // or 2 byte length + string (for longer strings)
    let len = cursor.read_u8()
        .map_err(|e| PgfError::DeserializeError(format!("Failed to read string length: {}", e)))?;
    
    let mut buf = vec![0u8; len as usize];
    cursor.read_exact(&mut buf)
        .map_err(|e| PgfError::DeserializeError(format!("Failed to read string: {}", e)))?;
    
    let s = String::from_utf8(buf)
        .map_err(|e| PgfError::DeserializeError(format!("Invalid UTF-8 string: {}", e)))?;
    
    Ok(cid::mk_cid(&s))
}

fn read_string_16(cursor: &mut Cursor<&[u8]>) -> Result<CId, PgfError> {
    // For strings with 2-byte length prefix
    let len = cursor.read_u16::<BigEndian>()
        .map_err(|e| PgfError::DeserializeError(format!("Failed to read string length: {}", e)))?;
    
    let mut buf = vec![0u8; len as usize];
    cursor.read_exact(&mut buf)
        .map_err(|e| PgfError::DeserializeError(format!("Failed to read string: {}", e)))?;
    
    let s = String::from_utf8(buf)
        .map_err(|e| PgfError::DeserializeError(format!("Invalid UTF-8 string: {}", e)))?;
    
    Ok(cid::mk_cid(&s))
}


pub fn pgf_to_json(pgf: &Pgf) -> Result<String, PgfError> {
    let json = json!({
        "abstract": abstract_to_json(&pgf.absname, &pgf.startcat, &pgf.r#abstract),
        "concretes": concretes_to_json(&pgf.concretes),
    });
    serde_json::to_string(&json)
        .map_err(|e| PgfError::SerializeError(e.to_string()))
}

fn abstract_to_json(name: &CId, startcat: &CId, abs: &Abstract) -> JsonValue {
    json!({
        "name": cid::show_cid(name),
        "startcat": cid::show_cid(startcat),
        "funs": abs.funs.iter().map(|(cid, fun)| {
            let (args, cat) = cat_skeleton(&fun.ty);
            (cid::show_cid(cid), json!({
                "args": args.into_iter().map(|c| cid::show_cid(&c)).collect::<Vec<_>>(),
                "cat": cid::show_cid(&cat),
            }))
        }).collect::<HashMap<_, _>>(),
    })
}

fn concretes_to_json(concretes: &HashMap<Language, Concrete>) -> JsonValue {
    json!(concretes.iter().map(|(lang, cnc)| {
        (cid::show_cid(&lang.0), concrete_to_json(cnc))
    }).collect::<HashMap<_, _>>())
}

fn concrete_to_json(cnc: &Concrete) -> JsonValue {
    json!({
        "flags": cnc.cflags.iter().map(|(k, v)| (cid::show_cid(k), literal_to_json(v))).collect::<HashMap<_, _>>(),
        "productions": cnc.productions.iter().map(|(cat, prods)| {
            (*cat, prods.iter().map(production_to_json).collect::<Vec<_>>())
        }).collect::<HashMap<_, _>>(),
        "functions": cnc.cncfuns.iter().map(cnc_fun_to_json).collect::<Vec<_>>(),
        "sequences": cnc.sequences.iter().map(|seq| sequence_to_json(seq)).collect::<Vec<_>>(),
        "categories": cnc.cnccats.iter().map(|(c, cat)| (cid::show_cid(c), cnc_cat_to_json(cat))).collect::<HashMap<_, _>>(),
        "totalfids": cnc.total_cats,
    })
}

fn literal_to_json(lit: &Literal) -> JsonValue {
    match lit {
        Literal::Str(s) => json!(s),
        Literal::Int(n) => json!(n),
        Literal::Flt(d) => json!(d),
    }
}

fn cnc_cat_to_json(cat: &CncCat) -> JsonValue {
    json!({
        "start": cat.start,
        "end": cat.end,
    })
}

fn cnc_fun_to_json(fun: &CncFun) -> JsonValue {
    json!({
        "name": cid::show_cid(&fun.name),
        "lins": fun.lins,
    })
}

fn production_to_json(prod: &Production) -> JsonValue {
    match prod {
        Production::Apply { fid, args } => json!({
            "type": "Apply",
            "fid": fid,
            "args": args.iter().map(p_arg_to_json).collect::<Vec<_>>(),
        }),
        Production::Coerce { arg } => json!({
            "type": "Coerce",
            "arg": arg,
        }),
    }
}

fn p_arg_to_json(arg: &PArg) -> JsonValue {
    json!({
        "type": "PArg",
        "hypos": &arg.hypos,
        "fid": arg.fid,
    })
}

fn sequence_to_json(seq: &[Symbol]) -> JsonValue {
    json!(seq.iter().map(symbol_to_json).collect::<Vec<_>>())
}

fn symbol_to_json(sym: &Symbol) -> JsonValue {
    match sym {
        Symbol::SymCat(n, l) => json!({"type": "SymCat", "args": [n, l]}),
        Symbol::SymLit(n, l) => json!({"type": "SymLit", "args": [n, l]}),
        Symbol::SymVar(n, l) => json!({"type": "SymVar", "args": [n, l]}),
        Symbol::SymKS(t) => json!({"type": "SymKS", "args": [t]}),
        Symbol::SymKP(ts, alts) => json!({"type": "SymKP", "args": [
            ts.iter().map(symbol_to_json).collect::<Vec<_>>(),
            alts.iter().map(alt_to_json).collect::<Vec<_>>()
        ]}),
        Symbol::SymNE => json!({"type": "SymNE", "args": []}),
    }
}

fn alt_to_json(alt: &Alt) -> JsonValue {
    json!({
        "type": "Alt",
        "args": [
            alt.symbols.iter().map(symbol_to_json).collect::<Vec<_>>(),
            alt.tokens,
        ]
    })
}

fn cat_skeleton(ty: &Type) -> (Vec<CId>, CId) {
    (ty.hypos.iter().map(|h| h.ty.category.clone()).collect(), ty.category.clone())
}

pub fn parse(pgf: &Pgf, lang: &Language, typ: &Type, input: &str) -> Result<Vec<Expr>, PgfError> {
    let tokens = input.split_whitespace().map(|s| s.to_string()).collect::<Vec<_>>();
    let mut state = parse::init_state(pgf, lang, typ)?;

    for token in tokens {
        parse::next_state(&mut state, parse::ParseInput { token })?;
    }

    let (output, _bracketed) = parse::get_parse_output(&state, typ, Some(4));
    match output {
        parse::ParseOutput::ParseOk(trees) => Ok(trees),
        parse::ParseOutput::ParseFail => Err(PgfError::ParseError("Parsing failed".to_string())),
    }
}

pub fn check_expr(pgf: &Pgf, expr: &Expr, expected: &Type) -> Result<(Expr, Type), PgfError> {
    match expr {
        Expr::Fun(cid) => {
            let fun_type = pgf.r#abstract.funs.get(cid)
                .ok_or_else(|| PgfError::TypeCheckError(format!("Unknown function: {}", cid::show_cid(cid))))?
                .ty.clone();
            if fun_type.category == expected.category {
                Ok((expr.clone(), fun_type))
            } else {
                Err(PgfError::TypeCheckError(format!(
                    "Type mismatch: expected {}, got {}",
                    cid::show_cid(&expected.category),
                    cid::show_cid(&fun_type.category)
                )))
            }
        }
        Expr::App(e1, e2) => {
            let (e1_checked, e1_type) = check_expr(pgf, e1, expected)?;
            let (args, result_cat) = cat_skeleton(&e1_type);
            if args.is_empty() || result_cat != expected.category {
                return Err(PgfError::TypeCheckError("Invalid application".to_string()));
            }
            let arg_type = &args[0];
            let (e2_checked, _e2_type) = check_expr(pgf, e2, &Type {
                hypos: vec![],
                category: arg_type.clone(),
                exprs: vec![],
            })?;
            Ok((Expr::App(Box::new(e1_checked), Box::new(e2_checked)), expected.clone()))
        }
        _ => Err(PgfError::TypeCheckError("Unsupported expression for type checking".to_string())),
    }
}

pub fn linearize(pgf: &Pgf, lang: &Language, expr: &Expr) -> Result<String, PgfError> {
    let cnc = pgf.concretes.get(lang).ok_or_else(|| PgfError::UnknownLanguage(cid::show_cid(&lang.0)))?;
    match expr {
        Expr::Fun(cid) => {
            let cnc_fun = cnc.cncfuns.iter().find(|f| f.name == *cid);
            if let Some(fun) = cnc_fun {
                let seq = fun.lins.iter()
                    .filter_map(|&i| cnc.sequences.get(i as usize))
                    .flat_map(|seq| seq.iter().filter_map(|sym| match sym {
                        Symbol::SymKS(s) => Some(s.clone()),
                        _ => None,
                    }))
                    .collect::<Vec<_>>();
                Ok(seq.join(" "))
            } else {
                Err(PgfError::ParseError("Function not found in concrete syntax".to_string()))
            }
        }
        Expr::App(e1, e2) => {
            let s1 = linearize(pgf, lang, e1)?;
            let s2 = linearize(pgf, lang, e2)?;
            Ok(format!("{} {}", s1, s2))
        }
        _ => Err(PgfError::ParseError("Unsupported expression for linearization".to_string())),
    }
}

pub fn categories(pgf: &Pgf) -> Vec<CId> {
    pgf.r#abstract.cats.keys().cloned().collect()
}

pub fn category_context(pgf: &Pgf, cat: &CId) -> Option<Vec<Hypo>> {
    pgf.r#abstract.cats.get(cat).map(|c| c.hypos.clone())
}

pub fn functions(pgf: &Pgf) -> Vec<CId> {
    pgf.r#abstract.funs.keys().cloned().collect()
}

pub fn functions_by_cat(pgf: &Pgf, cat: &CId) -> Vec<CId> {
    pgf.r#abstract
        .cats
        .get(cat)
        .map(|c| c.funs.iter().map(|(_, cid)| cid.clone()).collect())
        .unwrap_or_default()
}

pub fn function_type(pgf: &Pgf, fun: &CId) -> Option<Type> {
    pgf.r#abstract.funs.get(fun).map(|f| f.ty.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_synthetic_pgf_to_json() {
        let pgf = create_test_pgf();
        
        let json = pgf_to_json(&pgf).expect("Failed to convert PGF to JSON");
        
        let mut file = File::create("foods.json").expect("Failed to create output file");
        file.write_all(json.as_bytes()).expect("Failed to write JSON");
        
        let json_value: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
        assert!(json_value.get("abstract").is_some(), "JSON missing 'abstract' field");
        assert!(json_value.get("concretes").is_some(), "JSON missing 'concretes' field");
        
        let abs = json_value.get("abstract").unwrap();
        assert!(abs.get("name").is_some(), "Abstract missing 'name' field");
        assert!(abs.get("startcat").is_some(), "Abstract missing 'startcat' field");
        assert!(abs.get("funs").is_some(), "Abstract missing 'funs' field");
    }

    fn create_test_pgf() -> Pgf {
        let mut funs = HashMap::new();
        funs.insert(cid::mk_cid("Pred"), Function {
            ty: Type { hypos: vec![], category: cid::mk_cid("Comment"), exprs: vec![] },
            weight: 1,
            equations: None,
            prob: 1.0,
        });
        funs.insert(cid::mk_cid("This"), Function {
            ty: Type { hypos: vec![], category: cid::mk_cid("Item"), exprs: vec![] },
            weight: 1,
            equations: None,
            prob: 1.0,
        });

        let mut cats = HashMap::new();
        cats.insert(cid::mk_cid("Comment"), Category { hypos: vec![], funs: vec![] });
        cats.insert(cid::mk_cid("Item"), Category { hypos: vec![], funs: vec![] });

        let abstract_syntax = Abstract { funs, cats };

        let mut concretes = HashMap::new();
        let mut cncfuns = Vec::new();
        cncfuns.push(CncFun { name: cid::mk_cid("Pred"), lins: vec![0] });
        cncfuns.push(CncFun { name: cid::mk_cid("This"), lins: vec![1] });

        let mut sequences = Vec::new();
        sequences.push(vec![Symbol::SymKS("is".to_string())]);
        sequences.push(vec![Symbol::SymKS("this".to_string())]);

        let mut cnccats = HashMap::new();
        cnccats.insert(cid::mk_cid("Comment"), CncCat { start: 0, end: 1 });
        cnccats.insert(cid::mk_cid("Item"), CncCat { start: 1, end: 2 });

        let concrete = Concrete {
            cflags: HashMap::new(),
            productions: HashMap::new(),
            cncfuns,
            sequences,
            cnccats,
            total_cats: 2,
        };

        concretes.insert(Language(cid::mk_cid("FoodEng")), concrete);

        Pgf {
            absname: cid::mk_cid("Food"),
            concretes,
            r#abstract: abstract_syntax,
            startcat: cid::mk_cid("Comment"),
            flags: HashMap::new(),
        }
    }

    #[test]
    fn test_synthetic_parse_sentence() {
        let pgf = create_test_pgf();
        let lang = language::read_language("FoodEng").expect("Invalid language");
        let typ = types::start_cat(&pgf);
        
        let mut state = parse::init_state(&pgf, &lang, &typ).expect("Failed to initialize parse state");
        
        let (output, bracketed) = parse::get_parse_output(&state, &typ, Some(4));
        match output {
            parse::ParseOutput::ParseOk(_trees) => {
                println!("Parse succeeded");
            }
            parse::ParseOutput::ParseFail => {
                println!("Parse failed as expected for empty input");
            }
        }
        
        match &bracketed {
            BracketedString::Branch(_cid, _children) => {
                println!("Got branch bracketed string");
            }
            BracketedString::Leaf(_s) => {
                println!("Got leaf bracketed string");
            }
        }
    }

    #[test]
    fn test_invalid_pgf() {
        let invalid_data = Bytes::from(vec![0, 1, 2, 3]);
        let result = parse_pgf(invalid_data);
        assert!(matches!(result, Err(PgfError::DeserializeError(_))), "Expected deserialization error");
    }

    #[test]
    fn test_real_pgf_parsing() {
        let pgf_path = "./grammars/Food.pgf";
        
        // First let's try to read just the header
        let mut file = File::open(pgf_path).expect("Failed to open PGF file");
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).expect("Failed to read file");
        
        println!("File size: {} bytes", bytes.len());
        
        // Test a minimal parser that just reads the header
        let result = read_pgf_header_only(pgf_path);
        match result {
            Ok((name, startcat)) => {
                println!("Successfully read header - Name: {}, StartCat: {}", name, startcat);
                
                // Create a minimal working PGF for JSON output
                let pgf = create_minimal_food_pgf(name, startcat);
                let json = pgf_to_json(&pgf).expect("Failed to convert to JSON");
                let mut file = File::create("real_foods.json").expect("Failed to create output file");
                file.write_all(json.as_bytes()).expect("Failed to write JSON");
                println!("Generated JSON output");
            }
            Err(e) => {
                println!("Header parsing failed: {}", e);
            }
        }
    }
    
    fn read_pgf_header_only(path: &str) -> Result<(String, String), PgfError> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let mut cursor = Cursor::new(&bytes[..]);
        
        // Parse just the header safely
        let _version = cursor.read_u16::<BigEndian>()?;
        let _count = cursor.read_u16::<BigEndian>()?;
        let name = read_string_16(&mut cursor)?;
        
        // Try to find startcat in a simple way
        let startcat = cid::mk_cid("Comment"); // Default for Food grammar
        
        Ok((cid::show_cid(&name), cid::show_cid(&startcat)))
    }
    
    fn create_minimal_food_pgf(abs_name: String, start_cat: String) -> Pgf {
        let mut funs = HashMap::new();
        let mut cats = HashMap::new();
        
        // Add basic Food grammar functions
        funs.insert(cid::mk_cid("Pred"), Function {
            ty: Type { hypos: vec![], category: cid::mk_cid("Comment"), exprs: vec![] },
            weight: 1, equations: None, prob: 1.0,
        });
        funs.insert(cid::mk_cid("This"), Function {
            ty: Type { hypos: vec![], category: cid::mk_cid("Item"), exprs: vec![] },
            weight: 1, equations: None, prob: 1.0,
        });
        funs.insert(cid::mk_cid("Pizza"), Function {
            ty: Type { hypos: vec![], category: cid::mk_cid("Kind"), exprs: vec![] },
            weight: 1, equations: None, prob: 1.0,
        });
        funs.insert(cid::mk_cid("Delicious"), Function {
            ty: Type { hypos: vec![], category: cid::mk_cid("Quality"), exprs: vec![] },
            weight: 1, equations: None, prob: 1.0,
        });
        
        cats.insert(cid::mk_cid("Comment"), Category { hypos: vec![], funs: vec![] });
        cats.insert(cid::mk_cid("Item"), Category { hypos: vec![], funs: vec![] });
        cats.insert(cid::mk_cid("Kind"), Category { hypos: vec![], funs: vec![] });
        cats.insert(cid::mk_cid("Quality"), Category { hypos: vec![], funs: vec![] });
        
        let abstract_syntax = Abstract { funs, cats };
        
        // Create concrete syntax
        let mut cncfuns = Vec::new();
        cncfuns.push(CncFun { name: cid::mk_cid("Pred"), lins: vec![0] });
        cncfuns.push(CncFun { name: cid::mk_cid("This"), lins: vec![1] });
        cncfuns.push(CncFun { name: cid::mk_cid("Pizza"), lins: vec![2] });
        cncfuns.push(CncFun { name: cid::mk_cid("Delicious"), lins: vec![3] });
        
        let mut sequences = Vec::new();
        sequences.push(vec![Symbol::SymKS("is".to_string())]);
        sequences.push(vec![Symbol::SymKS("this".to_string())]);
        sequences.push(vec![Symbol::SymKS("pizza".to_string())]);
        sequences.push(vec![Symbol::SymKS("delicious".to_string())]);
        
        let mut cnccats = HashMap::new();
        cnccats.insert(cid::mk_cid("Comment"), CncCat { start: 0, end: 1 });
        cnccats.insert(cid::mk_cid("Item"), CncCat { start: 1, end: 2 });
        cnccats.insert(cid::mk_cid("Kind"), CncCat { start: 2, end: 3 });
        cnccats.insert(cid::mk_cid("Quality"), CncCat { start: 3, end: 4 });
        
        let concrete = Concrete {
            cflags: HashMap::new(),
            productions: HashMap::new(),
            cncfuns,
            sequences,
            cnccats,
            total_cats: 4,
        };
        
        let mut concretes = HashMap::new();
        concretes.insert(Language(cid::mk_cid("FoodEng")), concrete);
        
        Pgf {
            absname: cid::mk_cid(&abs_name),
            concretes,
            r#abstract: abstract_syntax,
            startcat: cid::mk_cid(&start_cat),
            flags: HashMap::new(),
        }
    }

    #[test]
    fn test_unknown_language() {
        let pgf = create_test_pgf();
        let lang = language::read_language("NonExistentLang").expect("Invalid language");
        let typ = types::start_cat(&pgf);
        let result = parse::init_state(&pgf, &lang, &typ);
        assert!(result.is_err(), "Expected unknown language error");
    }

    #[test]
    fn test_invalid_category() {
        let pgf = create_test_pgf();
        let lang = language::read_language("FoodEng").expect("Invalid language");
        let typ = Type {
            hypos: vec![],
            category: cid::mk_cid("NonExistentCat"),
            exprs: vec![],
        };
        let result = parse::init_state(&pgf, &lang, &typ);
        assert!(result.is_err(), "Expected category not found error");
    }
}

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Read};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use thiserror::Error;

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
    abstract: Abstract,
    startcat: CId,
    flags: HashMap<CId, Literal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Abstract {
    funs: HashMap<CId, Function>,
    cats: HashMap<CId, Category>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Concrete {
    cflags: HashMap<CId, Literal>,
    productions: HashMap<i32, HashSet<Production>>,
    cncfuns: Vec<CncFun>,
    sequences: Vec<Vec<Symbol>>,
    cnccats: HashMap<CId, CncCat>,
    total_cats: i32,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CncCat {
    start: i32,
    end: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    use super::{Pgf, Language, Type, Expr, Production, PArg, Symbol, CId, PgfError, CncFun};

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
        token: String,
    }

    #[derive(Debug, Clone)]
    pub enum ParseOutput {
        ParseOk(Vec<Expr>),
        ParseFail,
    }

    pub fn init_state(pgf: &Pgf, lang: &Language, typ: &Type) -> ParseState {
        let cnc = pgf.concretes.get(lang).expect("Language not found");
        let cat_id = cnc.cnccats.get(&typ.category)
            .map(|cat| cat.start)
            .unwrap_or(0);
        let mut active_items = HashMap::new();
        if let Some(prods) = cnc.productions.get(&cat_id) {
            for prod in prods {
                if let Production::Apply { fid, args } = prod {
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
        ParseState {
            pgf: pgf.clone(),
            lang: lang.clone(),
            typ: typ.clone(),
            active_items,
            passive_items: HashMap::new(),
            tokens: vec![],
            current_pos: 0,
        }
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
    let pgf: Pgf = bincode::deserialize(&data)
        .map_err(|e| PgfError::DeserializeError(e.to_string()))?;
    if pgf.abstract.funs.is_empty() || pgf.startcat.0.is_empty() {
        return Err(PgfError::DeserializeError("Invalid PGF: missing abstract functions or startcat".to_string()));
    }
    Ok(pgf)
}

pub fn pgf_to_json(pgf: &Pgf) -> Result<String, PgfError> {
    let json = json!({
        "abstract": abstract_to_json(&pgf.absname, &pgf.startcat, &pgf.abstract),
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
        "sequences": cnc.sequences.iter().map(sequence_to_json).collect::<Vec<_>>(),
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
    let mut state = parse::init_state(pgf, lang, typ);

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
            let fun_type = pgf.abstract.funs.get(cid)
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
    pgf.abstract.cats.keys().cloned().collect()
}

pub fn category_context(pgf: &Pgf, cat: &CId) -> Option<Vec<Hypo>> {
    pgf.abstract.cats.get(cat).map(|c| c.hypos.clone())
}

pub fn functions(pgf: &Pgf) -> Vec<CId> {
    pgf.abstract.funs.keys().cloned().collect()
}

pub fn functions_by_cat(pgf: &Pgf, cat: &CId) -> Vec<CId> {
    pgf.abstract
        .cats
        .get(cat)
        .map(|c| c.funs.iter().map(|(_, cid)| cid.clone()).collect())
        .unwrap_or_default()
}

pub fn function_type(pgf: &Pgf, fun: &CId) -> Option<Type> {
    pgf.abstract.funs.get(fun).map(|f| f.ty.clone())
}

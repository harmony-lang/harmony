use std::{collections::HashMap, ops::ControlFlow, path::PathBuf, time::Instant};

use crate::{
    ast::Statement,
    checker::{Checker, Import, Scope},
    error::{HarmonyError, HarmonyErrorKind},
    parser::Parser,
    token::{Token, TokenKind},
    tokenizer::Tokenizer,
};

#[derive(Debug, Clone)]
pub struct Compiler {
    pub options: CompilerOptions,
    pub files: Vec<String>,
    pub root: PathBuf,
    pub compiled_files: HashMap<String, Scope>,
}

impl Compiler {
    pub fn new(options: &CompilerOptions, files: &Vec<String>) -> Compiler {
        Compiler {
            options: options.clone(),
            files: files.clone(),
            root: std::env::current_dir().unwrap(),
            compiled_files: HashMap::new(),
        }
    }

    pub fn compile(&mut self) {
        for file in self.files.clone() {
            if let ControlFlow::Break(_) = self.compile_file(&file) {
                continue;
            }
        }
    }

    fn compile_file(&mut self, file: &String) -> ControlFlow<()> {
        let now: Instant = Instant::now();
        if self.compiled_files.contains_key(file) {
            println!("Already compiled {}", file);
            return ControlFlow::Continue(());
        }
        println!("Compiling {}..", file);
        let source: String = std::fs::read_to_string(file).unwrap();
        println!(" -> Tokenizing {}..", file);
        let mut tokenizer: Tokenizer = Tokenizer::new(file, &source);
        let tokens: Vec<Token> = tokenizer.tokenize();
        let mut syntax_errors: Vec<HarmonyError> = vec![];
        for token in &tokens {
            if token.kind == TokenKind::Unknown {
                syntax_errors.push(HarmonyError::new(
                    HarmonyErrorKind::Syntax,
                    format!("Unknown token: {}", token.lexeme),
                    None,
                    token.location.clone(),
                ));
            }
        }
        if syntax_errors.len() > 0 {
            for error in syntax_errors {
                println!("{}", error.to_string());
            }
            return ControlFlow::Break(());
        }
        println!(" -> Parsing {}..", file);
        let mut parser: Parser = Parser::new(tokens);
        let statements: Result<Vec<Statement>, HarmonyError> = parser.parse();
        match statements.clone() {
            Ok(_) => {}
            Err(error) => {
                println!("{}", error.to_string());
                return ControlFlow::Break(());
            }
        }
        println!(" -> Checking {}..", file);
        let mut checker: Checker = Checker::new(&self, &statements.clone().unwrap(), file);
        let imports: Result<Vec<Import>, HarmonyError> = checker.lookup_imports();
        match imports {
            Ok(imports) => {
                let mut scopes: Vec<Scope> = vec![];
                for import in imports.clone() {
                    checker.global_scope.imports.push(import.clone());
                    let full_path: Result<String, HarmonyError> =
                        checker.get_path_of_import(&import.clone());
                    if let Err(error) = full_path {
                        println!("{}", error.to_string());
                        return ControlFlow::Break(());
                    }
                    let full_path: String = full_path.unwrap();
                    if !self.compiled_files.contains_key(&full_path) {
                        if let ControlFlow::Break(_) = self.compile_file(&full_path) {
                            continue;
                        }
                    }
                    let scope: Scope = self.compiled_files.get(&full_path).unwrap().clone();
                    scopes.push(scope);
                }
                checker = Checker::new(&self, &statements.clone().unwrap(), file);
                for import in imports {
                    checker.global_scope.imports.push(import);
                }
                for scope in scopes {
                    checker.global_scope.merge(&scope);
                }
            }
            Err(error) => {
                println!("{}", error.to_string());
                return ControlFlow::Break(());
            }
        }

        let check_result: Result<(), HarmonyError> = checker.analyze();
        self.compiled_files
            .insert(file.clone(), checker.global_scope.clone());

        match check_result {
            Ok(_) => {}
            Err(error) => {
                println!("{}", error.to_string());
                return ControlFlow::Break(());
            }
        }

        println!("Compiled {} in {:?}!", file, now.elapsed());

        ControlFlow::Continue(())
    }
}

#[derive(Debug, Clone)]
pub struct CompilerOptions {
    pub keep: bool,
    pub output: Option<String>,
    pub verbose: bool,
}

impl CompilerOptions {
    pub fn new(options: Vec<String>) -> CompilerOptions {
        let mut keep = false;
        let mut output = None;
        let mut verbose = false;

        for option in options {
            match option.as_str() {
                "-k" | "--keep" => {
                    keep = true;
                }
                "-o" | "--output" => {
                    output = Some(option);
                }
                "-v" | "--verbose" => {
                    verbose = true;
                }
                _ => {
                    println!("Unknown option: {}", option);
                    std::process::exit(1);
                }
            }
        }

        CompilerOptions {
            keep: keep,
            output: output,
            verbose: verbose,
        }
    }
}

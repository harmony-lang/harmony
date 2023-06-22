use crate::{
    ast::Statement,
    error::{HarmonyError, HarmonyErrorKind},
    parser::Parser,
    token::{Token, TokenKind},
    tokenizer::Tokenizer,
};

#[derive(Debug, Clone)]
pub struct Compiler {
    pub options: CompilerOptions,
    pub files: Vec<String>,
}

impl Compiler {
    pub fn new(options: &CompilerOptions, files: &Vec<String>) -> Compiler {
        Compiler {
            options: options.clone(),
            files: files.clone(),
        }
    }

    pub fn compile(&self) {
        for file in &self.files {
            println!("Compiling {}", file);

            let source: String = std::fs::read_to_string(file).unwrap();
            // println!("{}", source);

            // Semantic analysis steps:
            //   1. Scan for imports
            //   2. Locate all identifiers and their types (ex. functions, variables, etc.)
            //   3. Check for type errors
            //   4. Remove unused code
            //   5. Generate JavaScript code

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
                continue;
            }

            let mut parser: Parser = Parser::new(tokens);
            let statements: Result<Vec<Statement>, HarmonyError> = parser.parse();
            match statements {
                Ok(statements) => {
                    for statement in statements {
                        println!("{:?}", statement);
                    }
                }
                Err(error) => {
                    println!("{}", error.to_string());
                }
            }
        }
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

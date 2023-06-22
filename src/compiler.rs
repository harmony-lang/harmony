use crate::{
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
            for token in tokens {
                if token.kind == TokenKind::Unknown {
                    println!("Unknown token: {}", token.lexeme);
                    break;
                }
                println!("{:?}", token);
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

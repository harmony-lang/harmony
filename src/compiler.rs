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
            println!("{}", source);

            println!(" -> {}", file);
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

use harmony::compiler::{Compiler, CompilerOptions};

fn usage() {
    println!("Usage: harmony <file> [options]");
    println!("Options:");
    println!("  -h, --help     Print this help message");
    println!("  -V, --version  Print version information");
    println!("  -k, --keep     Keep the generated JavaScript file");
    println!("  -o <file>, --output <file>");
    println!("                 Output the generated JavaScript to a file");
    println!("  -v, --verbose  Print verbose output");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let files: Vec<String> = args
        .iter()
        .filter(|&arg| !arg.starts_with("-") && !arg.starts_with("--"))
        .map(|arg| arg.to_string())
        .collect();
    let options: Vec<String> = args
        .iter()
        .filter(|&arg| arg.starts_with("-") || arg.starts_with("--"))
        .map(|arg| arg.to_string())
        .collect();

    if options.contains(&"-h".to_string()) || options.contains(&"--help".to_string()) {
        usage();
        return;
    }
    if options.contains(&"-V".to_string()) || options.contains(&"--version".to_string()) {
        println!("Harmony {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if files.len() == 0 {
        usage();
        return;
    }

    let compiler_options: CompilerOptions = CompilerOptions::new(options);
    let mut compiler: Compiler = Compiler::new(&compiler_options, &files);

    compiler.compile();
}

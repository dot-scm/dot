use std::process::ExitCode;

fn print_help() {
    println!("dot - A git command proxy CLI tool");
    println!();
    println!("USAGE:");
    println!("    dot <git-command> [arguments]");
    println!();
    println!("EXAMPLES:");
    println!("    dot status          # Same as 'git status'");
    println!("    dot add .           # Same as 'git add .'");
    println!("    dot commit -m \"msg\" # Same as 'git commit -m \"msg\"'");
    println!("    dot push            # Same as 'git push'");
    println!();
    println!("OPTIONS:");
    println!("    --version    Print version information");
    println!("    --help       Print this help message");
    println!();
    println!("All git commands and options are supported.");
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Handle dot's own commands
    if let Some(first_arg) = args.first() {
        match first_arg.as_str() {
            "--version" | "-V" => {
                println!("dot {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "--help" | "-h" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            _ => {}
        }
    }

    // If no arguments, show help
    if args.is_empty() {
        print_help();
        return ExitCode::SUCCESS;
    }

    // Proxy to git
    match dot::execute(&args) {
        Ok(code) => ExitCode::from(code as u8),
        Err(e) => {
            eprintln!("dot: {}", e);
            ExitCode::FAILURE
        }
    }
}

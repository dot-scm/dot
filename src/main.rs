use clap::{Parser, Subcommand};
use dot::{config::ConfigManager, index::IndexManager, repository::RepositoryManager, error::DotError};

#[derive(Parser)]
#[command(name = "dot")]
#[command(about = "A Git proxy for managing hidden directories with version control")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(long, help = "Skip hidden repository operations")]
    skip_hidden: bool,
    
    #[arg(long, help = "Disable atomic behavior")]
    no_atomic: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize dot project with hidden directories
    Init { 
        /// Hidden directories to manage
        directories: Vec<String> 
    },
    /// Show status of all repositories
    Status,
    /// Add files to all repositories
    Add { 
        /// Files to add (use . for all files)
        files: Vec<String> 
    },
    /// Commit changes to all repositories
    Commit { 
        #[arg(short, long)]
        /// Commit message
        message: String 
    },
    /// Push changes to all repositories
    Push,
    /// Clone project with hidden repositories
    Clone { 
        /// Repository URL to clone
        url: String,
        /// Target directory name (optional)
        target: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Load configuration
    let config = match ConfigManager::load().await {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };
    
    // Initialize index manager
    let index_manager = match IndexManager::new(&config).await {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("Failed to initialize index manager: {}", e);
            eprintln!("Make sure you have:");
            eprintln!("1. Set GITHUB_TOKEN environment variable");
            eprintln!("2. Added authorized organizations to ~/.dot/dot.conf");
            eprintln!("3. Set a default organization in the config");
            std::process::exit(1);
        }
    };
    
    // Create repository manager
    let mut repo_manager = RepositoryManager::new(config, index_manager);
    
    // Execute command
    let result = match cli.command {
        Commands::Init { directories } => {
            if directories.is_empty() {
                eprintln!("Error: At least one directory must be specified");
                eprintln!("Usage: dot init <directory1> [directory2] ...");
                std::process::exit(1);
            }
            repo_manager.init_project(directories, cli.skip_hidden, cli.no_atomic).await
                .map_err(DotError::from)
        },
        Commands::Status => {
            match repo_manager.status(cli.skip_hidden).await {
                Ok(status) => {
                    println!("{}", status);
                    Ok(())
                },
                Err(e) => Err(DotError::from(e)),
            }
        },
        Commands::Add { files } => {
            if files.is_empty() {
                eprintln!("Error: At least one file must be specified");
                eprintln!("Usage: dot add <file1> [file2] ... or dot add .");
                std::process::exit(1);
            }
            repo_manager.multi_repo_add(files, cli.skip_hidden, cli.no_atomic).await
                .map_err(DotError::from)
        },
        Commands::Commit { message } => {
            repo_manager.multi_repo_commit(message, cli.skip_hidden, cli.no_atomic).await
                .map_err(DotError::from)
        },
        Commands::Push => {
            match repo_manager.multi_repo_push(cli.skip_hidden, cli.no_atomic).await {
                Ok(results) => {
                    println!("{}", results);
                    Ok(())
                },
                Err(e) => Err(DotError::from(e)),
            }
        },
        Commands::Clone { url, target } => {
            repo_manager.clone_project(url, target).await
                .map_err(DotError::from)
        },
    };
    
    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("dot: {}", e);
            std::process::exit(1);
        }
    }
}

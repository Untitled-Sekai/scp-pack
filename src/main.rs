use std::path::PathBuf;
use clap::{Parser, Subcommand};
use scp_pack::Converter;

#[derive(Parser)]
#[command(name = "scp-pack")]
#[command(about = "A tool to convert between SCP files and pack directories")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Compression level (0-9, higher = better compression)
    #[arg(short, long, default_value = "6")]
    compression: i64,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert pack directory to SCP file
    Pack {
        /// Input pack directory
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output SCP file
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Convert SCP file to pack directory  
    Unpack {
        /// Input SCP file
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output pack directory
        #[arg(short, long)]
        output: PathBuf,
    },
    /// List contents of SCP file
    List {
        /// SCP file to list
        #[arg(short, long)]
        file: PathBuf,
    },
    /// Show content of specific file in SCP
    Show {
        /// SCP file
        #[arg(short, long)]
        scp: PathBuf,
        
        /// File path within SCP
        #[arg(short, long)]
        file: String,
    },
}

fn main() {
    let cli = Cli::parse();
    
    let converter = Converter::new().with_compression_level(cli.compression);
    
    let result = match cli.command {
        Commands::Pack { input, output } => {
            converter.pack_to_scp(&input, &output)
        },
        Commands::Unpack { input, output } => {
            converter.scp_to_pack(&input, &output)
        },
        Commands::List { file } => {
            converter.list_scp_contents(&file)
        },
        Commands::Show { scp, file } => {
            converter.show_file(&scp, &file)
        },
    };
    
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The jobsets where the given jobs will be searched for.
    #[arg(long = "jobset")]
    jobsets: Vec<String>,
    /// The jobs to monitor.
    #[arg(short, long = "job")]
    jobs: Vec<String>,
    /// The systems (e.g., x86_64-linux) to monitor jobs to.
    /// Defaults to nixpkgs' default systems (x86_64-linux, aarch64-linux, x86_64-darwin,
    /// and aarch64-dawrin).
    #[arg(long = "system", default_values_t = vec![
        "x86_64-linux".to_string(),
        "aarch64-linux".to_string(),
        "x86_64-darwin".to_string(),
        "aarch64-dawrin".to_string()
    ])]
    systems: Vec<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    println!("Hello, world!");
}

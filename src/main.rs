use std::process::exit;

use anyhow::Result;
use clap::Parser;
use nixpkgs_build_failure_notifier::{email::Mailer, hydra::HydraApi, state::BuildStore};

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
    /// and aarch64-darwin).
    #[arg(long = "system", default_values_t = vec![
        "x86_64-linux".to_string(),
        "aarch64-linux".to_string(),
        "x86_64-darwin".to_string(),
        "aarch64-darwin".to_string()
    ])]
    systems: Vec<String>,

    /// Connection string to the PostgreSQL database.
    #[arg(env)]
    db_url: String,

    /// Hostname of the SMTP server.
    #[arg(env)]
    smtp_host: String,
    /// Username to use when connecting to the SMTP server.
    #[arg(env)]
    smtp_username: String,
    /// Password to use when connecting to the SMTP server.
    #[arg(env)]
    smtp_password: String,
    /// Email address to send emails from.
    #[arg(env)]
    smtp_from: String,
    /// Destination address of the email notifications.
    #[arg(env)]
    smtp_to: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let hydra_api = HydraApi::new();
    let state = BuildStore::new(&cli.db_url).await?;

    let builds = {
        let mut builds = vec![];
        for (jobset, job) in generate_job_combinations(&cli) {
            match hydra_api.get_latest_build(jobset, &job).await {
                Ok(build) => {
                    builds.push(build);
                }
                Err(error) => eprintln!("Error: {error}"),
            }
        }
        builds
    };

    if builds.is_empty() {
        eprintln!("No builds found. Perhaps Hydra changed its HTML format?");
        exit(1);
    }

    let failing_changed: Vec<_> = {
        let mut failing_changed = vec![];
        for build in &builds {
            let old = state
                .update_build_status(&build.get_full_name(), build.buildstatus)
                .await?;
            if build.is_failing() && (old.is_none() || old != Some(build.buildstatus)) {
                failing_changed.push(build);
            }
        }
        failing_changed
    };
    let email = Mailer::new(
        &cli.smtp_host,
        cli.smtp_from,
        cli.smtp_to,
        cli.smtp_username,
        cli.smtp_password,
    )?;
    email.send_report(&failing_changed)?;

    let failing: Vec<_> = builds.iter().filter(|build| build.is_failing()).collect();
    if failing.is_empty() {
        println!("No builds failing");
    } else {
        println!("{} build(s) failing", failing.len());
        for build in failing {
            println!(
                "- {} - https://hydra.nixos.org/build/{} - {}",
                build.get_full_name(),
                build.id,
                build.build_status_to_str()
            );
        }
    }

    Ok(())
}

fn generate_job_combinations(cli: &Cli) -> impl Iterator<Item = (&str, String)> {
    cli.jobsets.iter().flat_map(|jobset| {
        cli.jobs.iter().flat_map(|job| {
            cli.systems.iter().map(|system| {
                (
                    jobset.as_str(),
                    format!("{}.{}", job.as_str(), system.as_str()),
                )
            })
        })
    })
}

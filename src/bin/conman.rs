use std::io;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::CompleteEnv;
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};
use conman::{Containers, PendingPackages};

fn containers_candidates() -> Vec<CompletionCandidate> {
    Vec::new()
}

fn packages_candidates() -> Vec<CompletionCandidate> {
    match PendingPackages::load(&conman::PENDING_PACKAGES_PATH) {
        Ok(packages) => packages.0.keys().map(CompletionCandidate::new).collect(),
        Err(_) => Vec::new(),
    }
}

#[derive(Subcommand)]
pub enum CliAction {
    Push {
        #[arg(add = ArgValueCandidates::new(containers_candidates))]
        containers: Vec<String>,
        #[arg(short, long, add = ArgValueCandidates::new(packages_candidates))]
        packages: Vec<String>,
        #[arg(short = 'e', long, add = ArgValueCandidates::new(packages_candidates))]
        packages_exclude: Vec<String>,
    },
    Drop {
        #[arg(add = ArgValueCandidates::new(packages_candidates))]
        packages: Vec<String>,
    },
    List,
    Install,
}

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    action: CliAction,
}

fn verify_packages(packages: &Vec<String>) -> bool {
    false
}

fn verify_containers(packages: &Vec<String>) -> bool {
    false
}

fn handle_push(containers: &Vec<String>) {}

fn main() -> Result<(), io::Error>{
    CompleteEnv::with_factory(Args::command).complete();

    let args = Args::parse();

    match args.action {
        CliAction::Push {
            containers,
            packages,
            packages_exclude,
        } => {

            let mut cs = Containers::load(&conman::CONTAINERS_PATH)
                .expect("Containers directory appears to be corrupt.");

            // final thing is that somehow stuff is removed from pending packages?
            cs.apply(&containers, PendingPackages::load(&conman::PENDING_PACKAGES_PATH)?);

            cs.store(&conman::CONTAINERS_PATH)?;
        }
        _ => (),
    }
    Ok(())
}

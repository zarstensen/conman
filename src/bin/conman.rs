use anyhow::Error;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::CompleteEnv;
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};
use conman::{Containers, PendingPackages};

use crate::commands::discover::handle_discover;
use crate::commands::drop::handle_drop;
use crate::commands::list::handle_list;
use crate::commands::push::handle_push;

mod commands;

fn containers_candidates() -> Vec<CompletionCandidate> {
    match Containers::load(&conman::CONTAINERS_PATH) {
        Ok(containers) => containers.containers.keys().map(CompletionCandidate::new).collect(),
        Err(_) => Vec::new(),
    }
}

fn packages_candidates() -> Vec<CompletionCandidate> {
    match PendingPackages::load(&conman::PENDING_PACKAGES_PATH) {
        Ok(pending_pkgs) => pending_pkgs.packages.keys().map(CompletionCandidate::new).collect(),
        Err(_) => Vec::new(),
    }
}

#[derive(Subcommand)]
pub enum CliAction {
    Push {
        #[arg(short, num_args = 1, add = ArgValueCandidates::new(containers_candidates))]
        containers: Vec<String>,
        #[arg(add = ArgValueCandidates::new(packages_candidates))]
        packages: Vec<String>,
    },
    Drop {
        #[arg(add = ArgValueCandidates::new(packages_candidates))]
        packages: Vec<String>,
    },
    List,
    Discover {
        packages: Vec<String>,
        ignore_containers: bool,
    },

}

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    action: CliAction,
}


fn main() -> Result<(), Error> {
    CompleteEnv::with_factory(Args::command).complete();

    let args = Args::parse();

    match args.action {
        CliAction::Push {
            containers,
            packages
        } => handle_push(containers, packages),
        CliAction::List => handle_list(),
        CliAction::Drop { packages } => handle_drop(packages),
        CliAction::Discover { packages, ignore_containers } => handle_discover(packages, ignore_containers),
        _ => Ok(()),
    }
}

use std::io;

use alpm::{Alpm, PackageReason};
use clap::Parser;
use conman::{CONTAINERS_PATH, Containers, PENDING_PACKAGES_PATH, PackageAction, PendingPackages, pacman_alpm};

#[derive(Parser)]
struct Args {
    action: PackageAction,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let mut pending_packages =
        PendingPackages::load(&PENDING_PACKAGES_PATH)?;

    let containers = Containers::load(&CONTAINERS_PATH)?;

    // go through the targets we have recieved and filter out the ones which are not marked as explicit.
    // then update the PendingPackages accordingly
    let alpm = pacman_alpm()?;
    let db = alpm.localdb();

    io::stdin()
        .lines()
        .map_while(Result::ok)
        .try_for_each(|target| -> Result<(), anyhow::Error> {
            let pkg = db.pkg(target.clone())?;
            let explicit = pkg.reason() == PackageReason::Explicit;

            if !explicit {
                return Ok(());
            }

            println!(
                "{} {}",
                match args.action {
                    PackageAction::Add => "Adding",
                    PackageAction::Remove => "Removing",
                },
                target
            );

            let should_register =  match args.action {
                PackageAction::Add => !containers.contains(&target),
                PackageAction::Remove => containers.contains(&target)
            };

            if should_register {
                pending_packages.packages.insert(target, args.action.clone());
            } else {
                pending_packages.packages.remove(&target);
            };
            Ok(())
        })?;

    // finally, store the pending packages back on disc
    pending_packages.store(&PENDING_PACKAGES_PATH);
    Ok(())
}

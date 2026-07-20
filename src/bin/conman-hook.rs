use std::io;

use alpm::{Alpm, PackageReason};
use clap::Parser;
use conman::{PENDING_PACKAGES_PATH, PackageAction, PendingPackages};

#[derive(Parser)]
struct Args {
    action: PackageAction,
}

fn main() {
    let args = Args::parse();

    let mut pending_packages =
        PendingPackages::load(&PENDING_PACKAGES_PATH).expect("Corrupt pending packages");

    // go through the targets we have recieved and filter out the ones which are not marked as explicit.
    // then update the PendingPackages accordingly
    let alpm_handle = Alpm::new("/", "/var/lib/pacman").expect("pacman no thingy?");
    let db = alpm_handle.localdb();

    io::stdin()
        .lines()
        .map_while(Result::ok)
        .for_each(|target| {
            let pkg = db.pkg(target.clone()).expect("Target was not installed");
            let explicit = pkg.reason() == PackageReason::Explicit;

            if !explicit {
                return;
            }

            println!(
                "{} {}",
                match args.action {
                    PackageAction::Add => "Adding",
                    PackageAction::Remove => "Removing",
                },
                target
            );

            pending_packages.0.insert(target, args.action.clone());
        });

    // finally, store the pending packages back on disc
    pending_packages.store(&PENDING_PACKAGES_PATH);
}

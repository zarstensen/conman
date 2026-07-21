use std::collections::HashSet;

use anyhow::Error;
use conman::{Containers, PendingPackages};

/// handle a push operation of some packages to some set of containers.
/// 
/// * `containers`: vec of containers which packages should be pushed to. if empty, defaults to common
/// * `packages`: list of packages to push, if empty defaults to all pending packages.
/// * `packages_exclude`: list of packages to *not* push.
pub fn handle_push(mut containers: Vec<String>, mut pkg_globs: Vec<String>) -> Result<(), Error> {
    let mut cs = Containers::load(&conman::CONTAINERS_PATH)
        .expect("Containers directory appears to be corrupt.");

    let pending_packages = PendingPackages::load(&conman::PENDING_PACKAGES_PATH)?;

    if pkg_globs.is_empty() {
        pkg_globs = pending_packages.packages.keys().cloned().collect();
    };

    if containers.is_empty() {
        containers = vec!["common".to_string()];
    }

    let (push_packages, pending_packages) = pending_packages.glob_extract(pkg_globs)?;
    
    println!("Pushing to {:?}:\n{}", containers, push_packages);

    cs.apply(&containers, &push_packages);
    cs.store(&conman::CONTAINERS_PATH)?;
    pending_packages.store(&conman::PENDING_PACKAGES_PATH);
    Ok(())
}

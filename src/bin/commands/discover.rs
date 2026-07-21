use anyhow::Error;
use conman::{Container, Containers, PackageAction, PendingPackages, discover_packages, pacman_alpm};

pub fn handle_discover(mut pkg_globs: Vec<String>, ignore_containers: bool) -> Result<(), Error> {
    if pkg_globs.is_empty() {
        pkg_globs = vec!["*".to_string()];
    }

    let mut discovered_pkgs = discover_packages(pkg_globs)?;

    if !ignore_containers {
        let containers = Containers::load(&conman::CONTAINERS_PATH)?;
        discovered_pkgs = discovered_pkgs.into_iter().filter(|pkg| !containers.contains(pkg)).collect();
    }

    println!("Discovered {} package(s)", discovered_pkgs.len());
    
    let mut pending_packages = PendingPackages::load(&conman::PENDING_PACKAGES_PATH)?;

    for pkg in discovered_pkgs {
        pending_packages.packages.insert(pkg, PackageAction::Add);
    }

    pending_packages.store(&conman::PENDING_PACKAGES_PATH);

    Ok(())
}

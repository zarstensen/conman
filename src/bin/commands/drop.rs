use conman::PendingPackages;

pub fn handle_drop(pkg_globs: Vec<String>) -> Result<(), anyhow::Error> {
    let pending_packages = PendingPackages::load(&conman::PENDING_PACKAGES_PATH)?;
    let (dropped_packages, pending_packages) = pending_packages.glob_extract(pkg_globs)?;
    println!("Dropped {} pending package(s):\n{}", dropped_packages.packages.len(), dropped_packages);
    pending_packages.store(&conman::PENDING_PACKAGES_PATH);
    Ok(())
}

use anyhow::Error;
use conman::{PendingPackages};

pub fn handle_list() -> Result<(), Error> {
    let pending_packages = PendingPackages::load(&conman::PENDING_PACKAGES_PATH)?;
    println!("{}", pending_packages);
    Ok(())
}

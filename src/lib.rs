// so... this is just pr. .config i guess?
pub mod tui;

use clap::ValueEnum;
use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap, HashSet},
    default, fs, io,
    path::{Path, PathBuf},
    sync::LazyLock,
};
use thiserror::Error;

use serde::{Deserialize, Serialize};

pub const PENDING_PACKAGES_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(r#"/var/lib/conman/pending_packages.json"#));
pub const CONTAINERS_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(r#"/etc/conman/containers/"#));

#[derive(Error, Debug)]
pub enum ConmanError {
    #[error("package '{0}' does not exist")]
    PackageNotFound(String),
}

#[derive(Serialize, Deserialize, ValueEnum, Clone)]
pub enum PackageAction {
    Add = 0,
    Remove = 1,
}

/// Set of packages which have been added / removed by the user,
/// but has not been added to any containers.
/// This is stored in a file on the PC, this struct also manages storing / loading this file.
#[derive(Serialize, Deserialize, Default)]
pub struct PendingPackages(pub HashMap<String, PackageAction>);

impl PendingPackages {
    pub fn load(path: &Path) -> Result<Self, serde_json::Error> {
        if path.exists() {
            let data = fs::read_to_string(path).unwrap_or_default();
            serde_json::from_str(&data)
        } else {
            Ok(PendingPackages(HashMap::new()))
        }
    }

    pub fn store(&self, path: &Path) {
        fs::create_dir_all(path.parent().unwrap()).expect("Could not save state");
        let data = serde_json::to_string(self).expect("Failed to serialize state");
        fs::write(path, data).expect("Failed to store state")
    }

    // modify pending packages somehow? maybe extract or split or something
    // basically need a way of saying, ok we *intend* on doing something with this subset of packages.
    // please extract it from the current pending packages, and if something fails, then we also
    // have a way of mergin the pending packages back in.
    pub fn extract(
        &mut self,
        packages: impl IntoIterator<Item = impl AsRef<str>> + Clone,
    ) -> Result<PendingPackages, ConmanError> {
        let mut res: PendingPackages = Default::default();

        for package in packages.clone() {
            res.0.insert(
                package.as_ref().to_owned(),
                self.0.get(package.as_ref()).ok_or(ConmanError(package.as_ref().to_owned()))?.clone(),
            );
        }

        for package in packages {
            self.0.remove(package.as_ref());
        }

        Ok(res)
    }
}

// pending packages could have methods on it which adds stuff to containers maybe? or returns stuff.
// container should at least have like an update which takes a target and an action?

/// A container manages a serializable set of packages.
#[derive(Default, Debug)]
pub struct Container {
    pub packages: BTreeSet<String>,
}

impl Container {
    pub fn serialize(&self) -> String {
        println!("Serializing: {:?}", self.packages);
        self.packages.iter().cloned().collect::<Vec<_>>().join("\n")
    }

    pub fn deserialize(serialized: String) -> Container {
        Container {
            packages: serialized
                .split('\n')
                .filter(|s| !s.trim().is_empty())
                .map(String::from)
                .collect(),
        }
    }

    pub fn update(&mut self, package: String, action: &PackageAction) -> () {
        match action {
            PackageAction::Add => {
                println!("{}", package);
                self.packages.insert(package);
            }
            PackageAction::Remove => {
                self.packages.remove(&package);
            }
        }
    }
}

#[derive(Default)]
pub struct Containers {
    pub containers: HashMap<String, Container>,
}

impl Containers {
    pub fn load(path: &Path) -> Result<Containers, io::Error> {
        let mut res: Containers = Default::default();

        if !path.exists() {
            return Ok(res);
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let _ = res.containers.insert(
                    entry.file_name().to_string_lossy().into_owned(),
                    Container::deserialize(fs::read_to_string(entry.path())?),
                );
            }
        }

        println!("Containers after load: {:?}", res.containers);

        Ok(res)
    }

    pub fn store(&self, path: &Path) -> Result<(), io::Error> {
        fs::create_dir_all(path)?;

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                fs::remove_file(entry.path())?
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Path must contain no subdirectories!",
                ));
            }
        }

        for (con_name, container) in &self.containers {
            fs::write(path.join(con_name), container.serialize())?;
        }

        Ok(())
    }

    pub fn apply<TConIter: IntoIterator<Item = impl AsRef<str>>>(
        &mut self,
        containers: TConIter,
        pending_packages: PendingPackages,
    ) {
        for container in containers {
            let con_entry = self
                .containers
                .entry(container.as_ref().to_owned())
                .or_default();

            for (package_name, action) in &pending_packages.0 {
                con_entry.update(package_name.clone(), action);
            }
        }
    }
}

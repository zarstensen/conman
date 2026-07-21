pub mod tui;

use alpm::Alpm;
use clap::ValueEnum;
use glob_match::glob_match;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap}, fs, io, path::{Path, PathBuf}, sync::LazyLock,
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

pub fn pacman_alpm() -> alpm::Result<Alpm> {
    Alpm::new("/", "/var/lib/pacman")
}

pub fn discover_packages(pkg_globs: Vec<String>) -> alpm::Result<Vec<String>> {
    let alpm = pacman_alpm()?;
    let db = alpm.localdb();

    Ok(db
        .pkgs()
        .iter()
        .filter(|pkg| pkg.reason() == alpm::PackageReason::Explicit)
        .map(|pkg| pkg.name().to_owned())
        .filter(|pkg| {
            pkg_globs
                .clone()
                .into_iter()
                .any(|glob| glob_match(glob.as_ref(), &pkg))
        })
        .collect())
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
pub struct PendingPackages {
    pub packages: BTreeMap<String, PackageAction>,
}

impl std::fmt::Display for PendingPackages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (package_name, action) in &self.packages {
            write!(
                f,
                "{} {}\n",
                match action {
                    PackageAction::Add => "+",
                    PackageAction::Remove => "-",
                },
                package_name
            )?;
        }
        Ok(())
    }
}

impl PendingPackages {
    pub fn load(path: &Path) -> Result<Self, serde_json::Error> {
        if path.exists() {
            let data = fs::read_to_string(path).unwrap_or_default();
            serde_json::from_str(&data)
        } else {
            Ok(Default::default())
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
    pub fn glob_extract(
        &self,
        pkg_globs: impl IntoIterator<Item = impl AsRef<str>> + Clone,
    ) -> Result<(PendingPackages, PendingPackages), ConmanError> {
        let mut matched: PendingPackages = Default::default();
        let mut unmatched: PendingPackages = Default::default();

        self.packages.iter().for_each(|(pkg, action)| {
            if pkg_globs
                .clone()
                .into_iter()
                .any(|glob| glob_match(glob.as_ref(), &pkg))
            {
                matched.packages.insert(pkg.to_owned(), action.to_owned());
            } else {
                unmatched.packages.insert(pkg.to_owned(), action.to_owned());
            }
        });

        Ok((matched, unmatched))
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
            if container.packages.is_empty() {
                continue;
            }

            fs::write(path.join(con_name), container.serialize())?;
        }

        Ok(())
    }

    pub fn apply<TConIter: IntoIterator<Item = impl AsRef<str>>>(
        &mut self,
        containers: TConIter,
        pending_packages: &PendingPackages,
    ) {
        for container in containers {
            let con_entry = self
                .containers
                .entry(container.as_ref().to_owned())
                .or_default();

            for (package_name, action) in &pending_packages.packages {
                con_entry.update(package_name.clone(), action);
            }
        }
    }

    pub fn contains(&self, package: &str) -> bool {
        for (_, container) in &self.containers {
            if container.packages.contains(package) {
                return true;
            }
        }
        false
    }
}

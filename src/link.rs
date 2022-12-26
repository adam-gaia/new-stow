use crate::settings::LinkSettings;
use anyhow::{bail, Result};
use log::{debug, error, info, trace, warn, Level};
use std::{
    fs::{self, Metadata},
    path::{Path, PathBuf},
};

#[derive(Debug)]
enum FileType {
    Dir,
    File,
    Symlink(PathBuf), // Includes the path the link points to
    BrokenSymlink,
    Other(Metadata),
}

#[derive(Debug)]
struct Target {
    path: PathBuf,
}
impl Target {
    fn new(p_string: String) -> Self {
        let path = PathBuf::from(p_string);
        Target { path }
    }

    fn create_parent_dir(&self, dry_run: bool) -> Result<()> {
        let Some(parent) = self.path.parent() else {
            bail!("Target {:?} does not have a valid parent path", self.path);
        };
        if dry_run {
            info!("Pretending to create parent dir for target {:?}", self.path)
        } else {
            info!("Creating parent dir for target {:?}", self.path);
            // TODO: add a specific error messages if creating the parent fails
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    fn file_type(&self) -> Result<Option<FileType>> {
        let path = &self.path;
        match path.try_exists() {
            Ok(exists) => {
                if exists {
                    if path.is_symlink() {
                        let points_to = path.canonicalize().unwrap();
                        return Ok(Some(FileType::Symlink(points_to)));
                    } else if path.is_file() {
                        return Ok(Some(FileType::File));
                    } else if path.is_dir() {
                        return Ok(Some(FileType::Dir));
                    } else {
                        let metadata = path.metadata()?;
                        return Ok(Some(FileType::Other(metadata)));
                    }
                } else {
                    // Check for broken symlink. I think try_exists() returns Ok(false) with broken
                    // symlinks
                    if path.is_symlink() {
                        return Ok(Some(FileType::BrokenSymlink));
                    } else {
                        return Ok(None);
                    }
                }
            }
            Err(e) => {
                bail!("Unexpected error: {}", e)
            }
        }
    }
}

fn files_are_the_same(a: &Path, b: &Path) -> Result<bool> {
    a.canonicalize()?;
    b.canonicalize()?;
    Ok(a == b)
}

#[derive(Debug)]
pub struct Link<'a> {
    src: PathBuf,
    target: Target,
    settings: &'a LinkSettings,
}
impl<'a> Link<'a> {
    pub fn new(src: String, target: String, settings: &'a LinkSettings) -> Result<Self> {
        let src = PathBuf::from(src);
        if src.canonicalize().is_err() {
            bail!("Source file {:?} does not exist", src);
        };
        let target = Target::new(target);
        Ok(Link {
            src,
            target,
            settings,
        })
    }

    pub fn link(&self) -> Result<()> {
        let dry_run = self.settings.dry_run();
        if let Some(target_file_type) = self.target.file_type()? {
            match target_file_type {
                FileType::Dir => {
                    todo!()
                }
                FileType::File => {
                    todo!()
                }
                FileType::Symlink(points_to) => {
                    todo!()
                }
                FileType::BrokenSymlink => {
                    if dry_run {
                        warn!(
                            "Target {:?} is an existing broken symlink. Pretending to remove",
                            self.target.path
                        );
                    } else {
                        warn!(
                            "Target {:?} is an existing broken symlink. Removing",
                            self.target.path
                        );
                        fs::remove_file(&self.target.path).unwrap();
                    }
                }
                FileType::Other(metadata) => {
                    bail!(
                        "Target {:?} exists and is an un-handable type: {:?}",
                        self.target.path,
                        metadata
                    )
                }
            }
        }

        self.target.create_parent_dir(dry_run)?;

        if dry_run {
            info!(
                "Pretending to link {:?} -> {:?}",
                self.src, self.target.path
            );
        } else {
            info!("Linking {:?} -> {:?}", self.src, self.target.path);
            std::os::unix::fs::symlink(&self.src, &self.target.path)?;
        }

        Ok(())
    }

    pub fn unlink(&self) -> Result<()> {
        let dry_run = self.settings.dry_run();
        let target_file_type = self.target.file_type()?;
        match target_file_type {
            Some(file_type) => match file_type {
                FileType::Symlink(points_to) => {
                    // TODO: dont unwrap
                    if !files_are_the_same(&self.src, &points_to).unwrap() {
                        bail!("Target points to something other than source. Source: {:?}, target: {:?}, target points to: {:?}", &self.target.path, self.src, points_to);
                    }
                    if dry_run {
                        info!("Pretending to remove target {:?}", self.target.path);
                    } else {
                        info!("Removing target {:?}", self.target.path);
                        fs::remove_file(&self.target.path)?;
                    }
                }
                _ => {
                    bail!(
                        "Cannot remove target {:?}. It is not a symlink",
                        self.target.path
                    );
                }
            },
            None => {
                bail!(
                    "Source file {:?} does not point to target {:?}. Cannot unlink",
                    self.src,
                    self.target.path
                );
            }
        }
        Ok(())
    }

    pub fn status(&self) -> Result<()> {
        if let Some(target_file_type) = self.target.file_type()? {
            match target_file_type {
                FileType::Dir => {
                    warn!("Target {:?} is a conflicting directory", self.target.path);
                }
                FileType::File => {
                    warn!("Target {:?} is a conflicting file", self.target.path);
                }
                FileType::Symlink(points_to) => {
                    if files_are_the_same(&self.src, &points_to).unwrap() {
                        info!("Target points to ");
                    } else {
                        warn!("{:?} -> {:?}", self.src, self.target.path);
                    }
                }
                FileType::BrokenSymlink => {
                    warn!("Target is a broken symlink");
                }
                FileType::Other(_) => {
                    warn!("Target is a conflicting file of an un-handable type")
                }
            }
        } else {
            info!("Source file {:?} is unlinked", self.src)
        };

        Ok(())
    }
}

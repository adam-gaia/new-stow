use crate::filter::StowFilters;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct LinkSettings {
    dry_run: bool,
    backup: Option<PathBuf>,
}
impl LinkSettings {
    pub fn new(dry_run: bool, backup: Option<PathBuf>) -> Self {
        LinkSettings { dry_run, backup }
    }

    pub fn dry_run(&self) -> bool {
        self.dry_run
    }

    pub fn backup(&self) -> &Option<PathBuf> {
        &self.backup
    }
}

pub struct Settings {
    stowfile_path: PathBuf,
    current_working_dir: PathBuf,
    filters: StowFilters,
    link_settings: LinkSettings,
}
impl Settings {
    pub fn new(
        stowfile_path: PathBuf,
        current_working_dir: PathBuf,
        dry_run: bool,
        backup: Option<PathBuf>,
        filters: StowFilters,
    ) -> Self {
        let link_settings = LinkSettings::new(dry_run, backup);
        Settings {
            stowfile_path,
            current_working_dir,
            filters,
            link_settings,
        }
    }

    pub fn stowfile_path(&self) -> &Path {
        &self.stowfile_path
    }

    pub fn current_working_dir(&self) -> &Path {
        &self.current_working_dir
    }

    pub fn filters(&self) -> &StowFilters {
        &self.filters
    }

    pub fn link_settings(&self) -> &LinkSettings {
        &self.link_settings
    }
}

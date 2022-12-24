use anyhow::{bail, Result};
use clap::{ArgAction, ArgGroup, Parser};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::{debug, error, info, trace, warn};
use std::env::{self, current_dir};
use std::path::{Path, PathBuf};
mod link;

mod stow;
use stow::Stow;

mod settings;
use settings::Settings;

mod filter;
use filter::StowFilters;

const DEFAULT_STOWFILE_NAMES: &'static [&'static str] = &[
    "stowfile",
    "Stowfile",
    "STOWFILE",
    "stowfile.yaml",
    "Stowfile.yaml",
    "STOWFILE.yaml",
];

/// New Stow - manage famrs of symbolic links with stowfiles
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(
            ArgGroup::new("action")
                .required(false)
                .args(["stow", "unstow", "delete", "restow"]),
        ))]
#[command(group(
            ArgGroup::new("test")
                .required(false)
                .args(["dry_run", "simulate", "no"]),
        ))]
struct Args {
    #[clap(flatten)]
    verbose: Verbosity<InfoLevel>,

    /// Sets a custom stowfilefile
    #[arg(short, long, value_name = "FILE")]
    stowfile: Option<PathBuf>,

    /// Sets the working directory to "DIR" instead of the current directory.
    /// Commands will be performed as if nstow was invoked from this directory.
    /// When combined with '--stowfile' paths in the stowfile will be intrepreted relative to
    /// "DIR", rather than the current working directory.
    #[arg(short, long, value_name = "DIR")]
    dir: Option<PathBuf>,

    /// Do not perform any actions, just show what would be done.
    /// This option is useless when combined with '-qqq'.
    #[arg(long)]
    dry_run: bool,

    /// Alias for '--dry-run' to preserve some compatibility with GNU Stow.
    #[arg(short, long)]
    no: bool,

    /// Alias for '--dry-run' to preserve some compatibility with GNU Stow.
    #[arg(long)]
    simulate: bool,

    /// Remove previously created symlinks.
    #[arg(long)]
    unstow: bool,

    /// Alias for '--unstow' arg. Used to preserve some compatibility with GNU Stow.
    #[arg(short = 'D', long)]
    delete: bool,

    /// Create symlinks specified by the stowfile.
    /// This is the default action when no other action is requested.
    #[arg(short = 'S', long)]
    stow: bool,

    /// Restow by deleting any existing links before linking again.
    /// Shortcut for 'nstow --unstow && nstow --stow'
    #[arg(short = 'R', long)]
    restow: bool,

    /// Ignore source files that match this regex.
    /// This flag may be passed multiple times and combined with '--only'.
    #[arg(long, value_name = "REGEX", action = ArgAction::Append)]
    ignore: Option<Vec<String>>,

    /// Only stow source files that match this regex.
    /// This flag may be passed multiple times and combined with '--ignore'.
    #[arg(long, value_name = "REGEX", action = ArgAction::Append)]
    only: Option<Vec<String>>,

    /// Ignore targets that match this regex.
    /// This flag may be passed multiple times and combined with '--only-target'.
    #[arg(long, value_name = "REGEX", action = ArgAction::Append)]
    ignore_target: Option<Vec<String>>,

    /// Only stow targets that match this regex.
    /// This flag may be passed multiple times and combined with '--ignore-target'.
    #[arg(long, value_name = "REGEX", action = ArgAction::Append)]
    only_target: Option<Vec<String>>,

    /// Sets a directory for backuping up any existing files at target locations.
    /// This option may be used with '--override', in which case all files are backed up except
    /// those marked for overriding.
    #[arg(short, long, value_name = "BACKUP_DIR")]
    backup: Option<PathBuf>, //TODO: set default location

    /// Force overriding of any existing targets or files at target locations.
    #[arg(long, value_name = "REGEX", action = ArgAction::Append)]
    r#override: Option<Vec<String>>,
    // TODO: add a '--restore' flag to restore the last backups when unstowing
    // Save the last backups in subdir and restore files from that subdir
}

fn check_for_default_stowfile(working_dir: &Path) -> Option<PathBuf> {
    for name in DEFAULT_STOWFILE_NAMES {
        let mut stowfile = working_dir.to_path_buf();
        stowfile.push(name);
        if stowfile.try_exists().is_ok() {
            return Some(stowfile);
        }
    }
    None
}

fn main() -> Result<()> {
    let args = Args::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let actual_current_working_dir = env::current_dir()?;

    // Grab the working dir from the user's input input arg. Fallback to the actual current working dir
    let working_dir = match args.dir {
        Some(working_dir) => working_dir,
        None => actual_current_working_dir.clone(),
    };
    working_dir.canonicalize()?;
    debug!("Working dir: {}", working_dir.display());

    // Grab the stowfile from user's input arg. Fallback to a stowfile in the current dir
    let stowfile_path = match args.stowfile {
        Some(stowfile_path) => {
            if stowfile_path.try_exists().is_err() {
                bail!(
                    "Specified stowfile file ({}) does not exist.",
                    stowfile_path.display()
                );
            }
            stowfile_path
        }
        None => {
            // Try to find a stowfile in the current directory
            let Some(stowfile_path) = check_for_default_stowfile(&working_dir) else {
                bail!("Unable to find stowfile in the working directory");
            // TODO: fall back to gnu stow's behavior when no stowfile is present?
            };
            stowfile_path
        }
    };
    debug!("Stowfile: {}", stowfile_path.display());

    let mut default_backup_location = actual_current_working_dir;
    default_backup_location.push("backups");

    let filters = StowFilters::new(
        args.only,
        args.ignore,
        args.only_target,
        args.ignore_target,
        args.r#override,
    );
    let dry_run = args.dry_run || args.simulate || args.no;
    let settings = Settings::new(stowfile_path, working_dir, dry_run, args.backup, filters);
    let app = Stow::with_settings(&settings)?;

    match (args.stow, args.unstow || args.delete, args.restow) {
        (false, false, false) => app.stow()?,
        (true, false, false) => app.stow()?,
        (false, true, false) => app.unstow()?,
        (false, false, true) => app.restow()?,
        _ => {
            bail!("Only one action may be specified");
        }
    }

    info!("Done");
    Ok(())
}

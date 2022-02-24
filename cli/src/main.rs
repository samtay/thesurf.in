use anyhow::{anyhow, bail, Result};
use clap::Parser;
use directories::ProjectDirs;
use std::{
    fs::{self, File},
    path::PathBuf,
};

use lib::msw::crawler::Crawler;

/// The accompanying CLI to thesurf.in
#[derive(Parser, Debug)]
#[clap(name = "thesurf.in", bin_name = "thesurf.in", version, author, version, about, long_about = None)]
struct Args {
    /// Surf spot
    #[clap(required_unless_present("update"))]
    spot: Option<String>,

    /// Update MSW surf spot mapping
    #[clap(short, long)]
    update: bool,

    /// Filepath to put spot mapping json
    ///
    /// For defaults, see https://docs.rs/directories/4.0.1/directories/struct.ProjectDirs.html#examples
    #[clap(short, long, requires = "update", value_hint = clap::ValueHint::FilePath)]
    path: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.update {
        let file_path = match args.path {
            Some(fp) => PathBuf::from(fp),
            None => {
                let project_dirs =
                    ProjectDirs::from("", "Sam Tay", "thesurf.in").ok_or_else(|| {
                        anyhow!(
                            "Couldn't find an appropriate cache dir, please specify with --path"
                        )
                    })?;
                let cache_dir = project_dirs.cache_dir();
                fs::create_dir_all(cache_dir)?;
                cache_dir.join("spots.json")
            }
        };
        let mut file = File::create(file_path)?;
        Crawler::new().crawl_spot_ids(&mut file)?;
    }

    if let Some(_spot) = args.spot {
        dbg!(_spot);
        bail!("Not yet implemented")
    }

    Ok(())
}

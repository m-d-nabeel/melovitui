use clap::Parser;
use directories::{ProjectDirs, UserDirs};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "melovitui",
    author,
    version,
    about = "A terminal-based music player with a clean TUI"
)]
pub struct Args {
    /// Path to the music directory
    #[arg(short, long)]
    music_dir: Option<PathBuf>,
}

#[derive(Deserialize, Debug)]
struct Config {
    music_dir: Option<String>,
}

fn get_config_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "", "melovitui").map(|proj_dirs| {
        let config_dir = proj_dirs.config_dir();
        config_dir.join("config.json")
    })
}

pub fn get_music_dir() -> PathBuf {
    let args = Args::parse();

    // Try command line argument first
    if let Some(dir) = args.music_dir {
        if dir.exists() {
            return dir;
        }
        eprintln!(
            "Warning: Specified music directory does not exist: {:?}",
            dir
        );
    }

    // Try config file
    if let Some(config_path) = get_config_path() {
        if let Ok(config_str) = fs::read_to_string(config_path) {
            if let Ok(config) = serde_json::from_str::<Config>(&config_str) {
                if let Some(dir_str) = config.music_dir {
                    let dir = PathBuf::from(dir_str);
                    if dir.exists() {
                        return dir;
                    }
                    eprintln!(
                        "Warning: Music directory from config does not exist: {:?}",
                        dir
                    );
                }
            }
        }
    }

    // Try user's Music directory
    if let Some(user_dirs) = UserDirs::new() {
        if let Some(music_dir) = user_dirs.audio_dir() {
            if music_dir.exists() {
                return music_dir.to_path_buf();
            }
        }
    }

    // Fallback to current directory
    eprintln!("Warning: Falling back to current directory");
    PathBuf::from(".")
}

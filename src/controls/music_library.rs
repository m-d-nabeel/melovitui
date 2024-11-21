use std::path::PathBuf;
use std::time::Duration;

#[derive(Default, Clone)]
pub struct MusicLibrary {
    pub tracks: Vec<Track>,
    pub selected_index: Option<usize>,
    pub current_dir: PathBuf,
}

#[derive(Clone, Debug)]
pub struct Track {
    pub title: String,
    pub artist: Option<String>,
    pub path: PathBuf,
    pub duration: Option<Duration>,
}

impl MusicLibrary {
    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }

    pub fn select_track(&mut self, index: usize) -> Option<&Track> {
        if index < self.tracks.len() {
            self.selected_index = Some(index);
            Some(&self.tracks[index])
        } else {
            None
        }
    }

    pub fn remove_track(&mut self, index: usize) -> Option<Track> {
        if index < self.tracks.len() {
            Some(self.tracks.remove(index))
        } else {
            None
        }
    }
}

impl Track {
    pub fn new(path: PathBuf) -> Self {
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Self {
            title: filename,
            artist: None,
            path,
            duration: None,
        }
    }

    pub fn with_metadata(
        path: PathBuf,
        title: String,
        artist: Option<String>,
        duration: Option<Duration>,
    ) -> Self {
        Self {
            title,
            artist,
            path,
            duration,
        }
    }
}

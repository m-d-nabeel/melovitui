use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;
use walkdir::WalkDir;

#[derive(Default, Clone)]
pub struct MusicLibrary {
    pub tracks: Vec<Track>,
    pub selected_index: Option<usize>,
    pub current_index: usize,
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
    /// Create a new MusicLibrary by loading tracks from the specified directory
    pub fn new(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let mut library = Self {
            current_dir: path.clone(),
            ..Default::default()
        };
        library.load_tracks(path)?;
        Ok(library)
    }

    /// Load tracks from a directory, supporting recursive search
    pub fn load_tracks(&mut self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        // Clear existing tracks
        self.tracks.clear();

        // Supported audio file extensions
        let supported_extensions = ["mp3", "wav", "flac", "ogg", "m4a"];

        // Walk through directory and load tracks
        for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
            let file_path = entry.path().to_path_buf();

            // Check if file has a supported audio extension
            if let Some(ext) = file_path.extension() {
                if supported_extensions.contains(&ext.to_str().unwrap_or("")) {
                    // Try to get metadata (you might want to use a library like metaflac or mp3-metadata)
                    let track = Track::new(file_path);
                    self.add_track(track);
                }
            }
        }

        // Sort tracks by filename if needed
        self.tracks.sort_by(|a, b| a.title.cmp(&b.title));

        // Set initial selection to first track if tracks exist
        if !self.tracks.is_empty() {
            self.selected_index = Some(0);
        }

        Ok(())
    }

    /// Add a track to the library
    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }

    /// Select a track by index
    pub fn select_track(&mut self, index: usize) -> Option<&Track> {
        if index < self.tracks.len() {
            self.selected_index = Some(index);
            Some(&self.tracks[index])
        } else {
            None
        }
    }

    /// Remove a track by index
    pub fn remove_track(&mut self, index: usize) -> Option<Track> {
        if index < self.tracks.len() {
            Some(self.tracks.remove(index))
        } else {
            None
        }
    }

    /// Select previous track
    pub fn select_previous(&mut self) {
        if let Some(current_index) = self.selected_index {
            if current_index > 0 {
                self.selected_index = Some(current_index - 1);
            }
        } else if !self.tracks.is_empty() {
            // If no track is selected, select the last track
            self.selected_index = Some(self.tracks.len() - 1);
        }
    }

    /// Select next track
    pub fn select_next(&mut self) {
        if let Some(current_index) = self.selected_index {
            if current_index < self.tracks.len() - 1 {
                self.selected_index = Some(current_index + 1);
            }
        } else if !self.tracks.is_empty() {
            // If no track is selected, select the first track
            self.selected_index = Some(0);
        }
    }

    /// Get the currently selected track
    pub fn get_selected_track(&self) -> Option<&Track> {
        self.selected_index.and_then(|index| self.tracks.get(index))
    }
}

impl Track {
    /// Create a new track from a path
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

    /// Create a track with full metadata
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

    /// Attempt to extract metadata (placeholder - you'd use a metadata library)
    pub fn extract_metadata(&mut self) {
        // TODO: Implement metadata extraction
        // You might want to use libraries like:
        // - metaflac for FLAC
        // - mp3-metadata for MP3
        // - symphonia for multiple formats
    }
}

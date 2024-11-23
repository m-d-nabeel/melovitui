use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;
use walkdir::WalkDir;

#[derive(Default, Clone)]
pub struct MusicLibrary {
    pub tracks: Vec<Track>,
    pub selected_index: Option<usize>,
    pub current_index: usize,
    #[allow(dead_code)]
    pub current_dir: PathBuf,
}

#[derive(Clone, Debug)]
pub struct Track {
    pub title: String,
    #[allow(dead_code)]
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
                    // Try to get metadata
                    let track = Track::from_file(file_path)?;
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

    /// Select previous track, wrapping around to the last track if at the beginning
    pub fn select_previous(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
        } else if !self.tracks.is_empty() {
            self.current_index = self.tracks.len() - 1;
        }
    }

    /// Select next track, wrapping around to the first track if at the end
    pub fn select_next(&mut self) {
        if self.current_index + 1 < self.tracks.len() {
            self.current_index += 1;
        } else if !self.tracks.is_empty() {
            self.current_index = 0;
        }
    }
}

impl Track {
    /// Create a new track from a file path and attempt to extract metadata
    pub fn from_file(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let title = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let duration = Track::extract_duration(&path)?;

        Ok(Self {
            title,
            artist: None, // Metadata for artist can also be extracted
            path,
            duration,
        })
    }

    /// Extract duration of the audio file using Symphonia
    fn extract_duration(path: &PathBuf) -> Result<Option<Duration>, Box<dyn Error>> {
        // Prepare to probe the file
        let hint = Hint::new();
        let src = std::fs::File::open(path)?;
        let mss = MediaSourceStream::new(Box::new(src), Default::default());
        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();

        // Probe the file
        let probed = get_probe().format(&hint, mss, &format_opts, &metadata_opts)?;

        // Get the first audio track
        let track = probed
            .format
            .tracks()
            .iter()
            .find(|t| t.codec_params.sample_rate.is_some())
            .ok_or("No playable audio tracks found")?;

        // Calculate duration
        if let Some(sample_rate) = track.codec_params.sample_rate {
            if let Some(n_frames) = track.codec_params.n_frames {
                let duration = Duration::from_secs_f64(n_frames as f64 / sample_rate as f64);
                return Ok(Some(duration));
            }
        }

        Ok(None)
    }
}

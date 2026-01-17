/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Error types for Ferro Media

use thiserror::Error;

/// Result type for media operations
pub type MediaResult<T> = Result<T, MediaError>;

/// Media error types
#[derive(Debug, Error)]
pub enum MediaError {
    #[cfg(feature = "ffmpeg")]
    #[error("FFmpeg error: {0}")]
    Ffmpeg(#[from] ffmpeg_next::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Decoder error: {0}")]
    Decoder(String),
    
    #[error("No audio stream found")]
    NoAudioStream,
    
    #[error("No video stream found")]
    NoVideoStream,
    
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    #[error("Playback error: {0}")]
    Playback(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("End of stream")]
    EndOfStream,
    
    #[error("Player not initialized")]
    NotInitialized,
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),
}

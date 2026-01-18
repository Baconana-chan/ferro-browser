/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Ferro Media - FFmpeg-based media backend for Ferro Browser
//!
//! This crate provides video and audio playback using FFmpeg instead of GStreamer,
//! offering better cross-platform compatibility and simpler deployment.
//!
//! ## Features
//! - `ffmpeg` - Enable full FFmpeg-based media playback (requires FFmpeg installed)
//! - `hwaccel` - Enable hardware acceleration where available

#![deny(unsafe_code)]

pub mod error;
pub mod video;
pub mod mse;

#[cfg(feature = "ffmpeg")]
pub mod player;
#[cfg(feature = "ffmpeg")]
pub mod audio;
#[cfg(feature = "ffmpeg")]
pub mod decoder;

pub use error::{MediaError, MediaResult};
pub use mse::{SegmentParser, MseSourceBuffer, TimeRange, MediaSegment, AppendMode, CodecInfo};

#[cfg(feature = "ffmpeg")]
pub use player::{MediaPlayer, MediaPlayerBuilder, PlaybackState, PlayerEvent};

use std::sync::OnceLock;
use log::info;

static INITIALIZED: OnceLock<bool> = OnceLock::new();

/// Initialize the Ferro Media subsystem.
/// Must be called before creating any media players.
pub fn init() -> MediaResult<()> {
    INITIALIZED.get_or_init(|| {
        info!("Initializing Ferro Media");
        
        #[cfg(feature = "ffmpeg")]
        {
            // Initialize FFmpeg
            ffmpeg_next::init().expect("Failed to initialize FFmpeg");
            
            // Log FFmpeg version info
            info!("FFmpeg version: {}", ffmpeg_next::format::version());
        }
        
        #[cfg(not(feature = "ffmpeg"))]
        {
            info!("Ferro Media initialized (FFmpeg disabled - using image-only mode)");
        }
        
        true
    });
    
    Ok(())
}

/// Check if FFmpeg supports a given MIME type
pub fn supports_mime_type(mime_type: &str) -> bool {
    #[cfg(feature = "ffmpeg")]
    {
        // Common supported formats
        matches!(mime_type, 
            "video/mp4" |
            "video/webm" |
            "video/ogg" |
            "video/x-matroska" |
            "audio/mp3" |
            "audio/mpeg" |
            "audio/ogg" |
            "audio/wav" |
            "audio/webm" |
            "audio/flac" |
            "audio/aac" |
            "audio/mp4"
        )
    }
    
    #[cfg(not(feature = "ffmpeg"))]
    {
        // Only image formats supported without FFmpeg
        matches!(mime_type,
            "image/gif" |
            "image/png" |
            "image/jpeg" |
            "image/webp"
        )
    }
}

/// Media type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    Audio,
    Video,
    AudioVideo,
    Image,
}

/// Check if ferro_media has FFmpeg support compiled in
pub fn has_ffmpeg_support() -> bool {
    cfg!(feature = "ffmpeg")
}

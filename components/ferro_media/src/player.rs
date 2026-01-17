/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Media player implementation using FFmpeg

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender, bounded};
use euclid::default::Size2D;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};

use crate::audio::AudioOutput;
use crate::decoder::MediaDecoder;
use crate::error::{MediaError, MediaResult};
use crate::video::VideoFrame;
use crate::MediaType;

/// Playback state of a media player
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    /// Player is stopped (initial state)
    Stopped,
    /// Player is buffering data
    Buffering,
    /// Player is playing
    Playing,
    /// Player is paused
    Paused,
    /// Playback completed
    Ended,
    /// An error occurred
    Error,
}

/// Events emitted by the media player
#[derive(Debug, Clone)]
pub enum PlayerEvent {
    /// Metadata has been loaded (duration, dimensions, etc)
    MetadataLoaded {
        duration: Option<Duration>,
        video_size: Option<Size2D<u32>>,
        has_audio: bool,
        has_video: bool,
    },
    /// Playback state changed
    StateChanged(PlaybackState),
    /// Current playback position updated
    PositionChanged(Duration),
    /// A new video frame is available
    VideoFrameAvailable,
    /// Buffering progress (0.0 - 1.0)
    BufferingProgress(f64),
    /// Seek completed
    SeekCompleted,
    /// An error occurred
    Error(String),
    /// End of media reached
    EndOfMedia,
}

/// Video frame renderer callback
pub trait VideoFrameRenderer: Send + Sync {
    fn render_frame(&self, frame: &VideoFrame);
}

/// Audio samples renderer callback  
pub trait AudioRenderer: Send + Sync {
    fn render_samples(&self, samples: &[f32], channels: u32, sample_rate: u32);
}

/// Media player builder
pub struct MediaPlayerBuilder {
    url: Option<String>,
    video_renderer: Option<Arc<dyn VideoFrameRenderer>>,
    audio_renderer: Option<Arc<dyn AudioRenderer>>,
}

impl MediaPlayerBuilder {
    pub fn new() -> Self {
        Self {
            url: None,
            video_renderer: None,
            audio_renderer: None,
        }
    }
    
    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }
    
    pub fn video_renderer(mut self, renderer: Arc<dyn VideoFrameRenderer>) -> Self {
        self.video_renderer = Some(renderer);
        self
    }
    
    pub fn audio_renderer(mut self, renderer: Arc<dyn AudioRenderer>) -> Self {
        self.audio_renderer = Some(renderer);
        self
    }
    
    pub fn build(self) -> MediaResult<MediaPlayer> {
        MediaPlayer::new(
            self.url,
            self.video_renderer,
            self.audio_renderer,
        )
    }
}

impl Default for MediaPlayerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Commands sent to the player thread
enum PlayerCommand {
    Play,
    Pause,
    Stop,
    Seek(Duration),
    SetVolume(f32),
    SetUrl(String),
    Shutdown,
}

/// Main media player struct
pub struct MediaPlayer {
    /// Current playback state
    state: Arc<RwLock<PlaybackState>>,
    
    /// Current playback position
    position: Arc<RwLock<Duration>>,
    
    /// Total duration (if known)
    duration: Arc<RwLock<Option<Duration>>>,
    
    /// Video dimensions
    video_size: Arc<RwLock<Option<Size2D<u32>>>>,
    
    /// Volume (0.0 - 1.0)
    volume: Arc<RwLock<f32>>,
    
    /// Command sender to player thread
    command_tx: Sender<PlayerCommand>,
    
    /// Event receiver from player thread
    event_rx: Receiver<PlayerEvent>,
    
    /// Is the player running
    running: Arc<AtomicBool>,
    
    /// Current video frame (for rendering)
    current_frame: Arc<Mutex<Option<VideoFrame>>>,
    
    /// Media type (audio, video, or both)
    media_type: Arc<RwLock<Option<MediaType>>>,
}

impl MediaPlayer {
    /// Create a new media player
    fn new(
        url: Option<String>,
        video_renderer: Option<Arc<dyn VideoFrameRenderer>>,
        audio_renderer: Option<Arc<dyn AudioRenderer>>,
    ) -> MediaResult<Self> {
        let (command_tx, command_rx) = bounded(32);
        let (event_tx, event_rx) = bounded(64);
        
        let state = Arc::new(RwLock::new(PlaybackState::Stopped));
        let position = Arc::new(RwLock::new(Duration::ZERO));
        let duration = Arc::new(RwLock::new(None));
        let video_size = Arc::new(RwLock::new(None));
        let volume = Arc::new(RwLock::new(1.0));
        let running = Arc::new(AtomicBool::new(true));
        let current_frame = Arc::new(Mutex::new(None));
        let media_type = Arc::new(RwLock::new(None));
        
        // Clone Arcs for the player thread
        let state_clone = Arc::clone(&state);
        let position_clone = Arc::clone(&position);
        let duration_clone = Arc::clone(&duration);
        let video_size_clone = Arc::clone(&video_size);
        let volume_clone = Arc::clone(&volume);
        let running_clone = Arc::clone(&running);
        let current_frame_clone = Arc::clone(&current_frame);
        let media_type_clone = Arc::clone(&media_type);
        
        // Spawn player thread
        std::thread::Builder::new()
            .name("ferro-media-player".to_string())
            .spawn(move || {
                player_thread(
                    command_rx,
                    event_tx,
                    state_clone,
                    position_clone,
                    duration_clone,
                    video_size_clone,
                    volume_clone,
                    running_clone,
                    current_frame_clone,
                    media_type_clone,
                    url,
                    video_renderer,
                    audio_renderer,
                );
            })?;
        
        Ok(Self {
            state,
            position,
            duration,
            video_size,
            volume,
            command_tx,
            event_rx,
            running,
            current_frame,
            media_type,
        })
    }
    
    /// Get current playback state
    pub fn state(&self) -> PlaybackState {
        *self.state.read()
    }
    
    /// Get current position
    pub fn position(&self) -> Duration {
        *self.position.read()
    }
    
    /// Get duration (if known)
    pub fn duration(&self) -> Option<Duration> {
        *self.duration.read()
    }
    
    /// Get video size (if video)
    pub fn video_size(&self) -> Option<Size2D<u32>> {
        *self.video_size.read()
    }
    
    /// Get volume
    pub fn volume(&self) -> f32 {
        *self.volume.read()
    }
    
    /// Set volume (0.0 - 1.0)
    pub fn set_volume(&self, volume: f32) {
        *self.volume.write() = volume.clamp(0.0, 1.0);
        let _ = self.command_tx.send(PlayerCommand::SetVolume(volume));
    }
    
    /// Start playback
    pub fn play(&self) -> MediaResult<()> {
        self.command_tx.send(PlayerCommand::Play)
            .map_err(|_| MediaError::InvalidState("Player thread not running".to_string()))
    }
    
    /// Pause playback
    pub fn pause(&self) -> MediaResult<()> {
        self.command_tx.send(PlayerCommand::Pause)
            .map_err(|_| MediaError::InvalidState("Player thread not running".to_string()))
    }
    
    /// Stop playback
    pub fn stop(&self) -> MediaResult<()> {
        self.command_tx.send(PlayerCommand::Stop)
            .map_err(|_| MediaError::InvalidState("Player thread not running".to_string()))
    }
    
    /// Seek to position
    pub fn seek(&self, position: Duration) -> MediaResult<()> {
        self.command_tx.send(PlayerCommand::Seek(position))
            .map_err(|_| MediaError::InvalidState("Player thread not running".to_string()))
    }
    
    /// Set media URL
    pub fn set_url(&self, url: &str) -> MediaResult<()> {
        self.command_tx.send(PlayerCommand::SetUrl(url.to_string()))
            .map_err(|_| MediaError::InvalidState("Player thread not running".to_string()))
    }
    
    /// Poll for events (non-blocking)
    pub fn poll_event(&self) -> Option<PlayerEvent> {
        self.event_rx.try_recv().ok()
    }
    
    /// Get current video frame for rendering
    pub fn get_current_frame(&self) -> Option<VideoFrame> {
        self.current_frame.lock().clone()
    }
    
    /// Check if player has video
    pub fn has_video(&self) -> bool {
        matches!(*self.media_type.read(), Some(MediaType::Video | MediaType::AudioVideo))
    }
    
    /// Check if player has audio
    pub fn has_audio(&self) -> bool {
        matches!(*self.media_type.read(), Some(MediaType::Audio | MediaType::AudioVideo))
    }
}

impl Drop for MediaPlayer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        let _ = self.command_tx.send(PlayerCommand::Shutdown);
    }
}

/// Player thread main function
#[allow(clippy::too_many_arguments)]
fn player_thread(
    command_rx: Receiver<PlayerCommand>,
    event_tx: Sender<PlayerEvent>,
    state: Arc<RwLock<PlaybackState>>,
    position: Arc<RwLock<Duration>>,
    duration: Arc<RwLock<Option<Duration>>>,
    video_size: Arc<RwLock<Option<Size2D<u32>>>>,
    volume: Arc<RwLock<f32>>,
    running: Arc<AtomicBool>,
    current_frame: Arc<Mutex<Option<VideoFrame>>>,
    media_type: Arc<RwLock<Option<MediaType>>>,
    initial_url: Option<String>,
    video_renderer: Option<Arc<dyn VideoFrameRenderer>>,
    audio_renderer: Option<Arc<dyn AudioRenderer>>,
) {
    let mut decoder: Option<MediaDecoder> = None;
    let mut audio_output: Option<AudioOutput> = None;
    
    // Load initial URL if provided
    if let Some(url) = initial_url {
        match load_media(&url, &state, &duration, &video_size, &media_type, &event_tx) {
            Ok((d, a)) => {
                decoder = Some(d);
                audio_output = a;
            }
            Err(e) => {
                let _ = event_tx.send(PlayerEvent::Error(e.to_string()));
            }
        }
    }
    
    while running.load(Ordering::SeqCst) {
        // Process commands
        match command_rx.try_recv() {
            Ok(PlayerCommand::Play) => {
                *state.write() = PlaybackState::Playing;
                let _ = event_tx.send(PlayerEvent::StateChanged(PlaybackState::Playing));
            }
            Ok(PlayerCommand::Pause) => {
                *state.write() = PlaybackState::Paused;
                let _ = event_tx.send(PlayerEvent::StateChanged(PlaybackState::Paused));
            }
            Ok(PlayerCommand::Stop) => {
                *state.write() = PlaybackState::Stopped;
                *position.write() = Duration::ZERO;
                let _ = event_tx.send(PlayerEvent::StateChanged(PlaybackState::Stopped));
            }
            Ok(PlayerCommand::Seek(pos)) => {
                if let Some(ref mut dec) = decoder {
                    if dec.seek(pos).is_ok() {
                        *position.write() = pos;
                        let _ = event_tx.send(PlayerEvent::SeekCompleted);
                    }
                }
            }
            Ok(PlayerCommand::SetVolume(vol)) => {
                *volume.write() = vol;
                if let Some(ref mut audio) = audio_output {
                    audio.set_volume(vol);
                }
            }
            Ok(PlayerCommand::SetUrl(url)) => {
                // Stop current playback
                *state.write() = PlaybackState::Stopped;
                decoder = None;
                audio_output = None;
                
                // Load new media
                match load_media(&url, &state, &duration, &video_size, &media_type, &event_tx) {
                    Ok((d, a)) => {
                        decoder = Some(d);
                        audio_output = a;
                    }
                    Err(e) => {
                        let _ = event_tx.send(PlayerEvent::Error(e.to_string()));
                    }
                }
            }
            Ok(PlayerCommand::Shutdown) => {
                break;
            }
            Err(_) => {}
        }
        
        // Decode and render frames if playing
        if *state.read() == PlaybackState::Playing {
            if let Some(ref mut dec) = decoder {
                match dec.decode_next_frame() {
                    Ok(Some(frame)) => {
                        // Update position
                        *position.write() = frame.timestamp;
                        let _ = event_tx.send(PlayerEvent::PositionChanged(frame.timestamp));
                        
                        // Handle video frame
                        if let Some(video_frame) = frame.video {
                            *current_frame.lock() = Some(video_frame.clone());
                            if let Some(ref renderer) = video_renderer {
                                renderer.render_frame(&video_frame);
                            }
                            let _ = event_tx.send(PlayerEvent::VideoFrameAvailable);
                        }
                        
                        // Handle audio samples
                        if let Some(audio_data) = frame.audio {
                            if let Some(ref renderer) = audio_renderer {
                                renderer.render_samples(
                                    &audio_data.samples,
                                    audio_data.channels,
                                    audio_data.sample_rate,
                                );
                            }
                            if let Some(ref mut output) = audio_output {
                                output.write_samples(&audio_data.samples);
                            }
                        }
                    }
                    Ok(None) => {
                        // End of stream
                        *state.write() = PlaybackState::Ended;
                        let _ = event_tx.send(PlayerEvent::EndOfMedia);
                        let _ = event_tx.send(PlayerEvent::StateChanged(PlaybackState::Ended));
                    }
                    Err(e) => {
                        log::error!("Decode error: {}", e);
                        *state.write() = PlaybackState::Error;
                        let _ = event_tx.send(PlayerEvent::Error(e.to_string()));
                    }
                }
            }
        } else {
            // Sleep a bit when not playing to avoid busy-waiting
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

/// Load media from URL
fn load_media(
    url: &str,
    state: &Arc<RwLock<PlaybackState>>,
    duration: &Arc<RwLock<Option<Duration>>>,
    video_size: &Arc<RwLock<Option<Size2D<u32>>>>,
    media_type: &Arc<RwLock<Option<MediaType>>>,
    event_tx: &Sender<PlayerEvent>,
) -> MediaResult<(MediaDecoder, Option<AudioOutput>)> {
    *state.write() = PlaybackState::Buffering;
    let _ = event_tx.send(PlayerEvent::StateChanged(PlaybackState::Buffering));
    
    let decoder = MediaDecoder::open(url)?;
    
    let dur = decoder.duration();
    let size = decoder.video_size();
    let has_audio = decoder.has_audio();
    let has_video = decoder.has_video();
    
    *duration.write() = dur;
    *video_size.write() = size;
    
    let mtype = match (has_audio, has_video) {
        (true, true) => Some(MediaType::AudioVideo),
        (true, false) => Some(MediaType::Audio),
        (false, true) => Some(MediaType::Video),
        (false, false) => None,
    };
    *media_type.write() = mtype;
    
    // Create audio output if needed
    let audio_output = if has_audio {
        AudioOutput::new(
            decoder.audio_sample_rate().unwrap_or(44100),
            decoder.audio_channels().unwrap_or(2),
        ).ok()
    } else {
        None
    };
    
    let _ = event_tx.send(PlayerEvent::MetadataLoaded {
        duration: dur,
        video_size: size,
        has_audio,
        has_video,
    });
    
    *state.write() = PlaybackState::Paused;
    let _ = event_tx.send(PlayerEvent::StateChanged(PlaybackState::Paused));
    
    Ok((decoder, audio_output))
}

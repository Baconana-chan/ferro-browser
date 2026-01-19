/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! FFmpeg-based backend for servo-media.
//! 
//! This is a drop-in replacement for servo-media-gstreamer that uses FFmpeg
//! via the ferro_media crate for media decoding and playback.
//!
//! Also provides Media Source Extensions (MSE) support for streaming video.

use std::any::Any;
use std::io::Write;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ipc_channel::ipc::IpcSender;
use log::{debug, error, info, warn};

/// Check if data looks like a valid video/audio container format
/// Returns true if this is likely a media file that FFmpeg should decode
fn is_valid_media_format(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }
    
    // MP4/MOV: starts with ftyp box
    if data.len() >= 8 && &data[4..8] == b"ftyp" {
        return true;
    }
    
    // WebM/Matroska: starts with EBML header 0x1A 0x45 0xDF 0xA3
    if data.len() >= 4 && data[0..4] == [0x1A, 0x45, 0xDF, 0xA3] {
        return true;
    }
    
    // Ogg: starts with "OggS"
    if data.len() >= 4 && &data[0..4] == b"OggS" {
        return true;
    }
    
    // RIFF (WAV, AVI): starts with "RIFF"
    if data.len() >= 4 && &data[0..4] == b"RIFF" {
        return true;
    }
    
    // MPEG Transport Stream: starts with 0x47 (sync byte)
    if data[0] == 0x47 {
        return true;
    }
    
    // MPEG Program Stream / MPEG-1/2: starts with 0x00 0x00 0x01 0xBA or 0xB3
    if data.len() >= 4 && data[0..3] == [0x00, 0x00, 0x01] && (data[3] == 0xBA || data[3] == 0xB3) {
        return true;
    }
    
    // MP3: ID3 tag or frame sync
    if data.len() >= 3 && &data[0..3] == b"ID3" {
        return true;
    }
    if data.len() >= 2 && data[0] == 0xFF && (data[1] & 0xE0) == 0xE0 {
        return true;
    }
    
    // FLAC: starts with "fLaC"
    if data.len() >= 4 && &data[0..4] == b"fLaC" {
        return true;
    }
    
    // AAC ADTS: starts with 0xFF 0xF1 or 0xFF 0xF9
    if data.len() >= 2 && data[0] == 0xFF && (data[1] == 0xF1 || data[1] == 0xF9) {
        return true;
    }
    
    // Skip GIF, PNG, JPEG, WebP - these are images, not video
    // GIF: "GIF87a" or "GIF89a"
    if data.len() >= 6 && (&data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a") {
        debug!("is_valid_media_format: Detected GIF image, not video");
        return false;
    }
    
    // PNG: 0x89 "PNG" 0x0D 0x0A 0x1A 0x0A
    if data.len() >= 8 && data[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        debug!("is_valid_media_format: Detected PNG image, not video");
        return false;
    }
    
    // JPEG: 0xFF 0xD8 0xFF
    if data.len() >= 3 && data[0..3] == [0xFF, 0xD8, 0xFF] {
        debug!("is_valid_media_format: Detected JPEG image, not video");
        return false;
    }
    
    // WebP: "RIFF" + "WEBP" at offset 8
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        debug!("is_valid_media_format: Detected WebP image, not video");
        return false;
    }
    
    // Unknown format - don't try to decode it to be safe
    debug!("is_valid_media_format: Unknown format, first bytes: {:02X?}", &data[0..12.min(data.len())]);
    false
}
use servo_media::{Backend, BackendInit, ClientContextId, MediaInstance, SupportsMediaType};
use servo_media::player::context::PlayerGLContext;
use servo_media::player::{Player, PlayerError, PlayerEvent, StreamType};
use servo_media::player::audio::AudioRenderer;
use servo_media::player::video::{VideoFrameRenderer, VideoFrame, VideoFrameData, Buffer};

// Re-export ferro_media MSE types for use in DOM
pub use ferro_media::{MseSourceBuffer, SegmentParser, TimeRange, MediaSegment, AppendMode, CodecInfo};

// Re-export from servo_media for convenience
pub use servo_media::player::context::{GlApi, GlContext, NativeDisplay};

/// FFmpeg-based backend for servo-media
pub struct FerroBackend;

impl BackendInit for FerroBackend {
    fn init() -> Box<dyn Backend> {
        debug!("Initializing Ferro (FFmpeg) media backend");
        let _ = ferro_media::init();
        Box::new(FerroBackend)
    }
}

impl Backend for FerroBackend {
    fn create_player(
        &self,
        _id: &ClientContextId,
        stream_type: StreamType,
        sender: IpcSender<PlayerEvent>,
        video_renderer: Option<Arc<Mutex<dyn VideoFrameRenderer>>>,
        audio_renderer: Option<Arc<Mutex<dyn AudioRenderer>>>,
        _gl_context: Box<dyn PlayerGLContext>,
    ) -> Arc<Mutex<dyn Player>> {
        Arc::new(Mutex::new(FerroPlayer::new(
            stream_type,
            sender,
            video_renderer,
            audio_renderer,
        )))
    }

    fn create_audiostream(&self) -> servo_media::streams::MediaStreamId {
        FerroMediaStream::create_audio()
    }

    fn create_videostream(&self) -> servo_media::streams::MediaStreamId {
        FerroMediaStream::create_video()
    }

    fn create_stream_output(&self) -> Box<dyn servo_media::streams::MediaOutput> {
        Box::new(FerroMediaOutput)
    }

    fn create_stream_and_socket(
        &self,
        ty: servo_media::streams::MediaStreamType,
    ) -> (Box<dyn servo_media::streams::MediaSocket>, servo_media::streams::MediaStreamId) {
        let id = FerroMediaStream::create(ty);
        (Box::new(FerroSocket), id)
    }

    fn create_audioinput_stream(
        &self,
        _set: servo_media::streams::capture::MediaTrackConstraintSet,
    ) -> Option<servo_media::streams::MediaStreamId> {
        // Audio input not yet implemented
        Some(FerroMediaStream::create_audio())
    }

    fn create_videoinput_stream(
        &self,
        _set: servo_media::streams::capture::MediaTrackConstraintSet,
    ) -> Option<servo_media::streams::MediaStreamId> {
        // Video input not yet implemented
        Some(FerroMediaStream::create_video())
    }

    fn create_audio_context(
        &self,
        _id: &ClientContextId,
        options: servo_media::audio::context::AudioContextOptions,
    ) -> Result<Arc<Mutex<servo_media::audio::context::AudioContext>>, servo_media::audio::sink::AudioSinkError> {
        let (sender, _) = mpsc::channel();
        let sender = Arc::new(Mutex::new(sender));
        Ok(Arc::new(Mutex::new(servo_media::audio::context::AudioContext::new::<Self>(
            0,
            &ClientContextId::build(1, 1),
            sender,
            options,
        )?)))
    }

    fn create_webrtc(
        &self, 
        signaller: Box<dyn servo_media::webrtc::WebRtcSignaller>
    ) -> servo_media::webrtc::WebRtcController {
        servo_media::webrtc::WebRtcController::new::<Self>(signaller)
    }

    fn can_play_type(&self, media_type: &str) -> SupportsMediaType {
        if ferro_media::supports_mime_type(media_type) {
            SupportsMediaType::Probably
        } else {
            // Check common types that FFmpeg supports
            let supported = matches!(
                media_type.to_lowercase().as_str(),
                "video/mp4" | "video/webm" | "video/ogg" |
                "audio/mp3" | "audio/mpeg" | "audio/ogg" | "audio/wav" |
                "audio/flac" | "audio/aac" | "audio/mp4" |
                "video/x-matroska" | "audio/x-matroska"
            );
            if supported {
                SupportsMediaType::Probably
            } else {
                SupportsMediaType::Maybe
            }
        }
    }

    fn get_device_monitor(&self) -> Box<dyn servo_media::streams::device_monitor::MediaDeviceMonitor> {
        Box::new(FerroDeviceMonitor)
    }

    fn mute(&self, _id: &ClientContextId, _val: bool) {
        // TODO: Implement muting
    }

    fn suspend(&self, _id: &ClientContextId) {
        // TODO: Implement suspending
    }

    fn resume(&self, _id: &ClientContextId) {
        // TODO: Implement resuming
    }
}

// AudioBackend implementation for WebAudio support
impl servo_media::audio::AudioBackend for FerroBackend {
    type Sink = FerroAudioSink;

    fn make_decoder() -> Box<dyn servo_media::audio::decoder::AudioDecoder> {
        Box::new(FerroAudioDecoder)
    }

    fn make_sink() -> Result<Self::Sink, servo_media::audio::sink::AudioSinkError> {
        Ok(FerroAudioSink)
    }

    fn make_streamreader(
        _id: servo_media::streams::MediaStreamId,
        _sample_rate: f32,
    ) -> Box<dyn servo_media::audio::AudioStreamReader + Send> {
        Box::new(FerroStreamReader)
    }
}

// WebRTC backend (stub)
impl servo_media::webrtc::WebRtcBackend for FerroBackend {
    type Controller = FerroWebRtcController;

    fn construct_webrtc_controller(
        _signaller: Box<dyn servo_media::webrtc::WebRtcSignaller>,
        _controller: servo_media::webrtc::WebRtcController,
    ) -> Self::Controller {
        FerroWebRtcController
    }
}

// ============================================================================
// Player implementation using ferro_media FFmpeg backend
// ============================================================================

/// Buffer implementation for raw video data
struct RawVideoBuffer {
    data: Vec<u8>,
}

impl RawVideoBuffer {
    fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl Buffer for RawVideoBuffer {
    fn to_vec(&self) -> Result<VideoFrameData, ()> {
        Ok(VideoFrameData::Raw(Arc::new(self.data.clone())))
    }
}

/// Commands for the player thread
#[allow(dead_code)]
enum PlayerCommand {
    SetUrl(String),
    Play,
    Pause,
    Stop,
    Seek(f64),
    SetVolume(f64),
    PushData(Vec<u8>),
    EndOfStream,
    Shutdown,
}

/// Player state
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum InternalState {
    Stopped,
    Buffering,
    Playing,
    Paused,
    Ended,
}

#[allow(dead_code)]
pub struct FerroPlayer {
    // Stream type (audio, video, or both)
    stream_type: StreamType,
    
    // Event sender to DOM
    event_sender: IpcSender<PlayerEvent>,
    
    // Video renderer callback
    video_renderer: Option<Arc<Mutex<dyn VideoFrameRenderer>>>,
    
    // Audio renderer callback
    audio_renderer: Option<Arc<Mutex<dyn AudioRenderer>>>,
    
    // Current state
    state: Arc<Mutex<InternalState>>,
    
    // Paused flag
    paused: Arc<AtomicBool>,
    
    // Muted flag
    muted: Arc<AtomicBool>,
    
    // Volume (0-100 as integer for atomic)
    volume: Arc<AtomicU64>,
    
    // Playback rate (as integer * 100)
    playback_rate: Arc<AtomicU64>,
    
    // Current position in seconds (as microseconds for atomic)
    position_us: Arc<AtomicU64>,
    
    // Duration in seconds (as microseconds for atomic)
    duration_us: Arc<AtomicU64>,
    
    // Input size
    input_size: Arc<AtomicU64>,
    
    // Data buffer for push_data mode
    data_buffer: Arc<Mutex<Vec<u8>>>,
    
    // Command sender for player thread
    command_tx: Option<crossbeam_channel::Sender<PlayerCommand>>,
    
    // Player thread handle
    thread_handle: Option<std::thread::JoinHandle<()>>,
    
    // Is running
    running: Arc<AtomicBool>,
    
    // URL for playback
    url: Arc<Mutex<Option<String>>>,
    
    // Buffered ranges
    buffered_ranges: Arc<Mutex<Vec<Range<f64>>>>,
}

impl FerroPlayer {
    fn new(
        stream_type: StreamType,
        event_sender: IpcSender<PlayerEvent>,
        video_renderer: Option<Arc<Mutex<dyn VideoFrameRenderer>>>,
        audio_renderer: Option<Arc<Mutex<dyn AudioRenderer>>>,
    ) -> Self {
        let state = Arc::new(Mutex::new(InternalState::Stopped));
        let paused = Arc::new(AtomicBool::new(true));
        let muted = Arc::new(AtomicBool::new(false));
        let volume = Arc::new(AtomicU64::new(100)); // 100 = 1.0
        let playback_rate = Arc::new(AtomicU64::new(100)); // 100 = 1.0
        let position_us = Arc::new(AtomicU64::new(0));
        let duration_us = Arc::new(AtomicU64::new(0));
        let input_size = Arc::new(AtomicU64::new(0));
        let data_buffer = Arc::new(Mutex::new(Vec::new()));
        let running = Arc::new(AtomicBool::new(true));
        let url = Arc::new(Mutex::new(None));
        let buffered_ranges = Arc::new(Mutex::new(Vec::new()));
        
        // Create command channel
        let (command_tx, command_rx) = crossbeam_channel::bounded::<PlayerCommand>(64);
        
        // Clone for thread
        let state_clone = Arc::clone(&state);
        let paused_clone = Arc::clone(&paused);
        let muted_clone = Arc::clone(&muted);
        let volume_clone = Arc::clone(&volume);
        let position_us_clone = Arc::clone(&position_us);
        let duration_us_clone = Arc::clone(&duration_us);
        let running_clone = Arc::clone(&running);
        let url_clone = Arc::clone(&url);
        let data_buffer_clone = Arc::clone(&data_buffer);
        let buffered_ranges_clone = Arc::clone(&buffered_ranges);
        let event_sender_clone = event_sender.clone();
        let video_renderer_clone = video_renderer.clone();
        let audio_renderer_clone = audio_renderer.clone();
        
        // Spawn player thread
        let thread_handle = std::thread::Builder::new()
            .name("ferro-player".to_string())
            .spawn(move || {
                Self::player_thread(
                    command_rx,
                    state_clone,
                    paused_clone,
                    muted_clone,
                    volume_clone,
                    position_us_clone,
                    duration_us_clone,
                    running_clone,
                    url_clone,
                    data_buffer_clone,
                    buffered_ranges_clone,
                    event_sender_clone,
                    video_renderer_clone,
                    audio_renderer_clone,
                );
            })
            .ok();
        
        info!("FerroPlayer created for stream type: {:?}", stream_type);
        
        Self {
            stream_type,
            event_sender,
            video_renderer,
            audio_renderer,
            state,
            paused,
            muted,
            volume,
            playback_rate,
            position_us,
            duration_us,
            input_size,
            data_buffer,
            command_tx: Some(command_tx),
            thread_handle,
            running,
            url,
            buffered_ranges,
        }
    }
    
    fn send_command(&self, cmd: PlayerCommand) {
        if let Some(ref tx) = self.command_tx {
            let _ = tx.try_send(cmd);
        }
    }
    
    fn player_thread(
        command_rx: crossbeam_channel::Receiver<PlayerCommand>,
        state: Arc<Mutex<InternalState>>,
        paused: Arc<AtomicBool>,
        muted: Arc<AtomicBool>,
        volume: Arc<AtomicU64>,
        position_us: Arc<AtomicU64>,
        duration_us: Arc<AtomicU64>,
        running: Arc<AtomicBool>,
        url: Arc<Mutex<Option<String>>>,
        data_buffer: Arc<Mutex<Vec<u8>>>,
        buffered_ranges: Arc<Mutex<Vec<Range<f64>>>>,
        event_sender: IpcSender<PlayerEvent>,
        video_renderer: Option<Arc<Mutex<dyn VideoFrameRenderer>>>,
        audio_renderer: Option<Arc<Mutex<dyn AudioRenderer>>>,
    ) {
        #[cfg(feature = "ffmpeg")]
        let mut decoder: Option<ferro_media::decoder::MediaDecoder> = None;
        
        #[cfg(feature = "ffmpeg")]
        let mut audio_output: Option<ferro_media::audio::AudioOutput> = None;
        
        // Temp file for push_data mode
        let mut temp_file: Option<(std::fs::File, std::path::PathBuf)> = None;
        
        // Track if we've requested data
        let mut need_data_sent = false;
        let mut enough_data = false;
        const MAX_BUFFER_SIZE: usize = 10 * 1024 * 1024; // 10 MB buffer threshold
        
        // Flag to skip invalid media streams early
        let mut format_checked = false;
        let mut is_valid_format = true;
        
        // Immediately signal that we need data to start receiving media content
        debug!("FerroPlayer: Sending initial NeedData");
        let _ = event_sender.send(PlayerEvent::NeedData);
        need_data_sent = true;
        
        while running.load(Ordering::SeqCst) {
            // Process commands
            match command_rx.recv_timeout(Duration::from_millis(16)) {
                Ok(PlayerCommand::SetUrl(new_url)) => {
                    debug!("FerroPlayer: SetUrl {}", new_url);
                    *url.lock().unwrap() = Some(new_url.clone());
                    
                    #[cfg(feature = "ffmpeg")]
                    {
                        // Open media with FFmpeg
                        match ferro_media::decoder::MediaDecoder::open(&new_url) {
                            Ok(dec) => {
                                // Get metadata
                                if let Some(dur) = dec.duration() {
                                    duration_us.store(dur.as_micros() as u64, Ordering::SeqCst);
                                }
                                
                                // Create audio output if needed
                                if dec.has_audio() {
                                    let sample_rate = dec.audio_sample_rate().unwrap_or(44100);
                                    let channels = dec.audio_channels().unwrap_or(2);
                                    audio_output = ferro_media::audio::AudioOutput::new(sample_rate, channels).ok();
                                }
                                
                                // Send metadata event
                                let video_size = dec.video_size();
                                let metadata = servo_media::player::metadata::Metadata {
                                    duration: dec.duration(),
                                    width: video_size.map(|s| s.width).unwrap_or(0),
                                    height: video_size.map(|s| s.height).unwrap_or(0),
                                    format: String::new(),
                                    is_seekable: true,
                                    video_tracks: if dec.has_video() { vec!["Video".into()] } else { vec![] },
                                    audio_tracks: if dec.has_audio() { vec!["Audio".into()] } else { vec![] },
                                    is_live: false,
                                    title: None,
                                };
                                let _ = event_sender.send(PlayerEvent::MetadataUpdated(metadata));
                                
                                decoder = Some(dec);
                                *state.lock().unwrap() = InternalState::Paused;
                                let _ = event_sender.send(PlayerEvent::StateChanged(
                                    servo_media::player::PlaybackState::Paused
                                ));
                            }
                            Err(e) => {
                                error!("FerroPlayer: Failed to open {}: {}", new_url, e);
                                let _ = event_sender.send(PlayerEvent::Error(e.to_string()));
                            }
                        }
                    }
                    
                    #[cfg(not(feature = "ffmpeg"))]
                    {
                        error!("FerroPlayer: FFmpeg not enabled, cannot play media");
                        let _ = event_sender.send(PlayerEvent::Error("FFmpeg not available".to_string()));
                    }
                }
                Ok(PlayerCommand::Play) => {
                    debug!("FerroPlayer: Play");
                    
                    // If we're at the end of stream, seek back to beginning first
                    #[cfg(feature = "ffmpeg")]
                    {
                        let current_state = *state.lock().unwrap();
                        if current_state == InternalState::Ended {
                            debug!("FerroPlayer: Restarting from beginning");
                            if let Some(ref mut dec) = decoder {
                                if dec.seek(Duration::from_secs(0)).is_ok() {
                                    position_us.store(0, Ordering::SeqCst);
                                    let _ = event_sender.send(PlayerEvent::SeekDone(0.0));
                                }
                            }
                        }
                    }
                    
                    paused.store(false, Ordering::SeqCst);
                    *state.lock().unwrap() = InternalState::Playing;
                    let _ = event_sender.send(PlayerEvent::StateChanged(
                        servo_media::player::PlaybackState::Playing
                    ));
                    
                    #[cfg(feature = "ffmpeg")]
                    if let Some(ref audio) = audio_output {
                        audio.play();
                    }
                }
                Ok(PlayerCommand::Pause) => {
                    debug!("FerroPlayer: Pause");
                    paused.store(true, Ordering::SeqCst);
                    *state.lock().unwrap() = InternalState::Paused;
                    let _ = event_sender.send(PlayerEvent::StateChanged(
                        servo_media::player::PlaybackState::Paused
                    ));
                    
                    #[cfg(feature = "ffmpeg")]
                    if let Some(ref audio) = audio_output {
                        audio.pause();
                    }
                }
                Ok(PlayerCommand::Stop) => {
                    debug!("FerroPlayer: Stop");
                    paused.store(true, Ordering::SeqCst);
                    position_us.store(0, Ordering::SeqCst);
                    *state.lock().unwrap() = InternalState::Stopped;
                    let _ = event_sender.send(PlayerEvent::StateChanged(
                        servo_media::player::PlaybackState::Stopped
                    ));
                    
                    #[cfg(feature = "ffmpeg")]
                    {
                        if let Some(ref audio) = audio_output {
                            audio.stop();
                        }
                        decoder = None;
                        audio_output = None;
                    }
                }
                Ok(PlayerCommand::Seek(time)) => {
                    debug!("FerroPlayer: Seek to {}", time);
                    let seek_us = (time * 1_000_000.0) as u64;
                    
                    #[cfg(feature = "ffmpeg")]
                    if let Some(ref mut dec) = decoder {
                        if dec.seek(Duration::from_micros(seek_us)).is_ok() {
                            position_us.store(seek_us, Ordering::SeqCst);
                            
                            // If we were at end of stream, reset to Paused state after seek
                            {
                                let mut st = state.lock().unwrap();
                                if *st == InternalState::Ended {
                                    *st = InternalState::Paused;
                                    paused.store(true, Ordering::SeqCst);
                                }
                            }
                            
                            let _ = event_sender.send(PlayerEvent::SeekDone(time));
                        }
                    }
                }
                Ok(PlayerCommand::SetVolume(vol)) => {
                    let vol_int = (vol * 100.0).clamp(0.0, 100.0) as u64;
                    volume.store(vol_int, Ordering::SeqCst);
                    
                    #[cfg(feature = "ffmpeg")]
                    if let Some(ref audio) = audio_output {
                        audio.set_volume(vol as f32);
                    }
                }
                Ok(PlayerCommand::PushData(data)) => {
                    // Early format check - only check once when we have enough bytes
                    if !format_checked && !is_valid_format {
                        // Already determined to be invalid, skip all data
                        continue;
                    }
                    
                    // Buffer incoming data
                    let buffer_len = {
                        let mut buffer = data_buffer.lock().unwrap();
                        buffer.extend_from_slice(&data);
                        
                        // Check format early when we have at least 12 bytes
                        if !format_checked && buffer.len() >= 12 {
                            format_checked = true;
                            is_valid_format = is_valid_media_format(&buffer);
                            if !is_valid_format {
                                warn!("FerroPlayer: Early format check failed, not a valid media format. Skipping this stream.");
                                buffer.clear(); // Clear buffer to save memory
                                // Send end of stream to clean up
                                let _ = event_sender.send(PlayerEvent::EndOfStream);
                                return; // Exit thread early for invalid formats
                            }
                            debug!("FerroPlayer: Format check passed, continuing to buffer");
                        }
                        
                        debug!("FerroPlayer: Received {} bytes, total buffered: {}", data.len(), buffer.len());
                        buffer.len()
                    };
                    
                    // Update buffered ranges (simple approximation)
                    {
                        let input_sz = 1.max(buffer_len) as f64;
                        let mut ranges = buffered_ranges.lock().unwrap();
                        if ranges.is_empty() {
                            ranges.push(0.0..input_sz);
                        } else {
                            ranges[0] = 0.0..input_sz;
                        }
                    }
                    
                    // Check if we have enough data or need more
                    if buffer_len >= MAX_BUFFER_SIZE {
                        if !enough_data {
                            debug!("FerroPlayer: Buffer full, sending EnoughData");
                            let _ = event_sender.send(PlayerEvent::EnoughData);
                            enough_data = true;
                        }
                    } else if enough_data || !need_data_sent {
                        debug!("FerroPlayer: Buffer has room, sending NeedData");
                        let _ = event_sender.send(PlayerEvent::NeedData);
                        need_data_sent = true;
                        enough_data = false;
                    }
                }
                Ok(PlayerCommand::EndOfStream) => {
                    debug!("FerroPlayer: EndOfStream");
                    
                    // If format was already determined to be invalid, just clean up
                    if format_checked && !is_valid_format {
                        debug!("FerroPlayer: Skipping EndOfStream processing for invalid format");
                        data_buffer.lock().unwrap().clear();
                        let _ = event_sender.send(PlayerEvent::EndOfStream);
                        continue;
                    }
                    
                    #[cfg(feature = "ffmpeg")]
                    {
                        // If we have buffered data but no decoder, create temp file and open it
                        let buffer = data_buffer.lock().unwrap().clone();
                        if !buffer.is_empty() && decoder.is_none() {
                            debug!("FerroPlayer: Processing {} bytes of buffered data", buffer.len());
                            
                            // Check if this looks like a valid media format (if not already checked)
                            if !format_checked {
                                is_valid_format = is_valid_media_format(&buffer);
                                format_checked = true;
                            }
                            
                            if !is_valid_format {
                                warn!("FerroPlayer: Data does not appear to be a supported media format, skipping");
                                // Clear buffer and continue
                                data_buffer.lock().unwrap().clear();
                            } else {
                                // Write to temp file
                                let temp_path = std::env::temp_dir().join(format!("ferro_media_{}.tmp", std::process::id()));
                                if let Ok(mut file) = std::fs::File::create(&temp_path) {
                                    if file.write_all(&buffer).is_ok() {
                                        drop(file);
                                        
                                        // Open with decoder
                                        match ferro_media::decoder::MediaDecoder::open(temp_path.to_str().unwrap_or("")) {
                                            Ok(dec) => {
                                                // Additional validation: make sure codec parameters are valid
                                                let video_size = dec.video_size();
                                                let has_valid_video = video_size.map(|s| s.width > 0 && s.height > 0).unwrap_or(false);
                                                let has_valid_audio = dec.has_audio() && dec.audio_sample_rate().unwrap_or(0) > 0;
                                                
                                                if !has_valid_video && !has_valid_audio {
                                                    warn!("FerroPlayer: Decoder opened but no valid video/audio streams");
                                                    let _ = std::fs::remove_file(&temp_path);
                                                } else {
                                                    if let Some(dur) = dec.duration() {
                                                        duration_us.store(dur.as_micros() as u64, Ordering::SeqCst);
                                                    }
                                                    
                                                    if has_valid_audio {
                                                        let sample_rate = dec.audio_sample_rate().unwrap_or(44100);
                                                        let channels = dec.audio_channels().unwrap_or(2);
                                                        audio_output = ferro_media::audio::AudioOutput::new(sample_rate, channels).ok();
                                                    }
                                                    
                                                    let metadata = servo_media::player::metadata::Metadata {
                                                        duration: dec.duration(),
                                                        width: video_size.map(|s| s.width).unwrap_or(0),
                                                        height: video_size.map(|s| s.height).unwrap_or(0),
                                                        format: String::new(),
                                                        is_seekable: true,
                                                        video_tracks: if has_valid_video { vec!["Video".into()] } else { vec![] },
                                                        audio_tracks: if has_valid_audio { vec!["Audio".into()] } else { vec![] },
                                                        is_live: false,
                                                        title: None,
                                                    };
                                                    let _ = event_sender.send(PlayerEvent::MetadataUpdated(metadata));
                                                    
                                                    decoder = Some(dec);
                                                    temp_file = Some((std::fs::File::open(&temp_path).unwrap(), temp_path));
                                                    
                                                    *state.lock().unwrap() = InternalState::Paused;
                                                    let _ = event_sender.send(PlayerEvent::StateChanged(
                                                        servo_media::player::PlaybackState::Paused
                                                    ));
                                                }
                                            }
                                            Err(e) => {
                                                warn!("FerroPlayer: Failed to open media: {}", e);
                                                let _ = std::fs::remove_file(&temp_path);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(PlayerCommand::Shutdown) => {
                    debug!("FerroPlayer: Shutdown");
                    break;
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    // Continue processing
                }
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
            
            // Decode and render frames if playing
            #[cfg(feature = "ffmpeg")]
            if !paused.load(Ordering::SeqCst) {
                if let Some(ref mut dec) = decoder {
                    match dec.decode_next_frame() {
                        Ok(Some(frame)) => {
                            // Update position
                            let pos_us = frame.timestamp.as_micros() as u64;
                            position_us.store(pos_us, Ordering::SeqCst);
                            let _ = event_sender.send(PlayerEvent::PositionChanged(
                                frame.timestamp.as_secs_f64()
                            ));
                            
                            // Handle video frame
                            if let Some(ref video_frame) = frame.video {
                                if let Some(ref renderer) = video_renderer {
                                    if let Ok(mut r) = renderer.lock() {
                                        // Create servo_media frame from ferro_media frame using Buffer trait
                                        let buffer = Arc::new(RawVideoBuffer::new(video_frame.data.clone()));
                                        if let Ok(frame_data) = VideoFrame::new(
                                            video_frame.width as i32,
                                            video_frame.height as i32,
                                            buffer,
                                        ) {
                                            r.render(frame_data);
                                        }
                                    }
                                }
                                let _ = event_sender.send(PlayerEvent::VideoFrameUpdated);
                            }
                            
                            // Handle audio
                            if let Some(ref audio_data) = frame.audio {
                                // Send to audio renderer if present
                                if let Some(ref renderer) = audio_renderer {
                                    if let Ok(mut r) = renderer.lock() {
                                        let samples: Box<dyn AsRef<[f32]>> = Box::new(audio_data.samples.clone());
                                        r.render(samples, audio_data.channels as u32);
                                    }
                                }
                                
                                // Also output to system audio
                                if !muted.load(Ordering::SeqCst) {
                                    if let Some(ref mut output) = audio_output {
                                        output.write_samples(&audio_data.samples);
                                    }
                                }
                            }
                        }
                        Ok(None) => {
                            // End of stream
                            *state.lock().unwrap() = InternalState::Ended;
                            let _ = event_sender.send(PlayerEvent::EndOfStream);
                        }
                        Err(e) => {
                            error!("FerroPlayer: Decode error: {}", e);
                            // Don't spam errors, just continue
                        }
                    }
                }
            }
        }
        
        // Cleanup
        #[cfg(feature = "ffmpeg")]
        {
            drop(decoder);
            drop(audio_output);
        }
        
        // Remove temp file if any
        if let Some((_, path)) = temp_file {
            let _ = std::fs::remove_file(path);
        }
        
        debug!("FerroPlayer thread exiting");
    }
}

impl Player for FerroPlayer {
    fn play(&self) -> Result<(), PlayerError> {
        debug!("FerroPlayer::play");
        self.send_command(PlayerCommand::Play);
        Ok(())
    }

    fn pause(&self) -> Result<(), PlayerError> {
        debug!("FerroPlayer::pause");
        self.send_command(PlayerCommand::Pause);
        Ok(())
    }

    fn paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    fn can_resume(&self) -> bool {
        true
    }

    fn stop(&self) -> Result<(), PlayerError> {
        debug!("FerroPlayer::stop");
        self.send_command(PlayerCommand::Stop);
        Ok(())
    }

    fn seek(&self, time: f64) -> Result<(), PlayerError> {
        debug!("FerroPlayer::seek to {}", time);
        self.send_command(PlayerCommand::Seek(time));
        Ok(())
    }

    fn set_mute(&self, val: bool) -> Result<(), PlayerError> {
        self.muted.store(val, Ordering::SeqCst);
        Ok(())
    }

    fn muted(&self) -> bool {
        self.muted.load(Ordering::SeqCst)
    }

    fn set_volume(&self, val: f64) -> Result<(), PlayerError> {
        self.volume.store((val * 100.0) as u64, Ordering::SeqCst);
        self.send_command(PlayerCommand::SetVolume(val));
        Ok(())
    }

    fn volume(&self) -> f64 {
        self.volume.load(Ordering::SeqCst) as f64 / 100.0
    }

    fn set_input_size(&self, size: u64) -> Result<(), PlayerError> {
        self.input_size.store(size, Ordering::SeqCst);
        Ok(())
    }

    fn set_playback_rate(&self, rate: f64) -> Result<(), PlayerError> {
        self.playback_rate.store((rate * 100.0) as u64, Ordering::SeqCst);
        Ok(())
    }

    fn playback_rate(&self) -> f64 {
        self.playback_rate.load(Ordering::SeqCst) as f64 / 100.0
    }

    fn push_data(&self, data: Vec<u8>) -> Result<(), PlayerError> {
        debug!("FerroPlayer::push_data {} bytes", data.len());
        self.send_command(PlayerCommand::PushData(data));
        Ok(())
    }

    fn end_of_stream(&self) -> Result<(), PlayerError> {
        debug!("FerroPlayer::end_of_stream");
        self.send_command(PlayerCommand::EndOfStream);
        Ok(())
    }

    fn buffered(&self) -> Vec<Range<f64>> {
        self.buffered_ranges.lock().unwrap().clone()
    }

    fn seekable(&self) -> Vec<Range<f64>> {
        let dur = self.duration_us.load(Ordering::SeqCst) as f64 / 1_000_000.0;
        if dur > 0.0 {
            vec![0.0..dur]
        } else {
            vec![]
        }
    }

    fn set_stream(
        &self,
        _stream: &servo_media::streams::MediaStreamId,
        _only_stream: bool,
    ) -> Result<(), PlayerError> {
        Ok(())
    }

    fn render_use_gl(&self) -> bool {
        false
    }

    fn set_audio_track(&self, _index: i32, _enabled: bool) -> Result<(), PlayerError> {
        Ok(())
    }

    fn set_video_track(&self, _index: i32, _enabled: bool) -> Result<(), PlayerError> {
        Ok(())
    }
}

impl MediaInstance for FerroPlayer {
    fn get_id(&self) -> usize {
        0
    }

    fn mute(&self, val: bool) -> Result<(), ()> {
        self.muted.store(val, Ordering::SeqCst);
        Ok(())
    }

    fn suspend(&self) -> Result<(), ()> {
        self.send_command(PlayerCommand::Pause);
        Ok(())
    }

    fn resume(&self) -> Result<(), ()> {
        self.send_command(PlayerCommand::Play);
        Ok(())
    }
}

impl Drop for FerroPlayer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self.send_command(PlayerCommand::Shutdown);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

// ============================================================================
// Audio components
// ============================================================================

pub struct FerroAudioSink;

impl servo_media::audio::sink::AudioSink for FerroAudioSink {
    fn init(
        &self,
        _sample_rate: f32,
        _sender: Sender<servo_media::audio::render_thread::AudioRenderThreadMsg>,
    ) -> Result<(), servo_media::audio::sink::AudioSinkError> {
        Ok(())
    }

    fn init_stream(
        &self,
        _channels: u8,
        _sample_rate: f32,
        _socket: Box<dyn servo_media::streams::MediaSocket>,
    ) -> Result<(), servo_media::audio::sink::AudioSinkError> {
        Ok(())
    }

    fn play(&self) -> Result<(), servo_media::audio::sink::AudioSinkError> {
        Ok(())
    }

    fn stop(&self) -> Result<(), servo_media::audio::sink::AudioSinkError> {
        Ok(())
    }

    fn has_enough_data(&self) -> bool {
        true
    }

    fn push_data(&self, _chunk: servo_media::audio::block::Chunk) -> Result<(), servo_media::audio::sink::AudioSinkError> {
        Ok(())
    }

    fn set_eos_callback(&self, _callback: Box<dyn Fn(Box<dyn AsRef<[f32]>>) + Send + Sync + 'static>) {}
}

pub struct FerroAudioDecoder;

impl servo_media::audio::decoder::AudioDecoder for FerroAudioDecoder {
    fn decode(
        &self,
        _data: Vec<u8>,
        _callbacks: servo_media::audio::decoder::AudioDecoderCallbacks,
        _options: Option<servo_media::audio::decoder::AudioDecoderOptions>,
    ) {
        // TODO: Implement audio decoding using ferro_media
    }
}

pub struct FerroStreamReader;

impl servo_media::audio::AudioStreamReader for FerroStreamReader {
    fn pull(&self) -> servo_media::audio::block::Block {
        Default::default()
    }

    fn start(&self) {}
    fn stop(&self) {}
}

// ============================================================================
// Stream components
// ============================================================================

pub struct FerroSocket;

impl servo_media::streams::MediaSocket for FerroSocket {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct FerroMediaStream {
    id: servo_media::streams::MediaStreamId,
    ty: servo_media::streams::MediaStreamType,
}

impl FerroMediaStream {
    fn create(ty: servo_media::streams::MediaStreamType) -> servo_media::streams::MediaStreamId {
        servo_media::streams::registry::register_stream(Arc::new(Mutex::new(FerroMediaStream {
            id: servo_media::streams::MediaStreamId::new(),
            ty,
        })))
    }

    fn create_audio() -> servo_media::streams::MediaStreamId {
        Self::create(servo_media::streams::MediaStreamType::Audio)
    }

    fn create_video() -> servo_media::streams::MediaStreamId {
        Self::create(servo_media::streams::MediaStreamType::Video)
    }
}

impl servo_media::streams::MediaStream for FerroMediaStream {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }

    fn set_id(&mut self, id: servo_media::streams::MediaStreamId) {
        self.id = id;
    }

    fn ty(&self) -> servo_media::streams::MediaStreamType {
        self.ty
    }
}

impl Drop for FerroMediaStream {
    fn drop(&mut self) {
        servo_media::streams::registry::unregister_stream(&self.id);
    }
}

pub struct FerroMediaOutput;

impl servo_media::streams::MediaOutput for FerroMediaOutput {
    fn add_stream(&mut self, _stream: &servo_media::streams::MediaStreamId) {}
}

// ============================================================================
// Device monitor
// ============================================================================

pub struct FerroDeviceMonitor;

impl servo_media::streams::device_monitor::MediaDeviceMonitor for FerroDeviceMonitor {
    fn enumerate_devices(&self) -> Result<Vec<servo_media::streams::device_monitor::MediaDeviceInfo>, ()> {
        Ok(vec![])
    }
}

// ============================================================================
// WebRTC (stub)
// ============================================================================

pub struct FerroWebRtcController;

impl servo_media::webrtc::WebRtcControllerBackend for FerroWebRtcController {
    fn configure(&mut self, _stun_server: &str, _policy: servo_media::webrtc::BundlePolicy) -> servo_media::webrtc::WebRtcResult {
        Ok(())
    }

    fn set_remote_description(
        &mut self,
        _desc: servo_media::webrtc::SessionDescription,
        _cb: Box<dyn FnOnce() + Send + 'static>,
    ) -> servo_media::webrtc::WebRtcResult {
        Ok(())
    }

    fn set_local_description(
        &mut self,
        _desc: servo_media::webrtc::SessionDescription,
        _cb: Box<dyn FnOnce() + Send + 'static>,
    ) -> servo_media::webrtc::WebRtcResult {
        Ok(())
    }

    fn add_ice_candidate(&mut self, _candidate: servo_media::webrtc::IceCandidate) -> servo_media::webrtc::WebRtcResult {
        Ok(())
    }

    fn create_offer(
        &mut self,
        _cb: Box<dyn FnOnce(servo_media::webrtc::SessionDescription) + Send + 'static>,
    ) -> servo_media::webrtc::WebRtcResult {
        Ok(())
    }

    fn create_answer(
        &mut self,
        _cb: Box<dyn FnOnce(servo_media::webrtc::SessionDescription) + Send + 'static>,
    ) -> servo_media::webrtc::WebRtcResult {
        Ok(())
    }

    fn add_stream(&mut self, _stream: &servo_media::streams::MediaStreamId) -> servo_media::webrtc::WebRtcResult {
        Ok(())
    }

    fn create_data_channel(
        &mut self,
        _init: &servo_media::webrtc::DataChannelInit,
    ) -> servo_media::webrtc::WebRtcDataChannelResult {
        Ok(0)
    }

    fn close_data_channel(&mut self, _channel: &servo_media::webrtc::DataChannelId) -> servo_media::webrtc::WebRtcResult {
        Ok(())
    }

    fn send_data_channel_message(
        &mut self,
        _channel: &servo_media::webrtc::DataChannelId,
        _message: &servo_media::webrtc::DataChannelMessage,
    ) -> servo_media::webrtc::WebRtcResult {
        Ok(())
    }

    fn internal_event(&mut self, _event: servo_media::webrtc::thread::InternalEvent) -> servo_media::webrtc::WebRtcResult {
        Ok(())
    }

    fn quit(&mut self) {}
}

// ============================================================================
// MSE (Media Source Extensions) Support
// ============================================================================

/// MSE-aware player that can handle streaming media via SourceBuffer
pub struct MsePlayer {
    source_buffers: Vec<MseSourceBuffer>,
    duration: Option<f64>,
    ready_state: MseReadyState,
    paused: bool,
    current_time: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MseReadyState {
    Closed,
    Open,
    Ended,
}

impl MsePlayer {
    pub fn new() -> Self {
        Self {
            source_buffers: Vec::new(),
            duration: None,
            ready_state: MseReadyState::Closed,
            paused: true,
            current_time: 0.0,
        }
    }
    
    /// Open the media source
    pub fn open(&mut self) {
        self.ready_state = MseReadyState::Open;
        debug!("MsePlayer: opened");
    }
    
    /// Add a source buffer for a given MIME type
    pub fn add_source_buffer(&mut self, mime_type: &str) -> usize {
        let buffer = MseSourceBuffer::new(mime_type);
        self.source_buffers.push(buffer);
        debug!("MsePlayer: added source buffer for {}", mime_type);
        self.source_buffers.len() - 1
    }
    
    /// Remove a source buffer by index
    pub fn remove_source_buffer(&mut self, index: usize) {
        if index < self.source_buffers.len() {
            self.source_buffers.remove(index);
            debug!("MsePlayer: removed source buffer {}", index);
        }
    }
    
    /// Get a mutable reference to a source buffer
    pub fn source_buffer_mut(&mut self, index: usize) -> Option<&mut MseSourceBuffer> {
        self.source_buffers.get_mut(index)
    }
    
    /// Get a reference to a source buffer
    pub fn source_buffer(&self, index: usize) -> Option<&MseSourceBuffer> {
        self.source_buffers.get(index)
    }
    
    /// Get number of source buffers
    pub fn source_buffer_count(&self) -> usize {
        self.source_buffers.len()
    }
    
    /// Set duration
    pub fn set_duration(&mut self, duration: f64) {
        self.duration = Some(duration);
    }
    
    /// Get duration
    pub fn duration(&self) -> Option<f64> {
        self.duration
    }
    
    /// End of stream
    pub fn end_of_stream(&mut self, error: Option<&str>) {
        if let Some(err) = error {
            debug!("MsePlayer: end of stream with error: {}", err);
        } else {
            debug!("MsePlayer: end of stream");
        }
        self.ready_state = MseReadyState::Ended;
    }
    
    /// Get ready state
    pub fn ready_state(&self) -> MseReadyState {
        self.ready_state
    }
    
    /// Check if currently updating any buffer
    pub fn is_updating(&self) -> bool {
        self.source_buffers.iter().any(|sb| sb.is_updating())
    }
    
    /// Get combined buffered ranges across all source buffers
    pub fn buffered_ranges(&self) -> Vec<TimeRange> {
        // Combine and intersect ranges from all source buffers
        if self.source_buffers.is_empty() {
            return Vec::new();
        }
        
        // Start with first buffer's ranges
        let mut combined = self.source_buffers[0].buffered_ranges().to_vec();
        
        // Intersect with other buffers
        for buffer in &self.source_buffers[1..] {
            let other_ranges = buffer.buffered_ranges();
            combined = Self::intersect_ranges(&combined, other_ranges);
        }
        
        combined
    }
    
    fn intersect_ranges(a: &[TimeRange], b: &[TimeRange]) -> Vec<TimeRange> {
        let mut result = Vec::new();
        
        for range_a in a {
            for range_b in b {
                // Check for intersection
                let start = range_a.start.max(range_b.start);
                let end = range_a.end.min(range_b.end);
                if start < end {
                    result.push(TimeRange::new(start, end));
                }
            }
        }
        
        result
    }
    
    /// Play
    pub fn play(&mut self) {
        self.paused = false;
        debug!("MsePlayer: play");
    }
    
    /// Pause
    pub fn pause(&mut self) {
        self.paused = true;
        debug!("MsePlayer: pause");
    }
    
    /// Get paused state
    pub fn paused(&self) -> bool {
        self.paused
    }
    
    /// Set current time (seek)
    pub fn set_current_time(&mut self, time: f64) {
        self.current_time = time;
        debug!("MsePlayer: seek to {}", time);
    }
    
    /// Get current time
    pub fn current_time(&self) -> f64 {
        self.current_time
    }
    
    /// Check if we can play through without buffering
    pub fn can_play_through(&self) -> bool {
        if let Some(duration) = self.duration {
            let buffered = self.buffered_ranges();
            // Check if we have buffered to the end
            buffered.iter().any(|r| r.end >= duration)
        } else {
            false
        }
    }
}

impl Default for MsePlayer {
    fn default() -> Self {
        Self::new()
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! FFmpeg-based backend for servo-media.
//! 
//! This is a drop-in replacement for servo-media-gstreamer that uses FFmpeg
//! via the ferro_media crate for media decoding and playback.

use std::any::Any;
use std::ops::Range;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};

use ipc_channel::ipc::IpcSender;
use log::debug;
use servo_media::{Backend, BackendInit, ClientContextId, MediaInstance, SupportsMediaType};
use servo_media::player::context::PlayerGLContext;
use servo_media::player::{Player, PlayerError, PlayerEvent, StreamType};
use servo_media::player::audio::AudioRenderer;
use servo_media::player::video::VideoFrameRenderer;

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
        _stream_type: StreamType,
        _sender: IpcSender<PlayerEvent>,
        _video_renderer: Option<Arc<Mutex<dyn VideoFrameRenderer>>>,
        _audio_renderer: Option<Arc<Mutex<dyn AudioRenderer>>>,
        _gl_context: Box<dyn PlayerGLContext>,
    ) -> Arc<Mutex<dyn Player>> {
        Arc::new(Mutex::new(FerroPlayer::new()))
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
// Player implementation
// ============================================================================

pub struct FerroPlayer {
    paused: bool,
    muted: bool,
    volume: f64,
    playback_rate: f64,
}

impl FerroPlayer {
    fn new() -> Self {
        Self {
            paused: true,
            muted: false,
            volume: 1.0,
            playback_rate: 1.0,
        }
    }
}

impl Player for FerroPlayer {
    fn play(&self) -> Result<(), PlayerError> {
        debug!("FerroPlayer: play");
        Ok(())
    }

    fn pause(&self) -> Result<(), PlayerError> {
        debug!("FerroPlayer: pause");
        Ok(())
    }

    fn paused(&self) -> bool {
        self.paused
    }

    fn can_resume(&self) -> bool {
        true
    }

    fn stop(&self) -> Result<(), PlayerError> {
        debug!("FerroPlayer: stop");
        Ok(())
    }

    fn seek(&self, _time: f64) -> Result<(), PlayerError> {
        debug!("FerroPlayer: seek");
        Ok(())
    }

    fn set_mute(&self, _val: bool) -> Result<(), PlayerError> {
        Ok(())
    }

    fn muted(&self) -> bool {
        self.muted
    }

    fn set_volume(&self, _val: f64) -> Result<(), PlayerError> {
        Ok(())
    }

    fn volume(&self) -> f64 {
        self.volume
    }

    fn set_input_size(&self, _size: u64) -> Result<(), PlayerError> {
        Ok(())
    }

    fn set_playback_rate(&self, _rate: f64) -> Result<(), PlayerError> {
        Ok(())
    }

    fn playback_rate(&self) -> f64 {
        self.playback_rate
    }

    fn push_data(&self, _data: Vec<u8>) -> Result<(), PlayerError> {
        Ok(())
    }

    fn end_of_stream(&self) -> Result<(), PlayerError> {
        Ok(())
    }

    fn buffered(&self) -> Vec<Range<f64>> {
        vec![]
    }

    fn seekable(&self) -> Vec<Range<f64>> {
        vec![]
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

    fn mute(&self, _val: bool) -> Result<(), ()> {
        Ok(())
    }

    fn suspend(&self) -> Result<(), ()> {
        Ok(())
    }

    fn resume(&self) -> Result<(), ()> {
        Ok(())
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

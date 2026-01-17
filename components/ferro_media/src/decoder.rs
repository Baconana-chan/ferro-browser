/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! FFmpeg-based media decoder

use std::time::Duration;

use euclid::default::Size2D;
use ffmpeg_next as ffmpeg;
use ffmpeg_next::format::{input, Pixel};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::{context::Context as ScalerContext, flag::Flags};
use ffmpeg_next::util::frame::video::Video as FfmpegVideoFrame;
use ffmpeg_next::util::frame::audio::Audio as FfmpegAudioFrame;

use crate::error::{MediaError, MediaResult};
use crate::video::VideoFrame;

/// Audio data from a decoded frame
#[derive(Debug, Clone)]
pub struct AudioData {
    pub samples: Vec<f32>,
    pub channels: u32,
    pub sample_rate: u32,
}

/// A decoded frame containing video and/or audio data
#[derive(Debug, Clone)]
pub struct DecodedFrame {
    pub timestamp: Duration,
    pub video: Option<VideoFrame>,
    pub audio: Option<AudioData>,
}

/// Media decoder using FFmpeg
pub struct MediaDecoder {
    input_context: ffmpeg::format::context::Input,
    video_stream_index: Option<usize>,
    audio_stream_index: Option<usize>,
    video_decoder: Option<ffmpeg::decoder::Video>,
    audio_decoder: Option<ffmpeg::decoder::Audio>,
    scaler: Option<ScalerContext>,
    duration: Option<Duration>,
    video_size: Option<Size2D<u32>>,
    audio_sample_rate: Option<u32>,
    audio_channels: Option<u32>,
    time_base_video: f64,
    time_base_audio: f64,
}

impl MediaDecoder {
    /// Open a media file or URL
    pub fn open(url: &str) -> MediaResult<Self> {
        let input_context = input(&url)?;
        
        let mut video_stream_index = None;
        let mut audio_stream_index = None;
        let mut video_decoder = None;
        let mut audio_decoder = None;
        let mut scaler = None;
        let mut duration = None;
        let mut video_size = None;
        let mut audio_sample_rate = None;
        let mut audio_channels = None;
        let mut time_base_video = 0.0;
        let mut time_base_audio = 0.0;
        
        // Get duration from container
        let container_duration = input_context.duration();
        if container_duration > 0 {
            duration = Some(Duration::from_micros(container_duration as u64));
        }
        
        // Find video stream
        if let Some(stream) = input_context.streams().best(Type::Video) {
            video_stream_index = Some(stream.index());
            time_base_video = f64::from(stream.time_base());
            
            let context_decoder = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
            let decoder = context_decoder.decoder().video()?;
            
            video_size = Some(Size2D::new(decoder.width(), decoder.height()));
            
            // Create scaler for RGBA output
            scaler = Some(ScalerContext::get(
                decoder.format(),
                decoder.width(),
                decoder.height(),
                Pixel::RGBA,
                decoder.width(),
                decoder.height(),
                Flags::BILINEAR,
            )?);
            
            video_decoder = Some(decoder);
        }
        
        // Find audio stream
        if let Some(stream) = input_context.streams().best(Type::Audio) {
            audio_stream_index = Some(stream.index());
            time_base_audio = f64::from(stream.time_base());
            
            let context_decoder = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
            let decoder = context_decoder.decoder().audio()?;
            
            audio_sample_rate = Some(decoder.rate());
            audio_channels = Some(decoder.channels() as u32);
            
            audio_decoder = Some(decoder);
        }
        
        if video_stream_index.is_none() && audio_stream_index.is_none() {
            return Err(MediaError::UnsupportedFormat("No audio or video streams found".to_string()));
        }
        
        Ok(Self {
            input_context,
            video_stream_index,
            audio_stream_index,
            video_decoder,
            audio_decoder,
            scaler,
            duration,
            video_size,
            audio_sample_rate,
            audio_channels,
            time_base_video,
            time_base_audio,
        })
    }
    
    /// Get media duration
    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }
    
    /// Get video size
    pub fn video_size(&self) -> Option<Size2D<u32>> {
        self.video_size
    }
    
    /// Check if has video
    pub fn has_video(&self) -> bool {
        self.video_stream_index.is_some()
    }
    
    /// Check if has audio
    pub fn has_audio(&self) -> bool {
        self.audio_stream_index.is_some()
    }
    
    /// Get audio sample rate
    pub fn audio_sample_rate(&self) -> Option<u32> {
        self.audio_sample_rate
    }
    
    /// Get audio channels
    pub fn audio_channels(&self) -> Option<u32> {
        self.audio_channels
    }
    
    /// Seek to a position
    pub fn seek(&mut self, position: Duration) -> MediaResult<()> {
        let timestamp = position.as_micros() as i64;
        self.input_context.seek(timestamp, ..timestamp)?;
        
        // Flush decoders
        if let Some(ref mut decoder) = self.video_decoder {
            decoder.flush();
        }
        if let Some(ref mut decoder) = self.audio_decoder {
            decoder.flush();
        }
        
        Ok(())
    }
    
    /// Decode the next frame
    pub fn decode_next_frame(&mut self) -> MediaResult<Option<DecodedFrame>> {
        loop {
            // Try to get next packet
            let Some((stream, packet)) = self.input_context.packets().next() else {
                return Ok(None); // End of stream
            };
            
            let stream_index = stream.index();
            
            // Process video packet
            if Some(stream_index) == self.video_stream_index {
                if let Some(ref mut decoder) = self.video_decoder {
                    decoder.send_packet(&packet)?;
                    
                    let mut decoded_frame = FfmpegVideoFrame::empty();
                    if decoder.receive_frame(&mut decoded_frame).is_ok() {
                        // Calculate timestamp
                        let pts = decoded_frame.pts().unwrap_or(0);
                        let timestamp = Duration::from_secs_f64(pts as f64 * self.time_base_video);
                        
                        // Convert to RGBA
                        let mut rgb_frame = FfmpegVideoFrame::empty();
                        if let Some(ref mut scaler) = self.scaler {
                            scaler.run(&decoded_frame, &mut rgb_frame)?;
                            
                            let video_frame = VideoFrame {
                                width: rgb_frame.width(),
                                height: rgb_frame.height(),
                                data: rgb_frame.data(0).to_vec(),
                                stride: rgb_frame.stride(0) as u32,
                                timestamp,
                            };
                            
                            return Ok(Some(DecodedFrame {
                                timestamp,
                                video: Some(video_frame),
                                audio: None,
                            }));
                        }
                    }
                }
            }
            
            // Process audio packet
            if Some(stream_index) == self.audio_stream_index {
                if let Some(ref mut decoder) = self.audio_decoder {
                    decoder.send_packet(&packet)?;
                    
                    let mut decoded_frame = FfmpegAudioFrame::empty();
                    if decoder.receive_frame(&mut decoded_frame).is_ok() {
                        // Calculate timestamp
                        let pts = decoded_frame.pts().unwrap_or(0);
                        let timestamp = Duration::from_secs_f64(pts as f64 * self.time_base_audio);
                        
                        // Convert audio to f32 samples
                        let samples = convert_audio_to_f32(&decoded_frame);
                        
                        let audio_data = AudioData {
                            samples,
                            channels: self.audio_channels.unwrap_or(2),
                            sample_rate: self.audio_sample_rate.unwrap_or(44100),
                        };
                        
                        return Ok(Some(DecodedFrame {
                            timestamp,
                            video: None,
                            audio: Some(audio_data),
                        }));
                    }
                }
            }
        }
    }
}

/// Convert FFmpeg audio frame to f32 samples
fn convert_audio_to_f32(frame: &FfmpegAudioFrame) -> Vec<f32> {
    let format = frame.format();
    let samples = frame.samples();
    let channels = frame.channels() as usize;
    let mut output = Vec::with_capacity(samples * channels);
    
    // Handle different sample formats
    match format {
        ffmpeg::format::Sample::F32(packed) if packed == ffmpeg::format::sample::Type::Packed => {
            let data = frame.data(0);
            for chunk in data.chunks_exact(4) {
                let sample = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                output.push(sample);
            }
        }
        ffmpeg::format::Sample::F32(_) => {
            // Planar F32
            for ch in 0..channels {
                let data = frame.data(ch);
                for i in 0..samples {
                    let offset = i * 4;
                    if offset + 4 <= data.len() {
                        let sample = f32::from_le_bytes([
                            data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
                        ]);
                        output.push(sample);
                    }
                }
            }
        }
        ffmpeg::format::Sample::I16(packed) if packed == ffmpeg::format::sample::Type::Packed => {
            let data = frame.data(0);
            for chunk in data.chunks_exact(2) {
                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                output.push(sample as f32 / 32768.0);
            }
        }
        ffmpeg::format::Sample::I16(_) => {
            // Planar I16
            for ch in 0..channels {
                let data = frame.data(ch);
                for i in 0..samples {
                    let offset = i * 2;
                    if offset + 2 <= data.len() {
                        let sample = i16::from_le_bytes([data[offset], data[offset + 1]]);
                        output.push(sample as f32 / 32768.0);
                    }
                }
            }
        }
        ffmpeg::format::Sample::I32(packed) if packed == ffmpeg::format::sample::Type::Packed => {
            let data = frame.data(0);
            for chunk in data.chunks_exact(4) {
                let sample = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                output.push(sample as f32 / 2147483648.0);
            }
        }
        _ => {
            // Fallback: silence
            output.resize(samples * channels, 0.0);
        }
    }
    
    output
}

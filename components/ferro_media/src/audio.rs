/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Audio output using cpal/rodio

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crossbeam_channel::{Sender, Receiver, bounded};
use parking_lot::Mutex;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};

use crate::error::{MediaError, MediaResult};

/// Audio output manager
pub struct AudioOutput {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    sink: Arc<Mutex<Sink>>,
    sample_tx: Sender<Vec<f32>>,
    sample_rate: u32,
    channels: u32,
    running: Arc<AtomicBool>,
}

impl AudioOutput {
    /// Create a new audio output
    pub fn new(sample_rate: u32, channels: u32) -> MediaResult<Self> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| MediaError::Playback(format!("Failed to open audio output: {}", e)))?;
        
        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| MediaError::Playback(format!("Failed to create audio sink: {}", e)))?;
        
        let (sample_tx, sample_rx) = bounded::<Vec<f32>>(32);
        let running = Arc::new(AtomicBool::new(true));
        
        // Create audio source from channel
        let source = ChannelSource {
            sample_rx,
            current_samples: Vec::new(),
            current_pos: 0,
            sample_rate,
            channels: channels as u16,
            running: Arc::clone(&running),
        };
        
        sink.append(source);
        
        Ok(Self {
            _stream: stream,
            _stream_handle: stream_handle,
            sink: Arc::new(Mutex::new(sink)),
            sample_tx,
            sample_rate,
            channels,
            running,
        })
    }
    
    /// Write audio samples
    pub fn write_samples(&mut self, samples: &[f32]) {
        let _ = self.sample_tx.try_send(samples.to_vec());
    }
    
    /// Set volume (0.0 - 1.0)
    pub fn set_volume(&self, volume: f32) {
        self.sink.lock().set_volume(volume.clamp(0.0, 1.0));
    }
    
    /// Pause playback
    pub fn pause(&self) {
        self.sink.lock().pause();
    }
    
    /// Resume playback
    pub fn play(&self) {
        self.sink.lock().play();
    }
    
    /// Stop playback
    pub fn stop(&self) {
        self.sink.lock().stop();
    }
    
    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    
    /// Get number of channels
    pub fn channels(&self) -> u32 {
        self.channels
    }
}

impl Drop for AudioOutput {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self.sink.lock().stop();
    }
}

/// Audio source that reads from a channel
struct ChannelSource {
    sample_rx: Receiver<Vec<f32>>,
    current_samples: Vec<f32>,
    current_pos: usize,
    sample_rate: u32,
    channels: u16,
    running: Arc<AtomicBool>,
}

impl Iterator for ChannelSource {
    type Item = f32;
    
    fn next(&mut self) -> Option<Self::Item> {
        if !self.running.load(Ordering::SeqCst) {
            return None;
        }
        
        // If we've exhausted current samples, try to get more
        if self.current_pos >= self.current_samples.len() {
            match self.sample_rx.try_recv() {
                Ok(samples) => {
                    self.current_samples = samples;
                    self.current_pos = 0;
                }
                Err(_) => {
                    // No samples available, return silence
                    return Some(0.0);
                }
            }
        }
        
        if self.current_pos < self.current_samples.len() {
            let sample = self.current_samples[self.current_pos];
            self.current_pos += 1;
            Some(sample)
        } else {
            Some(0.0) // Silence
        }
    }
}

impl Source for ChannelSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    
    fn channels(&self) -> u16 {
        self.channels
    }
    
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    
    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

/// Audio resampler for converting between sample rates
pub struct AudioResampler {
    from_rate: u32,
    to_rate: u32,
}

impl AudioResampler {
    /// Create a new resampler
    pub fn new(from_rate: u32, to_rate: u32) -> Self {
        Self { from_rate, to_rate }
    }
    
    /// Resample audio data using linear interpolation
    pub fn resample(&self, input: &[f32], channels: u32) -> Vec<f32> {
        if self.from_rate == self.to_rate {
            return input.to_vec();
        }
        
        let ratio = self.to_rate as f64 / self.from_rate as f64;
        let input_frames = input.len() / channels as usize;
        let output_frames = (input_frames as f64 * ratio) as usize;
        let mut output = Vec::with_capacity(output_frames * channels as usize);
        
        for i in 0..output_frames {
            let src_pos = i as f64 / ratio;
            let src_idx = src_pos as usize;
            let frac = src_pos - src_idx as f64;
            
            for ch in 0..channels as usize {
                let idx1 = src_idx * channels as usize + ch;
                let idx2 = (src_idx + 1) * channels as usize + ch;
                
                let s1 = input.get(idx1).copied().unwrap_or(0.0);
                let s2 = input.get(idx2).copied().unwrap_or(s1);
                
                // Linear interpolation
                let sample = s1 + (s2 - s1) * frac as f32;
                output.push(sample);
            }
        }
        
        output
    }
}

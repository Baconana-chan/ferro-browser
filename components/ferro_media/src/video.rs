/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Video frame handling

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// A decoded video frame in RGBA format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFrame {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// RGBA pixel data
    pub data: Vec<u8>,
    /// Stride (bytes per row)
    pub stride: u32,
    /// Presentation timestamp
    pub timestamp: Duration,
}

impl VideoFrame {
    /// Create a new video frame
    pub fn new(width: u32, height: u32, data: Vec<u8>, timestamp: Duration) -> Self {
        Self {
            width,
            height,
            stride: width * 4, // RGBA
            data,
            timestamp,
        }
    }
    
    /// Get frame size in bytes
    pub fn size_bytes(&self) -> usize {
        self.data.len()
    }
    
    /// Get pixel at coordinates
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        
        let offset = (y * self.stride + x * 4) as usize;
        if offset + 4 <= self.data.len() {
            Some([
                self.data[offset],
                self.data[offset + 1],
                self.data[offset + 2],
                self.data[offset + 3],
            ])
        } else {
            None
        }
    }
}

/// Video frame pool for reusing frame buffers
pub struct VideoFramePool {
    frames: Vec<VideoFrame>,
    width: u32,
    height: u32,
}

impl VideoFramePool {
    /// Create a new frame pool
    pub fn new(width: u32, height: u32, capacity: usize) -> Self {
        let mut frames = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            let data = vec![0u8; (width * height * 4) as usize];
            frames.push(VideoFrame::new(width, height, data, Duration::ZERO));
        }
        
        Self {
            frames,
            width,
            height,
        }
    }
    
    /// Get a frame from the pool
    pub fn get(&mut self) -> Option<VideoFrame> {
        self.frames.pop()
    }
    
    /// Return a frame to the pool
    pub fn put(&mut self, mut frame: VideoFrame) {
        // Only return if dimensions match
        if frame.width == self.width && frame.height == self.height {
            frame.timestamp = Duration::ZERO;
            self.frames.push(frame);
        }
    }
    
    /// Get pool capacity
    pub fn capacity(&self) -> usize {
        self.frames.capacity()
    }
    
    /// Get available frames count
    pub fn available(&self) -> usize {
        self.frames.len()
    }
}

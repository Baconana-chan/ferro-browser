/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Media Source Extensions (MSE) support for ferro_media
//!
//! This module provides segment parsing and buffering for MSE streams.
//! It handles initialization segments and media segments for both
//! ISOBMFF (MP4) and WebM containers.

use std::collections::VecDeque;
use std::io::{Cursor, Read};
use std::ops::Range;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use log::{debug, warn};

use crate::error::{MediaError, MediaResult};
use crate::video::VideoFrame;

/// Segment types in MSE
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentType {
    /// Initialization segment (moov for MP4, etc)
    Initialization,
    /// Media segment containing actual audio/video data
    Media,
    /// Unknown segment type
    Unknown,
}

/// Container format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerFormat {
    /// ISOBMFF (MP4, fMP4)
    Mp4,
    /// WebM/Matroska
    WebM,
    /// Unknown format
    Unknown,
}

/// Codec information extracted from initialization segment
#[derive(Debug, Clone)]
pub struct CodecInfo {
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub duration: Option<Duration>,
}

impl Default for CodecInfo {
    fn default() -> Self {
        Self {
            video_codec: None,
            audio_codec: None,
            width: None,
            height: None,
            sample_rate: None,
            channels: None,
            duration: None,
        }
    }
}

/// A parsed media segment ready for decoding
#[derive(Debug, Clone)]
pub struct MediaSegment {
    pub start_time: Duration,
    pub end_time: Duration,
    pub data: Vec<u8>,
    pub is_keyframe: bool,
    pub segment_type: SegmentType,
}

/// Time range for buffered data
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeRange {
    pub start: f64,
    pub end: f64,
}

impl TimeRange {
    pub fn new(start: f64, end: f64) -> Self {
        Self { start, end }
    }
    
    pub fn duration(&self) -> f64 {
        self.end - self.start
    }
    
    pub fn contains(&self, time: f64) -> bool {
        time >= self.start && time <= self.end
    }
    
    pub fn overlaps(&self, other: &TimeRange) -> bool {
        self.start <= other.end && self.end >= other.start
    }
    
    /// Merge with another range if they overlap or are adjacent
    pub fn merge(&self, other: &TimeRange) -> Option<TimeRange> {
        if self.overlaps(other) || (self.end - other.start).abs() < 0.001 || (self.start - other.end).abs() < 0.001 {
            Some(TimeRange::new(
                self.start.min(other.start),
                self.end.max(other.end),
            ))
        } else {
            None
        }
    }
}

/// Parser for MSE segments
pub struct SegmentParser {
    format: ContainerFormat,
    initialization_data: Option<Vec<u8>>,
    codec_info: CodecInfo,
    pending_data: Vec<u8>,
    buffered_ranges: Vec<TimeRange>,
    append_window_start: f64,
    append_window_end: f64,
    timestamp_offset: f64,
}

impl SegmentParser {
    /// Create a new segment parser
    pub fn new() -> Self {
        Self {
            format: ContainerFormat::Unknown,
            initialization_data: None,
            codec_info: CodecInfo::default(),
            pending_data: Vec::new(),
            buffered_ranges: Vec::new(),
            append_window_start: 0.0,
            append_window_end: f64::INFINITY,
            timestamp_offset: 0.0,
        }
    }
    
    /// Create parser for a specific MIME type
    pub fn for_mime_type(mime_type: &str) -> Self {
        let format = if mime_type.contains("mp4") || mime_type.contains("m4") {
            ContainerFormat::Mp4
        } else if mime_type.contains("webm") || mime_type.contains("matroska") {
            ContainerFormat::WebM
        } else {
            ContainerFormat::Unknown
        };
        
        let mut parser = Self::new();
        parser.format = format;
        parser
    }
    
    /// Set the timestamp offset for appended segments
    pub fn set_timestamp_offset(&mut self, offset: f64) {
        self.timestamp_offset = offset;
    }
    
    /// Get the timestamp offset
    pub fn timestamp_offset(&self) -> f64 {
        self.timestamp_offset
    }
    
    /// Set the append window
    pub fn set_append_window(&mut self, start: f64, end: f64) {
        self.append_window_start = start;
        self.append_window_end = end;
    }
    
    /// Get buffered time ranges
    pub fn buffered_ranges(&self) -> &[TimeRange] {
        &self.buffered_ranges
    }
    
    /// Add a time range to buffered ranges, merging if possible
    fn add_buffered_range(&mut self, range: TimeRange) {
        // Try to merge with existing ranges
        let mut merged = false;
        for existing in &mut self.buffered_ranges {
            if let Some(new_range) = existing.merge(&range) {
                *existing = new_range;
                merged = true;
                break;
            }
        }
        
        if !merged {
            self.buffered_ranges.push(range);
        }
        
        // Sort and re-merge ranges
        self.buffered_ranges.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
        
        let mut i = 0;
        while i + 1 < self.buffered_ranges.len() {
            if let Some(merged) = self.buffered_ranges[i].merge(&self.buffered_ranges[i + 1]) {
                self.buffered_ranges[i] = merged;
                self.buffered_ranges.remove(i + 1);
            } else {
                i += 1;
            }
        }
    }
    
    /// Remove buffered data in a time range
    pub fn remove_range(&mut self, start: f64, end: f64) {
        let remove_range = TimeRange::new(start, end);
        let mut new_ranges = Vec::new();
        
        for range in &self.buffered_ranges {
            if range.end <= start || range.start >= end {
                // Range is completely outside removal range
                new_ranges.push(*range);
            } else if range.start < start && range.end > end {
                // Removal splits this range into two
                new_ranges.push(TimeRange::new(range.start, start));
                new_ranges.push(TimeRange::new(end, range.end));
            } else if range.start < start {
                // Removal trims end of range
                new_ranges.push(TimeRange::new(range.start, start));
            } else if range.end > end {
                // Removal trims start of range
                new_ranges.push(TimeRange::new(end, range.end));
            }
            // else: range is completely inside removal range, drop it
        }
        
        self.buffered_ranges = new_ranges;
    }
    
    /// Get codec info from parsed initialization segment
    pub fn codec_info(&self) -> &CodecInfo {
        &self.codec_info
    }
    
    /// Check if we have initialization data
    pub fn has_initialization(&self) -> bool {
        self.initialization_data.is_some()
    }
    
    /// Get the initialization data
    pub fn initialization_data(&self) -> Option<&[u8]> {
        self.initialization_data.as_deref()
    }
    
    /// Detect segment type from data
    pub fn detect_segment_type(&self, data: &[u8]) -> SegmentType {
        if data.len() < 8 {
            return SegmentType::Unknown;
        }
        
        match self.format {
            ContainerFormat::Mp4 => self.detect_mp4_segment_type(data),
            ContainerFormat::WebM => self.detect_webm_segment_type(data),
            ContainerFormat::Unknown => {
                // Try to auto-detect format
                if self.looks_like_mp4(data) {
                    self.detect_mp4_segment_type(data)
                } else if self.looks_like_webm(data) {
                    self.detect_webm_segment_type(data)
                } else {
                    SegmentType::Unknown
                }
            }
        }
    }
    
    fn looks_like_mp4(&self, data: &[u8]) -> bool {
        if data.len() < 8 {
            return false;
        }
        // Check for ftyp or moov box
        let box_type = &data[4..8];
        matches!(box_type, b"ftyp" | b"moov" | b"moof" | b"styp")
    }
    
    fn looks_like_webm(&self, data: &[u8]) -> bool {
        if data.len() < 4 {
            return false;
        }
        // WebM/EBML signature
        data[0] == 0x1A && data[1] == 0x45 && data[2] == 0xDF && data[3] == 0xA3
    }
    
    fn detect_mp4_segment_type(&self, data: &[u8]) -> SegmentType {
        if data.len() < 8 {
            return SegmentType::Unknown;
        }
        
        let box_type = &data[4..8];
        match box_type {
            b"ftyp" | b"moov" => SegmentType::Initialization,
            b"moof" | b"mdat" | b"styp" => SegmentType::Media,
            _ => SegmentType::Unknown,
        }
    }
    
    fn detect_webm_segment_type(&self, data: &[u8]) -> SegmentType {
        if data.len() < 4 {
            return SegmentType::Unknown;
        }
        
        // EBML header indicates initialization segment
        if data[0] == 0x1A && data[1] == 0x45 && data[2] == 0xDF && data[3] == 0xA3 {
            return SegmentType::Initialization;
        }
        
        // Cluster element indicates media segment
        if data[0] == 0x1F && data[1] == 0x43 && data[2] == 0xB6 && data[3] == 0x75 {
            return SegmentType::Media;
        }
        
        SegmentType::Unknown
    }
    
    /// Append data to the buffer
    pub fn append_data(&mut self, data: &[u8]) -> MediaResult<Vec<MediaSegment>> {
        self.pending_data.extend_from_slice(data);
        
        let segment_type = self.detect_segment_type(&self.pending_data);
        
        match segment_type {
            SegmentType::Initialization => {
                self.parse_initialization_segment()?;
                Ok(Vec::new())
            }
            SegmentType::Media => {
                self.parse_media_segments()
            }
            SegmentType::Unknown => {
                // Keep buffering until we can identify the segment
                if self.pending_data.len() > 1024 * 1024 {
                    // Too much data without identification, probably error
                    warn!("Too much unidentified data in MSE buffer");
                    self.pending_data.clear();
                    return Err(MediaError::ParseError("Cannot identify segment type".to_string()));
                }
                Ok(Vec::new())
            }
        }
    }
    
    /// Parse initialization segment (moov for MP4, EBML header for WebM)
    fn parse_initialization_segment(&mut self) -> MediaResult<()> {
        match self.format {
            ContainerFormat::Mp4 | ContainerFormat::Unknown => {
                self.parse_mp4_init_segment()
            }
            ContainerFormat::WebM => {
                self.parse_webm_init_segment()
            }
        }
    }
    
    fn parse_mp4_init_segment(&mut self) -> MediaResult<()> {
        // Find moov box and extract codec info
        // For now, store the entire init segment for later use with FFmpeg
        
        let data = std::mem::take(&mut self.pending_data);
        
        // Parse basic MP4 structure to extract codec info
        let mut offset = 0;
        while offset + 8 <= data.len() {
            let size = u32::from_be_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]) as usize;
            let box_type = &data[offset+4..offset+8];
            
            if size == 0 || offset + size > data.len() {
                break;
            }
            
            match box_type {
                b"moov" => {
                    // Parse moov for track info
                    self.parse_moov_box(&data[offset+8..offset+size])?;
                }
                b"ftyp" => {
                    // File type box - confirms MP4
                    self.format = ContainerFormat::Mp4;
                }
                _ => {}
            }
            
            offset += size;
        }
        
        self.initialization_data = Some(data);
        debug!("Parsed MP4 initialization segment, codec_info: {:?}", self.codec_info);
        Ok(())
    }
    
    fn parse_moov_box(&mut self, data: &[u8]) -> MediaResult<()> {
        // Simple moov parser to extract basic track info
        let mut offset = 0;
        while offset + 8 <= data.len() {
            let size = u32::from_be_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]) as usize;
            let box_type = &data[offset+4..offset+8];
            
            if size == 0 || offset + size > data.len() {
                break;
            }
            
            match box_type {
                b"trak" => {
                    self.parse_trak_box(&data[offset+8..offset+size])?;
                }
                b"mvhd" => {
                    // Movie header - contains duration
                    if data.len() >= offset + 28 {
                        let timescale = u32::from_be_bytes([
                            data[offset+20], data[offset+21], data[offset+22], data[offset+23]
                        ]);
                        let duration_units = u32::from_be_bytes([
                            data[offset+24], data[offset+25], data[offset+26], data[offset+27]
                        ]);
                        if timescale > 0 {
                            let duration_secs = duration_units as f64 / timescale as f64;
                            self.codec_info.duration = Some(Duration::from_secs_f64(duration_secs));
                        }
                    }
                }
                _ => {}
            }
            
            offset += size;
        }
        Ok(())
    }
    
    fn parse_trak_box(&mut self, data: &[u8]) -> MediaResult<()> {
        let mut offset = 0;
        let mut is_video = false;
        let mut is_audio = false;
        
        while offset + 8 <= data.len() {
            let size = u32::from_be_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]) as usize;
            let box_type = &data[offset+4..offset+8];
            
            if size == 0 || offset + size > data.len() {
                break;
            }
            
            match box_type {
                b"mdia" => {
                    // Parse media box for handler type
                    self.parse_mdia_box(&data[offset+8..offset+size], &mut is_video, &mut is_audio)?;
                }
                _ => {}
            }
            
            offset += size;
        }
        Ok(())
    }
    
    fn parse_mdia_box(&mut self, data: &[u8], is_video: &mut bool, is_audio: &mut bool) -> MediaResult<()> {
        let mut offset = 0;
        while offset + 8 <= data.len() {
            let size = u32::from_be_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]) as usize;
            let box_type = &data[offset+4..offset+8];
            
            if size == 0 || offset + size > data.len() {
                break;
            }
            
            match box_type {
                b"hdlr" if size >= 20 => {
                    // Handler box - determine track type
                    let handler_type = &data[offset+16..offset+20];
                    *is_video = handler_type == b"vide";
                    *is_audio = handler_type == b"soun";
                }
                b"stsd" if size > 16 => {
                    // Sample description - extract codec info
                    // Skip version and flags (4 bytes) and entry count (4 bytes)
                    if *is_video {
                        self.parse_video_sample_desc(&data[offset+16..offset+size])?;
                    } else if *is_audio {
                        self.parse_audio_sample_desc(&data[offset+16..offset+size])?;
                    }
                }
                b"minf" => {
                    // Recurse into minf -> stbl -> stsd
                    self.parse_mdia_box(&data[offset+8..offset+size], is_video, is_audio)?;
                }
                b"stbl" => {
                    self.parse_mdia_box(&data[offset+8..offset+size], is_video, is_audio)?;
                }
                _ => {}
            }
            
            offset += size;
        }
        Ok(())
    }
    
    fn parse_video_sample_desc(&mut self, data: &[u8]) -> MediaResult<()> {
        if data.len() < 8 {
            return Ok(());
        }
        
        // Get codec FourCC
        let codec = String::from_utf8_lossy(&data[4..8]).to_string();
        self.codec_info.video_codec = Some(codec);
        
        // Width and height at offset 24 and 26 (relative to sample entry start)
        if data.len() >= 32 {
            self.codec_info.width = Some(u16::from_be_bytes([data[24], data[25]]) as u32);
            self.codec_info.height = Some(u16::from_be_bytes([data[26], data[27]]) as u32);
        }
        
        Ok(())
    }
    
    fn parse_audio_sample_desc(&mut self, data: &[u8]) -> MediaResult<()> {
        if data.len() < 8 {
            return Ok(());
        }
        
        // Get codec FourCC
        let codec = String::from_utf8_lossy(&data[4..8]).to_string();
        self.codec_info.audio_codec = Some(codec);
        
        // Channel count at offset 16, sample rate at offset 22
        if data.len() >= 28 {
            self.codec_info.channels = Some(u16::from_be_bytes([data[16], data[17]]) as u32);
            // Sample rate is 16.16 fixed point at offset 24
            self.codec_info.sample_rate = Some(u16::from_be_bytes([data[24], data[25]]) as u32);
        }
        
        Ok(())
    }
    
    fn parse_webm_init_segment(&mut self) -> MediaResult<()> {
        // WebM/EBML parsing - store init data for FFmpeg
        let data = std::mem::take(&mut self.pending_data);
        
        // Basic EBML/WebM header detection
        if data.len() >= 4 && data[0] == 0x1A && data[1] == 0x45 && data[2] == 0xDF && data[3] == 0xA3 {
            self.format = ContainerFormat::WebM;
        }
        
        self.initialization_data = Some(data);
        debug!("Parsed WebM initialization segment");
        Ok(())
    }
    
    fn parse_media_segments(&mut self) -> MediaResult<Vec<MediaSegment>> {
        if !self.has_initialization() {
            return Err(MediaError::ParseError("No initialization segment received".to_string()));
        }
        
        let data = std::mem::take(&mut self.pending_data);
        let mut segments = Vec::new();
        
        match self.format {
            ContainerFormat::Mp4 => {
                segments = self.parse_mp4_media_segment(data)?;
            }
            ContainerFormat::WebM => {
                segments = self.parse_webm_media_segment(data)?;
            }
            ContainerFormat::Unknown => {
                return Err(MediaError::ParseError("Unknown container format".to_string()));
            }
        }
        
        // Update buffered ranges
        for segment in &segments {
            let start = segment.start_time.as_secs_f64() + self.timestamp_offset;
            let end = segment.end_time.as_secs_f64() + self.timestamp_offset;
            self.add_buffered_range(TimeRange::new(start, end));
        }
        
        Ok(segments)
    }
    
    fn parse_mp4_media_segment(&mut self, data: Vec<u8>) -> MediaResult<Vec<MediaSegment>> {
        // For MP4 fragmented media, we look for moof + mdat pairs
        let mut segments = Vec::new();
        let mut offset = 0;
        let mut moof_data: Option<&[u8]> = None;
        let mut base_time: Option<Duration> = None;
        
        while offset + 8 <= data.len() {
            let size = u32::from_be_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]) as usize;
            let box_type = &data[offset+4..offset+8];
            
            if size == 0 || offset + size > data.len() {
                break;
            }
            
            match box_type {
                b"moof" => {
                    moof_data = Some(&data[offset..offset+size]);
                    // Extract base decode time from tfdt if present
                    base_time = self.extract_base_time(&data[offset+8..offset+size]);
                }
                b"mdat" => {
                    // Create segment from mdat
                    let start_time = base_time.unwrap_or(Duration::ZERO);
                    // Estimate end time (would need proper parsing for accuracy)
                    let end_time = start_time + Duration::from_millis(1000);
                    
                    segments.push(MediaSegment {
                        start_time,
                        end_time,
                        data: data[offset..offset+size].to_vec(),
                        is_keyframe: true, // Would need parsing to determine
                        segment_type: SegmentType::Media,
                    });
                }
                _ => {}
            }
            
            offset += size;
        }
        
        Ok(segments)
    }
    
    fn extract_base_time(&self, moof_data: &[u8]) -> Option<Duration> {
        // Look for traf -> tfdt to get base media decode time
        let mut offset = 0;
        while offset + 8 <= moof_data.len() {
            let size = u32::from_be_bytes([
                moof_data[offset], moof_data[offset+1], 
                moof_data[offset+2], moof_data[offset+3]
            ]) as usize;
            let box_type = &moof_data[offset+4..offset+8];
            
            if size == 0 || offset + size > moof_data.len() {
                break;
            }
            
            if box_type == b"traf" {
                return self.extract_tfdt_time(&moof_data[offset+8..offset+size]);
            }
            
            offset += size;
        }
        None
    }
    
    fn extract_tfdt_time(&self, traf_data: &[u8]) -> Option<Duration> {
        let mut offset = 0;
        while offset + 8 <= traf_data.len() {
            let size = u32::from_be_bytes([
                traf_data[offset], traf_data[offset+1], 
                traf_data[offset+2], traf_data[offset+3]
            ]) as usize;
            let box_type = &traf_data[offset+4..offset+8];
            
            if size == 0 || offset + size > traf_data.len() {
                break;
            }
            
            if box_type == b"tfdt" && size >= 16 {
                let version = traf_data[offset+8];
                if version == 0 && size >= 16 {
                    let base_time = u32::from_be_bytes([
                        traf_data[offset+12], traf_data[offset+13],
                        traf_data[offset+14], traf_data[offset+15]
                    ]);
                    // Assume 1000 timescale, would need proper timescale from init segment
                    return Some(Duration::from_millis(base_time as u64));
                } else if version == 1 && size >= 20 {
                    let base_time = u64::from_be_bytes([
                        traf_data[offset+12], traf_data[offset+13],
                        traf_data[offset+14], traf_data[offset+15],
                        traf_data[offset+16], traf_data[offset+17],
                        traf_data[offset+18], traf_data[offset+19]
                    ]);
                    return Some(Duration::from_millis(base_time));
                }
            }
            
            offset += size;
        }
        None
    }
    
    fn parse_webm_media_segment(&mut self, data: Vec<u8>) -> MediaResult<Vec<MediaSegment>> {
        // WebM cluster parsing - simplified
        let segments = vec![MediaSegment {
            start_time: Duration::ZERO,
            end_time: Duration::from_secs(1),
            data,
            is_keyframe: true,
            segment_type: SegmentType::Media,
        }];
        
        Ok(segments)
    }
    
    /// Abort current append operation
    pub fn abort(&mut self) {
        self.pending_data.clear();
    }
    
    /// Reset the parser (called when changing media type)
    pub fn reset(&mut self) {
        self.initialization_data = None;
        self.codec_info = CodecInfo::default();
        self.pending_data.clear();
        self.buffered_ranges.clear();
    }
}

impl Default for SegmentParser {
    fn default() -> Self {
        Self::new()
    }
}

/// MSE Source Buffer - manages buffered media data
pub struct MseSourceBuffer {
    parser: SegmentParser,
    segments: VecDeque<MediaSegment>,
    updating: bool,
    mode: AppendMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppendMode {
    Segments,
    Sequence,
}

impl MseSourceBuffer {
    pub fn new(mime_type: &str) -> Self {
        Self {
            parser: SegmentParser::for_mime_type(mime_type),
            segments: VecDeque::new(),
            updating: false,
            mode: AppendMode::Segments,
        }
    }
    
    pub fn is_updating(&self) -> bool {
        self.updating
    }
    
    pub fn set_mode(&mut self, mode: AppendMode) {
        self.mode = mode;
    }
    
    pub fn mode(&self) -> AppendMode {
        self.mode
    }
    
    pub fn append_buffer(&mut self, data: &[u8]) -> MediaResult<()> {
        if self.updating {
            return Err(MediaError::InvalidState("Already updating".to_string()));
        }
        
        self.updating = true;
        
        match self.parser.append_data(data) {
            Ok(new_segments) => {
                for segment in new_segments {
                    self.segments.push_back(segment);
                }
                self.updating = false;
                Ok(())
            }
            Err(e) => {
                self.updating = false;
                Err(e)
            }
        }
    }
    
    pub fn abort(&mut self) {
        self.parser.abort();
        self.updating = false;
    }
    
    pub fn remove(&mut self, start: f64, end: f64) -> MediaResult<()> {
        if self.updating {
            return Err(MediaError::InvalidState("Already updating".to_string()));
        }
        
        self.updating = true;
        
        // Remove segments in range
        self.segments.retain(|seg| {
            let seg_start = seg.start_time.as_secs_f64();
            let seg_end = seg.end_time.as_secs_f64();
            seg_end <= start || seg_start >= end
        });
        
        self.parser.remove_range(start, end);
        self.updating = false;
        
        Ok(())
    }
    
    pub fn buffered_ranges(&self) -> &[TimeRange] {
        self.parser.buffered_ranges()
    }
    
    pub fn set_timestamp_offset(&mut self, offset: f64) {
        self.parser.set_timestamp_offset(offset);
    }
    
    pub fn timestamp_offset(&self) -> f64 {
        self.parser.timestamp_offset()
    }
    
    pub fn set_append_window(&mut self, start: f64, end: f64) {
        self.parser.set_append_window(start, end);
    }
    
    /// Get initialization data for decoder
    pub fn initialization_data(&self) -> Option<&[u8]> {
        self.parser.initialization_data()
    }
    
    /// Take the next segment for decoding
    pub fn take_next_segment(&mut self) -> Option<MediaSegment> {
        self.segments.pop_front()
    }
    
    /// Peek at segments without removing them
    pub fn peek_segments(&self) -> impl Iterator<Item = &MediaSegment> {
        self.segments.iter()
    }
    
    /// Get codec info
    pub fn codec_info(&self) -> &CodecInfo {
        self.parser.codec_info()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_time_range_merge() {
        let r1 = TimeRange::new(0.0, 5.0);
        let r2 = TimeRange::new(5.0, 10.0);
        
        let merged = r1.merge(&r2);
        assert!(merged.is_some());
        let m = merged.unwrap();
        assert_eq!(m.start, 0.0);
        assert_eq!(m.end, 10.0);
    }
    
    #[test]
    fn test_time_range_no_merge() {
        let r1 = TimeRange::new(0.0, 5.0);
        let r2 = TimeRange::new(6.0, 10.0);
        
        let merged = r1.merge(&r2);
        assert!(merged.is_none());
    }
    
    #[test]
    fn test_segment_parser_new() {
        let parser = SegmentParser::new();
        assert!(!parser.has_initialization());
        assert_eq!(parser.buffered_ranges().len(), 0);
    }
    
    #[test]
    fn test_mp4_detection() {
        let parser = SegmentParser::for_mime_type("video/mp4");
        assert_eq!(parser.format, ContainerFormat::Mp4);
        
        // ftyp box header
        let data = [0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p'];
        assert_eq!(parser.detect_segment_type(&data), SegmentType::Initialization);
    }
}

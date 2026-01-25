/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! MediaSource API implementation for Ferro Browser
//! https://w3c.github.io/media-source/

use std::cell::Cell;

use dom_struct::dom_struct;
use crate::js::rust::HandleObject;
use script_bindings::codegen::GenericBindings::MediaSourceBinding::{
    EndOfStreamError, MediaSourceMethods, ReadyState,
};
use script_bindings::num::Finite;
use script_bindings::str::DOMString;
use uuid::Uuid;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::sourcebuffer::SourceBuffer;
use crate::dom::sourcebufferlist::SourceBufferList;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;
use style::Atom;

/// Supported MIME types for MediaSource
const SUPPORTED_TYPES: &[&str] = &[
    // Video
    "video/mp4",
    "video/webm",
    "video/mp4; codecs=\"avc1.42E01E\"",
    "video/mp4; codecs=\"avc1.42E01E, mp4a.40.2\"",
    "video/mp4; codecs=\"avc1.4D401E\"",
    "video/mp4; codecs=\"avc1.64001E\"",
    "video/webm; codecs=\"vp8\"",
    "video/webm; codecs=\"vp8, vorbis\"",
    "video/webm; codecs=\"vp9\"",
    "video/webm; codecs=\"vp9, opus\"",
    // Audio
    "audio/mp4",
    "audio/webm",
    "audio/mp4; codecs=\"mp4a.40.2\"",
    "audio/webm; codecs=\"opus\"",
    "audio/webm; codecs=\"vorbis\"",
];

#[dom_struct]
pub(crate) struct MediaSource {
    eventtarget: EventTarget,
    ready_state: Cell<ReadyState>,
    duration: Cell<f64>,
    source_buffers: MutNullableDom<SourceBufferList>,
    active_source_buffers: MutNullableDom<SourceBufferList>,
    live_seekable_range: DomRefCell<Option<(f64, f64)>>,
    object_url: DomRefCell<Option<String>>,
}

impl MediaSource {
    fn new_inherited() -> MediaSource {
        MediaSource {
            eventtarget: EventTarget::new_inherited(),
            ready_state: Cell::new(ReadyState::Closed),
            duration: Cell::new(f64::NAN),
            source_buffers: MutNullableDom::new(None),
            active_source_buffers: MutNullableDom::new(None),
            live_seekable_range: DomRefCell::new(None),
            object_url: DomRefCell::new(None),
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<MediaSource> {
        let media_source = reflect_dom_object_with_proto(
            Box::new(MediaSource::new_inherited()),
            global,
            proto,
            can_gc,
        );
        
        // Initialize source buffer lists
        let source_buffers = SourceBufferList::new(global, can_gc);
        let active_source_buffers = SourceBufferList::new(global, can_gc);
        
        media_source.source_buffers.set(Some(&source_buffers));
        media_source.active_source_buffers.set(Some(&active_source_buffers));
        
        // Generate object URL
        let url = format!("blob:ferro-mediasource-{}", Uuid::new_v4());
        *media_source.object_url.borrow_mut() = Some(url);
        
        media_source
    }

    #[allow(dead_code)]
    pub(crate) fn get_ready_state(&self) -> ReadyState {
        self.ready_state.get()
    }

    pub(crate) fn set_ready_state(&self, state: ReadyState, can_gc: CanGc) {
        let old_state = self.ready_state.get();
        self.ready_state.set(state);

        // Fire appropriate events
        match (old_state, state) {
            (ReadyState::Closed, ReadyState::Open) => {
                self.fire_event(Atom::from("sourceopen"), can_gc);
            },
            (ReadyState::Open, ReadyState::Ended) => {
                self.fire_event(Atom::from("sourceended"), can_gc);
            },
            (_, ReadyState::Closed) => {
                self.fire_event(Atom::from("sourceclose"), can_gc);
            },
            _ => {},
        }
    }

    fn fire_event(&self, name: Atom, can_gc: CanGc) {
        let event = Event::new(
            &self.global(),
            name,
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            can_gc,
        );
        event.upcast::<Event>().fire(self.upcast::<EventTarget>(), can_gc);
    }

    pub(crate) fn get_object_url(&self) -> Option<String> {
        self.object_url.borrow().clone()
    }

    fn is_type_supported_internal(mime_type: &str) -> bool {
        let mime_lower = mime_type.to_lowercase();
        
        // Check exact matches first
        for &supported in SUPPORTED_TYPES {
            if mime_lower == supported.to_lowercase() {
                return true;
            }
        }
        
        // Check base types without codecs
        let base_type = mime_lower.split(';').next().unwrap_or("").trim();
        matches!(
            base_type,
            "video/mp4" | "video/webm" | "audio/mp4" | "audio/webm" | "audio/mpeg"
        )
    }
}

impl MediaSourceMethods<crate::DomTypeHolder> for MediaSource {
    /// https://w3c.github.io/media-source/#dom-mediasource-constructor
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<MediaSource>> {
        Ok(MediaSource::new(window.upcast(), proto, can_gc))
    }

    /// https://w3c.github.io/media-source/#dom-mediasource-sourcebuffers
    fn SourceBuffers(&self) -> DomRoot<SourceBufferList> {
        self.source_buffers.get().expect("source_buffers not initialized")
    }

    /// https://w3c.github.io/media-source/#dom-mediasource-activesourcebuffers
    fn ActiveSourceBuffers(&self) -> DomRoot<SourceBufferList> {
        self.active_source_buffers.get().expect("active_source_buffers not initialized")
    }

    /// https://w3c.github.io/media-source/#dom-mediasource-readystate
    fn ReadyState(&self) -> ReadyState {
        self.ready_state.get()
    }

    /// https://w3c.github.io/media-source/#dom-mediasource-duration
    fn Duration(&self) -> f64 {
        self.duration.get()
    }

    /// https://w3c.github.io/media-source/#dom-mediasource-duration
    fn SetDuration(&self, value: f64) {
        // TODO: Validate state and throw if needed
        self.duration.set(value);
    }

    /// https://w3c.github.io/media-source/#dom-mediasource-addsourcebuffer
    fn AddSourceBuffer(&self, mime_type: DOMString) -> Fallible<DomRoot<SourceBuffer>> {
        // Step 1: If type is empty, throw TypeError
        if mime_type.is_empty() {
            return Err(Error::Type("MIME type cannot be empty".into()));
        }

        // Step 2: If type is not supported, throw NotSupportedError
        if !Self::is_type_supported_internal(&mime_type.to_string()) {
            return Err(Error::NotSupported(None));
        }

        // Step 3: If readyState is not "open", throw InvalidStateError
        if self.ready_state.get() != ReadyState::Open {
            return Err(Error::InvalidState(None));
        }

        // Create new SourceBuffer
        let can_gc = CanGc::note();
        let source_buffer = SourceBuffer::new(&self.global(), mime_type.clone(), can_gc);
        
        // Add to sourceBuffers list
        if let Some(list) = self.source_buffers.get() {
            list.append(&source_buffer);
        }

        Ok(source_buffer)
    }

    /// https://w3c.github.io/media-source/#dom-mediasource-removesourcebuffer
    fn RemoveSourceBuffer(&self, source_buffer: &SourceBuffer) -> Fallible<()> {
        // Step 1: If sourceBuffer is not in sourceBuffers, throw NotFoundError
        if let Some(list) = self.source_buffers.get() {
            if !list.contains(source_buffer) {
                return Err(Error::NotFound(None));
            }
            list.remove(source_buffer);
        }

        // Also remove from activeSourceBuffers if present
        if let Some(active_list) = self.active_source_buffers.get() {
            active_list.remove(source_buffer);
        }

        Ok(())
    }

    /// https://w3c.github.io/media-source/#dom-mediasource-endofstream
    fn EndOfStream(&self, error: Option<EndOfStreamError>) -> Fallible<()> {
        // Step 1: If readyState is not "open", throw InvalidStateError
        if self.ready_state.get() != ReadyState::Open {
            return Err(Error::InvalidState(None));
        }

        // Step 2: If any SourceBuffer is updating, throw InvalidStateError
        if let Some(list) = self.source_buffers.get() {
            if list.has_updating_buffer() {
                return Err(Error::InvalidState(None));
            }
        }

        // Step 3: Set readyState to "ended"
        let can_gc = CanGc::note();
        self.set_ready_state(ReadyState::Ended, can_gc);

        // Step 4: If error is provided, handle it
        if let Some(err) = error {
            match err {
                EndOfStreamError::Network => {
                    // TODO: Signal network error to media element
                },
                EndOfStreamError::Decode => {
                    // TODO: Signal decode error to media element
                },
            }
        }

        Ok(())
    }

    /// https://w3c.github.io/media-source/#dom-mediasource-setliveseekablerange
    fn SetLiveSeekableRange(&self, start: Finite<f64>, end: Finite<f64>) -> Fallible<()> {
        // Step 1: If readyState is not "open", throw InvalidStateError
        if self.ready_state.get() != ReadyState::Open {
            return Err(Error::InvalidState(None));
        }

        let start_val = *start;
        let end_val = *end;

        // Step 2: If start < 0 or start > end, throw TypeError
        if start_val < 0.0 || start_val > end_val {
            return Err(Error::Type("Invalid range".into()));
        }

        *self.live_seekable_range.borrow_mut() = Some((start_val, end_val));
        Ok(())
    }

    /// https://w3c.github.io/media-source/#dom-mediasource-clearliveseekablerange
    fn ClearLiveSeekableRange(&self) -> Fallible<()> {
        // Step 1: If readyState is not "open", throw InvalidStateError
        if self.ready_state.get() != ReadyState::Open {
            return Err(Error::InvalidState(None));
        }

        *self.live_seekable_range.borrow_mut() = None;
        Ok(())
    }

    /// https://w3c.github.io/media-source/#dom-mediasource-istypesupported
    fn IsTypeSupported(_window: &Window, mime_type: DOMString) -> bool {
        Self::is_type_supported_internal(&mime_type.to_string())
    }

    /// Ferro extension: Get object URL for MediaSource
    fn GetObjectURL(_window: &Window, media_source: &MediaSource) -> DOMString {
        media_source
            .get_object_url()
            .map(DOMString::from)
            .unwrap_or_default()
    }

    // Event handlers
    event_handler!(sourceopen, GetOnsourceopen, SetOnsourceopen);
    event_handler!(sourceended, GetOnsourceended, SetOnsourceended);
    event_handler!(sourceclose, GetOnsourceclose, SetOnsourceclose);
}

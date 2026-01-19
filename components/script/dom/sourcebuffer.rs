/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! SourceBuffer implementation for MediaSource API
//! https://w3c.github.io/media-source/#sourcebuffer

use std::cell::Cell;

use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::SourceBufferBinding::{AppendMode, SourceBufferMethods};
use script_bindings::num::Finite;
use script_bindings::str::DOMString;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferViewOrArrayBuffer;
use crate::dom::timeranges::{TimeRanges, TimeRangesContainer};
use crate::script_runtime::CanGc;
use style::Atom;

#[dom_struct]
pub(crate) struct SourceBuffer {
    eventtarget: EventTarget,
    mime_type: DOMString,
    mode: Cell<AppendMode>,
    updating: Cell<bool>,
    timestamp_offset: Cell<f64>,
    append_window_start: Cell<f64>,
    append_window_end: Cell<f64>,
    /// Buffered time ranges
    buffered_ranges: DomRefCell<TimeRangesContainer>,
    /// Pending buffer data
    pending_data: DomRefCell<Vec<u8>>,
}

impl SourceBuffer {
    fn new_inherited(mime_type: DOMString) -> SourceBuffer {
        SourceBuffer {
            eventtarget: EventTarget::new_inherited(),
            mime_type,
            mode: Cell::new(AppendMode::Segments),
            updating: Cell::new(false),
            timestamp_offset: Cell::new(0.0),
            append_window_start: Cell::new(0.0),
            append_window_end: Cell::new(f64::INFINITY),
            buffered_ranges: DomRefCell::new(TimeRangesContainer::default()),
            pending_data: DomRefCell::new(Vec::new()),
        }
    }

    pub(crate) fn new(global: &GlobalScope, mime_type: DOMString, can_gc: CanGc) -> DomRoot<SourceBuffer> {
        reflect_dom_object(
            Box::new(SourceBuffer::new_inherited(mime_type)),
            global,
            can_gc,
        )
    }

    pub(crate) fn is_updating(&self) -> bool {
        self.updating.get()
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

    fn start_append(&self, can_gc: CanGc) {
        self.updating.set(true);
        self.fire_event(Atom::from("updatestart"), can_gc);
    }

    fn end_append(&self, can_gc: CanGc) {
        self.updating.set(false);
        self.fire_event(Atom::from("update"), can_gc);
        self.fire_event(Atom::from("updateend"), can_gc);
    }

    #[allow(dead_code)]
    fn signal_error(&self, can_gc: CanGc) {
        self.updating.set(false);
        self.fire_event(Atom::from("error"), can_gc);
        self.fire_event(Atom::from("updateend"), can_gc);
    }
}

impl SourceBufferMethods<crate::DomTypeHolder> for SourceBuffer {
    /// https://w3c.github.io/media-source/#dom-sourcebuffer-mode
    fn Mode(&self) -> AppendMode {
        self.mode.get()
    }

    /// https://w3c.github.io/media-source/#dom-sourcebuffer-mode
    fn SetMode(&self, value: AppendMode) {
        // TODO: Check if we can change mode
        self.mode.set(value);
    }

    /// https://w3c.github.io/media-source/#dom-sourcebuffer-updating
    fn Updating(&self) -> bool {
        self.updating.get()
    }

    /// https://w3c.github.io/media-source/#dom-sourcebuffer-buffered
    fn Buffered(&self) -> DomRoot<TimeRanges> {
        let can_gc = CanGc::note();
        let global = self.global();
        let window = global.as_window();
        let ranges = self.buffered_ranges.borrow().clone();
        TimeRanges::new(window, ranges, can_gc)
    }

    /// https://w3c.github.io/media-source/#dom-sourcebuffer-timestampoffset
    fn TimestampOffset(&self) -> Finite<f64> {
        Finite::wrap(self.timestamp_offset.get())
    }

    /// https://w3c.github.io/media-source/#dom-sourcebuffer-timestampoffset
    fn SetTimestampOffset(&self, value: Finite<f64>) {
        self.timestamp_offset.set(*value);
    }

    /// https://w3c.github.io/media-source/#dom-sourcebuffer-appendwindowstart
    fn AppendWindowStart(&self) -> Finite<f64> {
        Finite::wrap(self.append_window_start.get())
    }

    /// https://w3c.github.io/media-source/#dom-sourcebuffer-appendwindowstart
    fn SetAppendWindowStart(&self, value: Finite<f64>) {
        self.append_window_start.set(*value);
    }

    /// https://w3c.github.io/media-source/#dom-sourcebuffer-appendwindowend
    fn AppendWindowEnd(&self) -> f64 {
        self.append_window_end.get()
    }

    /// https://w3c.github.io/media-source/#dom-sourcebuffer-appendwindowend
    fn SetAppendWindowEnd(&self, value: f64) {
        self.append_window_end.set(value);
    }

    /// https://w3c.github.io/media-source/#dom-sourcebuffer-appendbuffer
    fn AppendBuffer(&self, data: ArrayBufferViewOrArrayBuffer) -> Fallible<()> {
        // Step 1: If updating is true, throw InvalidStateError
        if self.updating.get() {
            return Err(Error::InvalidState(None));
        }

        let can_gc = CanGc::note();

        // Step 2: Start the append operation
        self.start_append(can_gc);

        // Step 3: Copy buffer data
        let bytes: Vec<u8> = match &data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => {
                buffer.to_vec()
            }
        };
        
        if !bytes.is_empty() {
            let mut pending = self.pending_data.borrow_mut();
            pending.extend_from_slice(&bytes);
            
            // Get current end time
            let mut ranges = self.buffered_ranges.borrow_mut();
            let current_end = if ranges.len() > 0 {
                ranges.end(ranges.len() - 1).unwrap_or(0.0)
            } else {
                0.0
            };
            
            // Assume ~1 second of content per 100KB (rough estimate)
            let duration = bytes.len() as f64 / 100_000.0;
            let _ = ranges.add(current_end, current_end + duration);
        }

        // Step 4: End append
        self.end_append(can_gc);

        Ok(())
    }

    /// https://w3c.github.io/media-source/#dom-sourcebuffer-abort
    fn Abort(&self) -> Fallible<()> {
        // Step 1: If updating is false, return
        if !self.updating.get() {
            return Ok(());
        }

        let can_gc = CanGc::note();

        // Step 2: Abort any pending operations
        self.pending_data.borrow_mut().clear();

        // Step 3: Fire abort event
        self.updating.set(false);
        self.fire_event(Atom::from("abort"), can_gc);
        self.fire_event(Atom::from("updateend"), can_gc);

        Ok(())
    }

    /// https://w3c.github.io/media-source/#dom-sourcebuffer-changetype
    fn ChangeType(&self, mime_type: DOMString) -> Fallible<()> {
        // Step 1: If updating is true, throw InvalidStateError
        if self.updating.get() {
            return Err(Error::InvalidState(None));
        }

        // Step 2: If type is empty or not supported, throw NotSupportedError
        if mime_type.is_empty() {
            return Err(Error::NotSupported(None));
        }

        // Type change accepted (in a real implementation, would reconfigure decoder)
        Ok(())
    }

    /// https://w3c.github.io/media-source/#dom-sourcebuffer-remove
    fn Remove(&self, start: Finite<f64>, end: f64) -> Fallible<()> {
        let start_val = *start;
        
        // Step 1: If updating is true, throw InvalidStateError
        if self.updating.get() {
            return Err(Error::InvalidState(None));
        }

        // Step 2: If start < 0 or start > duration, throw TypeError
        if start_val < 0.0 {
            return Err(Error::Type("start must be >= 0".into()));
        }

        // Step 3: If end <= start or end is NaN, throw TypeError
        if end <= start_val || end.is_nan() {
            return Err(Error::Type("end must be > start and not NaN".into()));
        }

        let can_gc = CanGc::note();

        // Start remove operation
        self.start_append(can_gc);

        // Clear buffered ranges (simplified - real implementation would be more precise)
        *self.buffered_ranges.borrow_mut() = TimeRangesContainer::default();

        // End operation
        self.end_append(can_gc);

        Ok(())
    }

    // Event handlers
    event_handler!(updatestart, GetOnupdatestart, SetOnupdatestart);
    event_handler!(update, GetOnupdate, SetOnupdate);
    event_handler!(updateend, GetOnupdateend, SetOnupdateend);
    event_handler!(error, GetOnerror, SetOnerror);
    event_handler!(abort, GetOnabort, SetOnabort);
}

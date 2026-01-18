/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! SourceBufferList implementation for MediaSource API
//! https://w3c.github.io/media-source/#sourcebufferlist

use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::SourceBufferListBinding::SourceBufferListMethods;
use style::Atom;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::sourcebuffer::SourceBuffer;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct SourceBufferList {
    eventtarget: EventTarget,
    buffers: DomRefCell<Vec<Dom<SourceBuffer>>>,
}

impl SourceBufferList {
    fn new_inherited() -> SourceBufferList {
        SourceBufferList {
            eventtarget: EventTarget::new_inherited(),
            buffers: DomRefCell::new(Vec::new()),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<SourceBufferList> {
        reflect_dom_object(
            Box::new(SourceBufferList::new_inherited()),
            global,
            can_gc,
        )
    }

    pub(crate) fn append(&self, buffer: &SourceBuffer) {
        self.buffers.borrow_mut().push(Dom::from_ref(buffer));
    }

    pub(crate) fn remove(&self, buffer: &SourceBuffer) {
        self.buffers.borrow_mut().retain(|b| &**b != buffer);
    }

    pub(crate) fn contains(&self, buffer: &SourceBuffer) -> bool {
        self.buffers.borrow().iter().any(|b| &**b == buffer)
    }

    pub(crate) fn has_updating_buffer(&self) -> bool {
        self.buffers.borrow().iter().any(|b| b.is_updating())
    }

    #[allow(dead_code)]
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
}

impl SourceBufferListMethods<crate::DomTypeHolder> for SourceBufferList {
    /// https://w3c.github.io/media-source/#dom-sourcebufferlist-length
    fn Length(&self) -> u32 {
        self.buffers.borrow().len() as u32
    }

    /// https://w3c.github.io/media-source/#dom-sourcebufferlist-getter
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<SourceBuffer>> {
        self.buffers
            .borrow()
            .get(index as usize)
            .map(|b| DomRoot::from_ref(&**b))
    }

    // Event handlers
    event_handler!(addsourcebuffer, GetOnaddsourcebuffer, SetOnaddsourcebuffer);
    event_handler!(removesourcebuffer, GetOnremovesourcebuffer, SetOnremovesourcebuffer);
}

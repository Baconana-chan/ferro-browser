/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::cross_process_instant::CrossProcessInstant;
use dom_struct::dom_struct;
use time::Duration;

use crate::dom::bindings::codegen::Bindings::PerformanceMeasureBinding::PerformanceMeasureMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performance::performanceentry::{EntryType, PerformanceEntry};
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub(crate) struct PerformanceMeasure {
    entry: PerformanceEntry,
}

impl PerformanceMeasure {
    fn new_inherited(name: DOMString, start_time: CrossProcessInstant, duration: Duration) -> PerformanceMeasure {
        PerformanceMeasure {
            entry: PerformanceEntry::new_inherited(name, EntryType::Measure, Some(start_time), duration),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        global: &GlobalScope,
        name: DOMString,
        start_time: CrossProcessInstant,
        duration: Duration,
    ) -> DomRoot<PerformanceMeasure> {
        let entry = PerformanceMeasure::new_inherited(name, start_time, duration);
        reflect_dom_object(Box::new(entry), global, CanGc::note())
    }
}

impl PerformanceMeasureMethods<crate::DomTypeHolder> for PerformanceMeasure {
    /// https://w3c.github.io/user-timing/#dom-performancemeasure-detail
    fn Detail(&self, _cx: JSContext, mut retval: crate::js::rust::MutableHandleValue) {
        // Return null for now - full implementation would store the detail from options
        retval.set(crate::js::jsval::NullValue());
    }
}

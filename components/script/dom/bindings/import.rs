/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) mod base {
    pub(crate) use std::ptr;

    #[allow(unused_imports)]
    pub(crate) use crate::js::context::JSContext;
    #[allow(unused_imports)]
    pub(crate) use crate::js::realm::CurrentRealm;
    pub(crate) use crate::js::rust::{HandleObject, MutableHandleObject};

    pub(crate) use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
}

pub(crate) mod module {
    #[cfg(feature = "js-spidermonkey")]
    pub(crate) use script_bindings::codegen::PrototypeList;
    #[cfg(feature = "js-boa")]
    pub(crate) use crate::script_bindings::codegen::PrototypeList;
    #[cfg(feature = "js-spidermonkey")]
    pub(crate) use script_bindings::conversions::IDLInterface;
    #[cfg(feature = "js-boa")]
    pub(crate) use crate::script_bindings::conversions::IDLInterface;
    #[cfg(feature = "js-spidermonkey")]
    pub(crate) use script_bindings::utils::DOMClass;
    #[cfg(feature = "js-boa")]
    pub(crate) use crate::script_bindings::utils::DOMClass;

    pub(crate) use super::base::*;
    pub(crate) use crate::dom::bindings::iterable::IterableIterator;
    pub(crate) use crate::dom::bindings::reflector::{
        DomObjectIteratorWrap, DomObjectWrap, Reflector,
    };
    pub(crate) use crate::dom::bindings::root::{Dom, Root};
    pub(crate) use crate::dom::bindings::weakref::WeakReferenceable;
}

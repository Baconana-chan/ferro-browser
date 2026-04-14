/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implementation of `iterable<...>` and `iterable<..., ...>` WebIDL declarations.

use crate::js::conversions::ToJSValConvertible;
use crate::utils::DOMClass;

/// The values that an iterator will iterate over.
#[cfg_attr(not(feature = "js-boa"), derive(JSTraceable))]
#[derive(MallocSizeOf)]
pub(crate) enum IteratorType {
    /// The keys of the iterable object.
    Keys,
    /// The values of the iterable object.
    Values,
    /// The keys and values of the iterable object combined.
    Entries,
}

#[cfg(feature = "js-boa")]
unsafe impl crate::JSTraceable for IteratorType {
    unsafe fn trace(&self, _tracer: *mut crate::js::jsapi::JSTracer) {}
}

/// A DOM object that can be iterated over using a pair value iterator.
pub trait Iterable {
    /// The type of the key of the iterator pair.
    type Key: ToJSValConvertible;
    /// The type of the value of the iterator pair.
    type Value: ToJSValConvertible;
    /// Return the number of entries that can be iterated over.
    fn get_iterable_length(&self) -> u32;
    /// Return the value at the provided index.
    fn get_value_at_index(&self, index: u32) -> Self::Value;
    /// Return the key at the provided index.
    fn get_key_at_index(&self, index: u32) -> Self::Key;
}

/// A version of the [IDLInterface] trait that is specific to types that have
/// iterators defined for them. This allows the `script` crate to define the
/// derives check for the concrete interface type, while the [IteratableIterator]
/// type defined in this module can be parameterized over an unknown generic.
pub trait IteratorDerives {
    fn derives(class: &'static DOMClass) -> bool;
}

// Full implementation for SpiderMonkey
#[cfg(not(feature = "js-boa"))]
mod spidermonkey_impl {
use std::cell::Cell;
use std::marker::PhantomData;
use std::ptr;
use std::ptr::NonNull;

use dom_struct::dom_struct;
use crate::js::conversions::ToJSValConvertible;
use crate::js::jsapi::{Heap, JSObject};
use crate::js::jsval::UndefinedValue;
use crate::js::rust::{HandleObject, HandleValue, MutableHandleObject};

use crate::codegen::GenericBindings::IterableIteratorBinding::{
    IterableKeyAndValueResult, IterableKeyOrValueResult,
};
use crate::conversions::IDLInterface;
use crate::error::Fallible;
use crate::interfaces::DomHelpers;
use crate::realms::InRealm;
use crate::reflector::{DomGlobalGeneric, DomObjectIteratorWrap, DomObjectWrap, Reflector};
use crate::root::{Dom, DomRoot, Root};
use crate::script_runtime::{CanGc, JSContext};
use crate::trace::{NoTrace, RootedTraceableBox};
use crate::utils::DOMClass;
use crate::{DomTypes, JSTraceable};
use super::{IteratorType, Iterable, IteratorDerives};

/// An iterator over the iterable entries of a given DOM interface.
#[dom_struct]
pub struct IterableIterator<
    D: DomTypes,
    T: DomObjectIteratorWrap<D> + JSTraceable + Iterable + DomGlobalGeneric<D>,
> {
    reflector: Reflector,
    iterable: Dom<T>,
    type_: IteratorType,
    index: Cell<u32>,
    _marker: NoTrace<PhantomData<D>>,
}

impl<D: DomTypes, T: DomObjectIteratorWrap<D> + JSTraceable + Iterable> IterableIterator<D, T> {
    pub fn global_(&self, realm: InRealm) -> DomRoot<D::GlobalScope> {
        <Self as DomGlobalGeneric<D>>::global_(self, realm)
    }
}

impl<
    D: DomTypes,
    T: DomObjectIteratorWrap<D>
        + JSTraceable
        + Iterable
        + DomGlobalGeneric<D>
        + IDLInterface
        + IteratorDerives,
> IDLInterface for IterableIterator<D, T>
{
    fn derives(class: &'static DOMClass) -> bool {
        <T as IteratorDerives>::derives(class)
    }
}

impl<D: DomTypes, T: DomObjectIteratorWrap<D> + JSTraceable + Iterable + DomGlobalGeneric<D>>
    IterableIterator<D, T>
{
    /// Create a new iterator instance for the provided iterable DOM interface.
    pub(crate) fn new(iterable: &T, type_: IteratorType, realm: InRealm) -> DomRoot<Self> {
        let iterator = Box::new(IterableIterator {
            reflector: Reflector::new(),
            type_,
            iterable: Dom::from_ref(iterable),
            index: Cell::new(0),
            _marker: NoTrace(PhantomData),
        });
        <D as DomHelpers<D>>::reflect_dom_object(iterator, &*iterable.global_(realm), CanGc::note())
    }

    /// Return the next value from the iterable object.
    #[allow(non_snake_case)]
    pub fn Next(&self, cx: JSContext) -> Fallible<NonNull<JSObject>> {
        let index = self.index.get();
        rooted!(in(*cx) let mut value = UndefinedValue());
        rooted!(in(*cx) let mut rval = ptr::null_mut::<JSObject>());
        let result = if index >= self.iterable.get_iterable_length() {
            dict_return(cx, rval.handle_mut(), true, value.handle())
        } else {
            match self.type_ {
                IteratorType::Keys => {
                    unsafe {
                        self.iterable
                            .get_key_at_index(index)
                            .to_jsval(*cx, value.handle_mut());
                    }
                    dict_return(cx, rval.handle_mut(), false, value.handle())
                },
                IteratorType::Values => {
                    unsafe {
                        self.iterable
                            .get_value_at_index(index)
                            .to_jsval(*cx, value.handle_mut());
                    }
                    dict_return(cx, rval.handle_mut(), false, value.handle())
                },
                IteratorType::Entries => {
                    rooted!(in(*cx) let mut key = UndefinedValue());
                    unsafe {
                        self.iterable
                            .get_key_at_index(index)
                            .to_jsval(*cx, key.handle_mut());
                        self.iterable
                            .get_value_at_index(index)
                            .to_jsval(*cx, value.handle_mut());
                    }
                    key_and_value_return(cx, rval.handle_mut(), key.handle(), value.handle())
                },
            }
        };
        self.index.set(index + 1);
        result.map(|_| NonNull::new(rval.get()).expect("got a null pointer"))
    }
}

impl<D: DomTypes, T: DomObjectIteratorWrap<D> + JSTraceable + Iterable + DomGlobalGeneric<D>>
    DomObjectWrap<D> for IterableIterator<D, T>
{
    const WRAP: unsafe fn(
        JSContext,
        &D::GlobalScope,
        Option<HandleObject>,
        Box<Self>,
        CanGc,
    ) -> Root<Dom<Self>> = T::ITER_WRAP;
}

fn dict_return(
    cx: JSContext,
    mut result: MutableHandleObject,
    done: bool,
    value: HandleValue,
) -> Fallible<()> {
    let mut dict = IterableKeyOrValueResult::empty();
    dict.done = done;
    dict.value = Some(value.get());
    rooted!(in(*cx) let mut dict_value = UndefinedValue());
    unsafe {
        dict.to_jsval(*cx, dict_value.handle_mut());
    }
    result.set(dict_value.to_object());
    Ok(())
}

fn key_and_value_return(
    cx: JSContext,
    mut result: MutableHandleObject,
    key: HandleValue,
    value: HandleValue,
) -> Fallible<()> {
    let mut dict = IterableKeyAndValueResult::empty();
    dict.done = false;
    dict.value = Some((key.get(), value.get()));
    rooted!(in(*cx) let mut dict_value = UndefinedValue());
    unsafe {
        dict.to_jsval(*cx, dict_value.handle_mut());
    }
    result.set(dict_value.to_object());
    Ok(())
}
} // end spidermonkey_impl

#[cfg(not(feature = "js-boa"))]
pub use spidermonkey_impl::*;

// Minimal stub for js-boa
#[cfg(feature = "js-boa")]
mod boa_impl {
    use std::marker::PhantomData;
    use std::ptr::NonNull;
    use malloc_size_of::MallocSizeOf;
    use crate::DomTypes;
    use crate::reflector::{DomObject, Reflector};
    use crate::realms::InRealm;
    use crate::root::DomRoot;
    use crate::conversions::IDLInterface;
    use crate::error::Fallible;
    use crate::JSTraceable;
    use crate::js::jsapi::JSObject;
    use crate::script_runtime::JSContext;
    use super::IteratorType;
    
    /// Stub IterableIterator for Boa
    pub struct IterableIterator<D: DomTypes, T> {
        reflector: Reflector,
        _marker: PhantomData<(D, T)>,
    }
    
    impl<D: DomTypes, T> IterableIterator<D, T> {
        /// Create a new iterator instance (stub)
        pub fn new(_iterable: &T, _type_: IteratorType, _realm: InRealm) -> DomRoot<Self> {
            // Stub: return a dummy root
            // SAFETY: This is a stub for Boa compatibility
            unsafe {
                let iterator = Box::new(IterableIterator {
                    reflector: Reflector::new(),
                    _marker: PhantomData,
                });
                // For Boa, we can't properly root this yet, so we leak it
                // This is a temporary stub implementation
                std::mem::forget(iterator);
                DomRoot::from_ptr(Box::leak(iterator) as *mut _)
            }
        }
        
        /// Return the next value from the iterable (stub)
        #[allow(non_snake_case)]
        pub fn Next(&self, _cx: JSContext) -> Fallible<NonNull<JSObject>> {
            // Stub: return a dummy object
            // SAFETY: This is a stub for Boa compatibility
            Ok(NonNull::dangling())
        }
        
        /// Get the global scope (stub)
        pub fn global_(&self, _realm: InRealm) -> DomRoot<D::GlobalScope> {
            // Stub: return a dummy global
            // SAFETY: This is a stub for Boa compatibility
            unsafe { DomRoot::from_ptr(std::ptr::null_mut()) }
        }
    }
    
    unsafe impl<D: DomTypes, T> JSTraceable for IterableIterator<D, T> {
        unsafe fn trace(&self, _tracer: *mut crate::js::jsapi::JSTracer) {
            // Stub: no-op for Boa
        }
    }
    
    impl<D: DomTypes, T> DomObject for IterableIterator<D, T> {
        // Stub: minimal DomObject implementation
    }
    
    impl<D: DomTypes, T> IDLInterface for IterableIterator<D, T> {
        fn derives(_class: &'static crate::utils::DOMClass) -> bool {
            // Stub: always false for now
            false
        }
    }
    
    unsafe impl<D: DomTypes, T> MallocSizeOf for IterableIterator<D, T> {
        fn size_of(&self, _ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
            // Stub: return 0 for now
            0
        }
    }
}

#[cfg(feature = "js-boa")]
pub use boa_impl::*;

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::js::jsapi::{GetCurrentRealmOrNull, JSAutoRealm};
use crate::js::realm::CurrentRealm;

use crate::DomTypes;
#[cfg(not(feature = "js-boa"))]
use crate::interfaces::GlobalScopeHelpers;
use crate::reflector::DomObject;
use crate::script_runtime::JSContext;

pub struct AlreadyInRealm(());

impl AlreadyInRealm {
    #![expect(unsafe_code)]
    #[cfg(not(feature = "js-boa"))]
    pub fn assert<D: DomTypes>() -> AlreadyInRealm {
        unsafe {
            assert!(!GetCurrentRealmOrNull(*D::GlobalScope::get_cx()).is_null());
        }
        AlreadyInRealm(())
    }

    #[cfg(feature = "js-boa")]
    pub fn assert<D: DomTypes>() -> AlreadyInRealm {
        AlreadyInRealm(())
    }

    pub fn assert_for_cx(cx: JSContext) -> AlreadyInRealm {
        #[cfg(not(feature = "js-boa"))]
        unsafe {
            assert!(!GetCurrentRealmOrNull(*cx).is_null());
        }
        #[cfg(feature = "js-boa")]
        let _ = cx; // No realm tracking in the Boa shim.
        AlreadyInRealm(())
    }
}

impl<'a, 'b> From<&'a mut CurrentRealm<'b>> for AlreadyInRealm {
    fn from(_: &'a mut CurrentRealm<'b>) -> AlreadyInRealm {
        AlreadyInRealm(())
    }
}

#[derive(Clone, Copy)]
pub enum InRealm<'a> {
    Already(&'a AlreadyInRealm),
    Entered(&'a JSAutoRealm),
}

impl<'a> From<&'a AlreadyInRealm> for InRealm<'a> {
    fn from(token: &'a AlreadyInRealm) -> InRealm<'a> {
        InRealm::already(token)
    }
}

impl<'a> From<&'a JSAutoRealm> for InRealm<'a> {
    fn from(token: &'a JSAutoRealm) -> InRealm<'a> {
        InRealm::entered(token)
    }
}

impl InRealm<'_> {
    pub fn already(token: &AlreadyInRealm) -> InRealm<'_> {
        InRealm::Already(token)
    }

    pub fn entered(token: &JSAutoRealm) -> InRealm<'_> {
        InRealm::Entered(token)
    }
}

#[cfg(not(feature = "js-boa"))]
pub fn enter_realm<D: DomTypes>(object: &impl DomObject) -> JSAutoRealm {
    JSAutoRealm::new(
        *D::GlobalScope::get_cx(),
        object.reflector().get_jsobject().get(),
    )
}

#[cfg(feature = "js-boa")]
pub fn enter_realm<D: DomTypes>(_object: &impl DomObject) -> JSAutoRealm {
    // Realm management is a no-op in the Boa shim.
    JSAutoRealm::new(std::ptr::null_mut(), std::ptr::null_mut())
}

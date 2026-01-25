/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[cfg(feature = "js-spidermonkey")]
pub(crate) use script_bindings::codegen::InheritTypes::*;
#[cfg(feature = "js-boa")]
pub(crate) use crate::script_bindings::codegen::InheritTypes::*;
#[cfg(feature = "js-spidermonkey")]
pub(crate) use script_bindings::inheritance::{Castable, HasParent};
#[cfg(feature = "js-boa")]
pub(crate) use crate::script_bindings::inheritance::{Castable, HasParent};
// Re-export type IDs for Boa
#[cfg(feature = "js-boa")]
pub(crate) use crate::script_bindings::inheritance::{
    NodeTypeId, CharacterDataTypeId, TextTypeId, ElementTypeId, HTMLElementTypeId, SVGElementTypeId,
    SVGGraphicsElementTypeId, EventTargetTypeId, DocumentFragmentTypeId, HTMLMediaElementTypeId,
    WorkerGlobalScopeTypeId, GlobalScopeTypeId,
};
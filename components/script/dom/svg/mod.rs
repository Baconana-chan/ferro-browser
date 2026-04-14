/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) mod svgelement;
pub(crate) mod svggraphicselement;
pub(crate) mod svgimageelement;
pub(crate) mod svgsvgelement;

// Re-export types for use in dom::types
pub(crate) use svgelement::SVGElement;
pub(crate) use svggraphicselement::SVGGraphicsElement;
pub(crate) use svgimageelement::SVGImageElement;
pub(crate) use svgsvgelement::SVGSVGElement;

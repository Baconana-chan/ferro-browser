/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[expect(clippy::module_inception, reason = "The interface name is Geolocation")]
pub(crate) mod geolocation;
pub(crate) mod geolocationcoordinates;
pub(crate) mod geolocationposition;
pub(crate) mod geolocationpositionerror;

pub(crate) use geolocation::Geolocation;
pub(crate) use geolocationcoordinates::GeolocationCoordinates;
pub(crate) use geolocationposition::GeolocationPosition;
pub(crate) use geolocationpositionerror::GeolocationPositionError;

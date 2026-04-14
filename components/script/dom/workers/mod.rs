/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) mod abstractworker;
pub(crate) mod abstractworkerglobalscope;
pub(crate) mod dedicatedworkerglobalscope;
#[expect(dead_code)]
pub(crate) mod serviceworker;
pub(crate) mod serviceworkercontainer;
pub(crate) mod serviceworkerglobalscope;
#[expect(dead_code)]
pub(crate) mod serviceworkerregistration;
pub(crate) mod worker;
pub(crate) mod workerglobalscope;
pub(crate) mod workerlocation;
pub(crate) mod workernavigator;

// Re-export types for use in dom::types
// Note: AbstractWorkerGlobalScope and AbstractWorker are traits, not structs
pub(crate) use dedicatedworkerglobalscope::DedicatedWorkerGlobalScope;
pub(crate) use serviceworker::ServiceWorker;
pub(crate) use serviceworkercontainer::ServiceWorkerContainer;
pub(crate) use serviceworkerglobalscope::ServiceWorkerGlobalScope;
pub(crate) use serviceworkerregistration::ServiceWorkerRegistration;
pub(crate) use worker::Worker;
pub(crate) use workerglobalscope::WorkerGlobalScope;
pub(crate) use workerlocation::WorkerLocation;
pub(crate) use workernavigator::WorkerNavigator;

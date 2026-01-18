/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/media-source/#mediasource

enum ReadyState {
    "closed",
    "open",
    "ended"
};

enum EndOfStreamError {
    "network",
    "decode"
};

[Exposed=Window, Pref="dom_mediasource_enabled"]
interface MediaSource : EventTarget {
    [Throws] constructor();

    readonly attribute SourceBufferList sourceBuffers;
    readonly attribute SourceBufferList activeSourceBuffers;
    readonly attribute ReadyState readyState;

    attribute unrestricted double duration;

    attribute EventHandler onsourceopen;
    attribute EventHandler onsourceended;
    attribute EventHandler onsourceclose;

    [NewObject, Throws]
    SourceBuffer addSourceBuffer(DOMString type);

    [Throws]
    undefined removeSourceBuffer(SourceBuffer sourceBuffer);

    [Throws]
    undefined endOfStream(optional EndOfStreamError error);

    [Throws]
    undefined setLiveSeekableRange(double start, double end);

    [Throws]
    undefined clearLiveSeekableRange();

    static boolean isTypeSupported(DOMString type);

    // Ferro extension: get object URL for this MediaSource
    static DOMString getObjectURL(MediaSource mediaSource);
};

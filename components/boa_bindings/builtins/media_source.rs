// MIT License - Ferro Browser Project
// MediaSource Extensions API bindings for Boa JavaScript engine

use boa_engine::{
    js_string, Context, JsNativeError, JsObject, JsResult, JsValue, NativeFunction,
    object::ObjectInitializer,
    object::builtins::JsArray,
    property::Attribute,
};
use std::cell::RefCell;

thread_local! {
    static SOURCE_BUFFER_ID: RefCell<u32> = const { RefCell::new(0) };
}

fn next_source_buffer_id() -> u32 {
    SOURCE_BUFFER_ID.with(|id| {
        let mut id = id.borrow_mut();
        *id += 1;
        *id
    })
}

/// Register MediaSource Extensions APIs
pub fn register(ctx: &mut Context) -> JsResult<()> {
    let global = ctx.global_object();

    // Create MediaSource constructor
    let media_source = create_media_source_constructor(ctx)?;
    global.set(js_string!("MediaSource"), media_source, false, ctx)?;

    // Create SourceBuffer (exposed for type checking)
    let source_buffer = create_source_buffer_constructor(ctx)?;
    global.set(js_string!("SourceBuffer"), source_buffer, false, ctx)?;

    // Create SourceBufferList (exposed for type checking)
    let source_buffer_list = create_source_buffer_list_constructor(ctx)?;
    global.set(js_string!("SourceBufferList"), source_buffer_list, false, ctx)?;

    Ok(())
}

/// Create MediaSource constructor
fn create_media_source_constructor(ctx: &mut Context) -> JsResult<JsValue> {
    // Create isTypeSupported static method
    let is_type_supported = NativeFunction::from_fn_ptr(|_this, args, _ctx| {
        let mime_type = args
            .get(0)
            .and_then(|v| v.as_string())
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_default();

        // Support common video/audio types
        let supported = mime_type.starts_with("video/mp4")
            || mime_type.starts_with("video/webm")
            || mime_type.starts_with("audio/mp4")
            || mime_type.starts_with("audio/webm")
            || mime_type.starts_with("audio/mpeg")
            || mime_type.contains("codecs=\"avc1")
            || mime_type.contains("codecs=\"vp8")
            || mime_type.contains("codecs=\"vp9")
            || mime_type.contains("codecs=\"opus")
            || mime_type.contains("codecs=\"mp4a");

        Ok(JsValue::from(supported))
    });

    // MediaSource constructor function
    let constructor = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        // Create source buffers list
        let source_buffers = JsArray::new(ctx);
        let active_source_buffers = JsArray::new(ctx);

        let media_source = ObjectInitializer::new(ctx)
            // readyState: "closed", "open", "ended"
            .property(js_string!("readyState"), js_string!("closed"), Attribute::all())
            // duration (NaN when closed)
            .property(js_string!("duration"), f64::NAN, Attribute::all())
            // sourceBuffers
            .property(js_string!("sourceBuffers"), source_buffers, Attribute::READONLY)
            // activeSourceBuffers
            .property(js_string!("activeSourceBuffers"), active_source_buffers, Attribute::READONLY)
            // Event handlers
            .property(js_string!("onsourceopen"), JsValue::null(), Attribute::all())
            .property(js_string!("onsourceended"), JsValue::null(), Attribute::all())
            .property(js_string!("onsourceclose"), JsValue::null(), Attribute::all())
            .property(js_string!("onerror"), JsValue::null(), Attribute::all())
            .build();

        // Add methods
        add_media_source_methods(&media_source, ctx)?;

        Ok(JsValue::from(media_source))
    });

    let constructor_obj = constructor.to_js_function(ctx.realm());

    // Add static isTypeSupported method
    constructor_obj.set(
        js_string!("isTypeSupported"),
        is_type_supported.to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    Ok(JsValue::from(constructor_obj))
}

/// Add methods to MediaSource instance
fn add_media_source_methods(media_source: &JsObject, ctx: &mut Context) -> JsResult<()> {
    // addSourceBuffer(type)
    let add_source_buffer = NativeFunction::from_fn_ptr(|this, args, ctx| {
        let mime_type = args
            .get(0)
            .and_then(|v| v.as_string())
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_default();

        // Check if type is supported
        let supported = mime_type.starts_with("video/")
            || mime_type.starts_with("audio/");

        if !supported {
            return Err(JsNativeError::typ()
                .with_message(format!("Unsupported MIME type: {}", mime_type))
                .into());
        }

        // Create new SourceBuffer
        let source_buffer = create_source_buffer_instance(ctx, &mime_type)?;

        // Add to sourceBuffers list
        if let Some(this_obj) = this.as_object() {
            if let Ok(buffers) = this_obj.get(js_string!("sourceBuffers"), ctx) {
                if let Some(buffers_obj) = buffers.as_object() {
                    let length = buffers_obj
                        .get(js_string!("length"), ctx)?
                        .to_u32(ctx)
                        .unwrap_or(0);
                    buffers_obj.set(length, source_buffer.clone(), false, ctx)?;
                }
            }
        }

        Ok(source_buffer)
    });

    media_source.set(
        js_string!("addSourceBuffer"),
        add_source_buffer.to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    // removeSourceBuffer(sourceBuffer)
    let remove_source_buffer = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        // Stub: would remove from sourceBuffers list
        Ok(JsValue::undefined())
    });

    media_source.set(
        js_string!("removeSourceBuffer"),
        remove_source_buffer.to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    // endOfStream(error?)
    let end_of_stream = NativeFunction::from_fn_ptr(|this, args, ctx| {
        let _error = args.get(0).cloned();

        // Update readyState to "ended"
        if let Some(this_obj) = this.as_object() {
            this_obj.set(js_string!("readyState"), js_string!("ended"), false, ctx)?;
        }

        Ok(JsValue::undefined())
    });

    media_source.set(
        js_string!("endOfStream"),
        end_of_stream.to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    // setLiveSeekableRange(start, end)
    let set_live_seekable_range = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        // Stub: would set seekable range for live streams
        Ok(JsValue::undefined())
    });

    media_source.set(
        js_string!("setLiveSeekableRange"),
        set_live_seekable_range.to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    // clearLiveSeekableRange()
    let clear_live_seekable_range = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });

    media_source.set(
        js_string!("clearLiveSeekableRange"),
        clear_live_seekable_range.to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    // addEventListener (stub)
    let add_event_listener = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });

    media_source.set(
        js_string!("addEventListener"),
        add_event_listener.to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    // removeEventListener (stub)
    let remove_event_listener = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });

    media_source.set(
        js_string!("removeEventListener"),
        remove_event_listener.to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    Ok(())
}

/// Create SourceBuffer instance
fn create_source_buffer_instance(ctx: &mut Context, mime_type: &str) -> JsResult<JsValue> {
    let id = next_source_buffer_id();

    // Create TimeRanges first to avoid borrow issues
    let buffered = create_time_ranges(ctx, vec![])?;

    let source_buffer = ObjectInitializer::new(ctx)
        // Internal ID
        .property(js_string!("_id"), id, Attribute::READONLY)
        // mode: "segments" or "sequence"
        .property(js_string!("mode"), js_string!("segments"), Attribute::all())
        // updating: true during appendBuffer/remove operations
        .property(js_string!("updating"), false, Attribute::READONLY)
        // buffered: TimeRanges object
        .property(js_string!("buffered"), buffered, Attribute::READONLY)
        // timestampOffset
        .property(js_string!("timestampOffset"), 0.0, Attribute::all())
        // appendWindowStart
        .property(js_string!("appendWindowStart"), 0.0, Attribute::all())
        // appendWindowEnd
        .property(js_string!("appendWindowEnd"), f64::INFINITY, Attribute::all())
        // MIME type (internal)
        .property(js_string!("_mimeType"), js_string!(mime_type), Attribute::READONLY)
        // Event handlers
        .property(js_string!("onupdatestart"), JsValue::null(), Attribute::all())
        .property(js_string!("onupdate"), JsValue::null(), Attribute::all())
        .property(js_string!("onupdateend"), JsValue::null(), Attribute::all())
        .property(js_string!("onerror"), JsValue::null(), Attribute::all())
        .property(js_string!("onabort"), JsValue::null(), Attribute::all())
        .build();

    // Add methods
    add_source_buffer_methods(&source_buffer, ctx)?;

    Ok(JsValue::from(source_buffer))
}

/// Add methods to SourceBuffer instance
fn add_source_buffer_methods(source_buffer: &JsObject, ctx: &mut Context) -> JsResult<()> {
    // appendBuffer(data)
    let append_buffer = NativeFunction::from_fn_ptr(|this, args, ctx| {
        let _data = args.get(0).cloned().unwrap_or(JsValue::undefined());

        // Set updating = true (would be async in real implementation)
        if let Some(this_obj) = this.as_object() {
            // In real implementation, this would:
            // 1. Parse the segment data
            // 2. Add to internal buffer
            // 3. Update buffered TimeRanges
            // 4. Fire updatestart/update/updateend events
            
            // For stub, just acknowledge the call
            let _ = this_obj.set(js_string!("_lastAppend"), js_string!("data"), false, ctx);
        }

        Ok(JsValue::undefined())
    });

    source_buffer.set(
        js_string!("appendBuffer"),
        append_buffer.to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    // abort()
    let abort = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        // Would abort pending appendBuffer operation
        Ok(JsValue::undefined())
    });

    source_buffer.set(
        js_string!("abort"),
        abort.to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    // remove(start, end)
    let remove = NativeFunction::from_fn_ptr(|_this, args, _ctx| {
        let _start = args.get(0).cloned().unwrap_or(JsValue::from(0.0));
        let _end = args.get(1).cloned().unwrap_or(JsValue::from(f64::INFINITY));
        // Would remove buffered data in the range
        Ok(JsValue::undefined())
    });

    source_buffer.set(
        js_string!("remove"),
        remove.to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    // changeType(type)
    let change_type = NativeFunction::from_fn_ptr(|this, args, ctx| {
        let new_type = args
            .get(0)
            .and_then(|v| v.as_string())
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_default();

        if let Some(this_obj) = this.as_object() {
            this_obj.set(js_string!("_mimeType"), js_string!(new_type.as_str()), false, ctx)?;
        }

        Ok(JsValue::undefined())
    });

    source_buffer.set(
        js_string!("changeType"),
        change_type.to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    // addEventListener (stub)
    let add_event_listener = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });

    source_buffer.set(
        js_string!("addEventListener"),
        add_event_listener.to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    // removeEventListener (stub)
    let remove_event_listener = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });

    source_buffer.set(
        js_string!("removeEventListener"),
        remove_event_listener.to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    Ok(())
}

/// Create TimeRanges object
fn create_time_ranges(ctx: &mut Context, _ranges: Vec<(f64, f64)>) -> JsResult<JsValue> {
    // For stub implementation, always return empty TimeRanges
    // Real implementation would store ranges and access them
    let time_ranges = ObjectInitializer::new(ctx)
        .property(js_string!("length"), 0u32, Attribute::READONLY)
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                // Stub: always throw for empty ranges
                Err(JsNativeError::range()
                    .with_message("Index out of bounds")
                    .into())
            }),
            js_string!("start"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                // Stub: always throw for empty ranges
                Err(JsNativeError::range()
                    .with_message("Index out of bounds")
                    .into())
            }),
            js_string!("end"),
            1,
        )
        .build();

    Ok(JsValue::from(time_ranges))
}

/// Create SourceBuffer constructor (for type checking)
fn create_source_buffer_constructor(ctx: &mut Context) -> JsResult<JsValue> {
    let constructor = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Err(JsNativeError::typ()
            .with_message("SourceBuffer cannot be constructed directly")
            .into())
    });

    Ok(JsValue::from(constructor.to_js_function(ctx.realm())))
}

/// Create SourceBufferList constructor (for type checking)
fn create_source_buffer_list_constructor(ctx: &mut Context) -> JsResult<JsValue> {
    let constructor = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Err(JsNativeError::typ()
            .with_message("SourceBufferList cannot be constructed directly")
            .into())
    });

    Ok(JsValue::from(constructor.to_js_function(ctx.realm())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_source_registration() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"typeof MediaSource !== 'undefined'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_media_source_is_type_supported() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"MediaSource.isTypeSupported('video/mp4')"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_media_source_is_type_supported_webm() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"MediaSource.isTypeSupported('video/webm; codecs=\"vp8\"')"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_media_source_unsupported_type() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"MediaSource.isTypeSupported('video/avi')"
        )).unwrap();
        assert_eq!(result.to_boolean(), false);
    }

    #[test]
    fn test_media_source_creation() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var ms = MediaSource();
              ms.readyState === 'closed'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_media_source_source_buffers() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var ms = MediaSource();
              Array.isArray(ms.sourceBuffers)"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_media_source_add_source_buffer() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var ms = MediaSource();
              var sb = ms.addSourceBuffer('video/mp4');
              sb !== null && typeof sb === 'object'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_source_buffer_properties() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var ms = MediaSource();
              var sb = ms.addSourceBuffer('video/mp4');
              sb.mode === 'segments' && 
              sb.updating === false &&
              sb.timestampOffset === 0"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_source_buffer_methods() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var ms = MediaSource();
              var sb = ms.addSourceBuffer('video/mp4');
              typeof sb.appendBuffer === 'function' &&
              typeof sb.abort === 'function' &&
              typeof sb.remove === 'function'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_media_source_end_of_stream() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var ms = MediaSource();
              ms.endOfStream();
              ms.readyState === 'ended'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_source_buffer_constructor_throws() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"try { SourceBuffer(); false; } catch(e) { true; }"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_source_buffer_list_constructor_throws() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"try { SourceBufferList(); false; } catch(e) { true; }"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}

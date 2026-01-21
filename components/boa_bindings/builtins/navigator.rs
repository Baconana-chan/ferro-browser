// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Navigator API for Boa.
//!
//! Provides:
//! - navigator.userAgent: User agent string
//! - navigator.language/languages: Preferred languages
//! - navigator.platform: Platform identifier
//! - navigator.onLine: Network connectivity
//! - navigator.cookieEnabled: Cookie support
//! - navigator.hardwareConcurrency: CPU cores
//! - navigator.clipboard: Clipboard API stub
//! - navigator.geolocation: Geolocation API stub
//! - navigator.permissions: Permissions API stub
//! - navigator.mediaDevices: Media Devices API stub
//! - navigator.serviceWorker: Service Worker stub

use boa_engine::{
    Context, JsArgs, JsObject, JsResult, JsString, JsValue,
    NativeFunction, property::PropertyDescriptor,
    object::builtins::JsArray, object::builtins::JsPromise,
};

/// Ferro Browser version
const FERRO_VERSION: &str = "0.1.0";

/// Default user agent string
fn default_user_agent() -> String {
    format!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) FerroKit/537.36 (KHTML, like Gecko) Ferro/{} Safari/537.36",
        FERRO_VERSION
    )
}

/// Register Navigator API on the global object.
pub fn register(context: &mut Context) -> JsResult<()> {
    let navigator = create_navigator_object(context)?;
    
    context.global_object().define_property_or_throw(
        JsString::from("navigator"),
        PropertyDescriptor::builder()
            .value(navigator)
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(())
}

fn create_navigator_object(context: &mut Context) -> JsResult<JsValue> {
    let navigator = JsObject::with_null_proto();
    
    // userAgent
    navigator.define_property_or_throw(
        JsString::from("userAgent"),
        PropertyDescriptor::builder()
            .value(JsString::from(default_user_agent().as_str()))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // appName
    navigator.define_property_or_throw(
        JsString::from("appName"),
        PropertyDescriptor::builder()
            .value(JsString::from("Netscape"))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // appVersion
    navigator.define_property_or_throw(
        JsString::from("appVersion"),
        PropertyDescriptor::builder()
            .value(JsString::from("5.0 (Windows)"))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // appCodeName
    navigator.define_property_or_throw(
        JsString::from("appCodeName"),
        PropertyDescriptor::builder()
            .value(JsString::from("Mozilla"))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // product
    navigator.define_property_or_throw(
        JsString::from("product"),
        PropertyDescriptor::builder()
            .value(JsString::from("Gecko"))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // productSub
    navigator.define_property_or_throw(
        JsString::from("productSub"),
        PropertyDescriptor::builder()
            .value(JsString::from("20030107"))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // vendor
    navigator.define_property_or_throw(
        JsString::from("vendor"),
        PropertyDescriptor::builder()
            .value(JsString::from("Ferro Browser"))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // vendorSub
    navigator.define_property_or_throw(
        JsString::from("vendorSub"),
        PropertyDescriptor::builder()
            .value(JsString::from(""))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // platform
    let platform = if cfg!(target_os = "windows") {
        "Win32"
    } else if cfg!(target_os = "macos") {
        "MacIntel"
    } else if cfg!(target_os = "linux") {
        "Linux x86_64"
    } else {
        "Unknown"
    };
    navigator.define_property_or_throw(
        JsString::from("platform"),
        PropertyDescriptor::builder()
            .value(JsString::from(platform))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // language
    navigator.define_property_or_throw(
        JsString::from("language"),
        PropertyDescriptor::builder()
            .value(JsString::from("en-US"))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // languages
    let languages = JsArray::new(context);
    languages.set(0, JsValue::from(JsString::from("en-US")), true, context)?;
    languages.set(1, JsValue::from(JsString::from("en")), true, context)?;
    navigator.define_property_or_throw(
        JsString::from("languages"),
        PropertyDescriptor::builder()
            .value(languages)
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // onLine
    navigator.define_property_or_throw(
        JsString::from("onLine"),
        PropertyDescriptor::builder()
            .value(JsValue::from(true))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // cookieEnabled
    navigator.define_property_or_throw(
        JsString::from("cookieEnabled"),
        PropertyDescriptor::builder()
            .value(JsValue::from(true))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // doNotTrack
    navigator.define_property_or_throw(
        JsString::from("doNotTrack"),
        PropertyDescriptor::builder()
            .value(JsString::from("1"))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // hardwareConcurrency
    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4) as f64;
    navigator.define_property_or_throw(
        JsString::from("hardwareConcurrency"),
        PropertyDescriptor::builder()
            .value(JsValue::from(cpu_count))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // maxTouchPoints
    navigator.define_property_or_throw(
        JsString::from("maxTouchPoints"),
        PropertyDescriptor::builder()
            .value(JsValue::from(0))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // webdriver (for bot detection)
    navigator.define_property_or_throw(
        JsString::from("webdriver"),
        PropertyDescriptor::builder()
            .value(JsValue::from(false))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // pdfViewerEnabled
    navigator.define_property_or_throw(
        JsString::from("pdfViewerEnabled"),
        PropertyDescriptor::builder()
            .value(JsValue::from(true))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // javaEnabled()
    let java_enabled = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::from(false))
    });
    navigator.define_property_or_throw(
        JsString::from("javaEnabled"),
        PropertyDescriptor::builder()
            .value(java_enabled.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // sendBeacon()
    let send_beacon = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let url = args.get_or_undefined(0)
            .to_string(ctx)?
            .to_std_string_escaped();
        log::debug!("[Navigator] sendBeacon to: {} (stub)", url);
        Ok(JsValue::from(true))
    });
    navigator.define_property_or_throw(
        JsString::from("sendBeacon"),
        PropertyDescriptor::builder()
            .value(send_beacon.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // vibrate()
    let vibrate = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        log::debug!("[Navigator] vibrate called (stub)");
        Ok(JsValue::from(true))
    });
    navigator.define_property_or_throw(
        JsString::from("vibrate"),
        PropertyDescriptor::builder()
            .value(vibrate.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // clipboard
    let clipboard = create_clipboard_object(context)?;
    navigator.define_property_or_throw(
        JsString::from("clipboard"),
        PropertyDescriptor::builder()
            .value(clipboard)
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // geolocation
    let geolocation = create_geolocation_object(context)?;
    navigator.define_property_or_throw(
        JsString::from("geolocation"),
        PropertyDescriptor::builder()
            .value(geolocation)
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // permissions
    let permissions = create_permissions_object(context)?;
    navigator.define_property_or_throw(
        JsString::from("permissions"),
        PropertyDescriptor::builder()
            .value(permissions)
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // mediaDevices
    let media_devices = create_media_devices_object(context)?;
    navigator.define_property_or_throw(
        JsString::from("mediaDevices"),
        PropertyDescriptor::builder()
            .value(media_devices)
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // serviceWorker
    let service_worker = create_service_worker_container(context)?;
    navigator.define_property_or_throw(
        JsString::from("serviceWorker"),
        PropertyDescriptor::builder()
            .value(service_worker)
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // connection (Network Information API)
    let connection = create_network_information(context)?;
    navigator.define_property_or_throw(
        JsString::from("connection"),
        PropertyDescriptor::builder()
            .value(connection)
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // plugins (empty PluginArray)
    let plugins = JsArray::new(context);
    navigator.define_property_or_throw(
        JsString::from("plugins"),
        PropertyDescriptor::builder()
            .value(plugins)
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // mimeTypes (empty MimeTypeArray)
    let mime_types = JsArray::new(context);
    navigator.define_property_or_throw(
        JsString::from("mimeTypes"),
        PropertyDescriptor::builder()
            .value(mime_types)
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(navigator))
}

fn create_clipboard_object(context: &mut Context) -> JsResult<JsValue> {
    let clipboard = JsObject::with_null_proto();
    
    // readText()
    let read_text = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        log::debug!("[Clipboard] readText called (stub)");
        let promise = JsPromise::resolve(JsValue::from(JsString::from("")), ctx);
        Ok(promise.into())
    });
    clipboard.define_property_or_throw(
        JsString::from("readText"),
        PropertyDescriptor::builder()
            .value(read_text.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // writeText()
    let write_text = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let text = args.get_or_undefined(0)
            .to_string(ctx)?
            .to_std_string_escaped();
        log::debug!("[Clipboard] writeText: {} (stub)", text);
        let promise = JsPromise::resolve(JsValue::undefined(), ctx);
        Ok(promise.into())
    });
    clipboard.define_property_or_throw(
        JsString::from("writeText"),
        PropertyDescriptor::builder()
            .value(write_text.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // read()
    let read = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        log::debug!("[Clipboard] read called (stub)");
        let promise = JsPromise::resolve(JsValue::from(JsArray::new(ctx)), ctx);
        Ok(promise.into())
    });
    clipboard.define_property_or_throw(
        JsString::from("read"),
        PropertyDescriptor::builder()
            .value(read.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // write()
    let write = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        log::debug!("[Clipboard] write called (stub)");
        let promise = JsPromise::resolve(JsValue::undefined(), ctx);
        Ok(promise.into())
    });
    clipboard.define_property_or_throw(
        JsString::from("write"),
        PropertyDescriptor::builder()
            .value(write.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(clipboard))
}

fn create_geolocation_object(context: &mut Context) -> JsResult<JsValue> {
    let geolocation = JsObject::with_null_proto();
    
    // getCurrentPosition()
    let get_current_position = NativeFunction::from_fn_ptr(|_this, args, _ctx| {
        let _success_callback = args.get_or_undefined(0);
        let error_callback = args.get_or_undefined(1);
        
        log::debug!("[Geolocation] getCurrentPosition called (stub - permission denied)");
        
        // Call error callback with permission denied
        if error_callback.is_callable() {
            // In a real implementation, we would call error callback
        }
        
        Ok(JsValue::undefined())
    });
    geolocation.define_property_or_throw(
        JsString::from("getCurrentPosition"),
        PropertyDescriptor::builder()
            .value(get_current_position.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // watchPosition()
    let watch_position = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        log::debug!("[Geolocation] watchPosition called (stub)");
        Ok(JsValue::from(0))
    });
    geolocation.define_property_or_throw(
        JsString::from("watchPosition"),
        PropertyDescriptor::builder()
            .value(watch_position.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // clearWatch()
    let clear_watch = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        log::debug!("[Geolocation] clearWatch called (stub)");
        Ok(JsValue::undefined())
    });
    geolocation.define_property_or_throw(
        JsString::from("clearWatch"),
        PropertyDescriptor::builder()
            .value(clear_watch.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(geolocation))
}

fn create_permissions_object(context: &mut Context) -> JsResult<JsValue> {
    let permissions = JsObject::with_null_proto();
    
    // query()
    let query = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let descriptor = args.get_or_undefined(0);
        let name = if let Some(obj) = descriptor.as_object() {
            obj.get(JsString::from("name"), ctx)
                .map(|v| v.to_string(ctx).map(|s| s.to_std_string_escaped()).unwrap_or_default())
                .unwrap_or_default()
        } else {
            String::new()
        };
        
        log::debug!("[Permissions] query for: {} (stub)", name);
        
        // Create PermissionStatus object
        let status = JsObject::with_null_proto();
        status.define_property_or_throw(
            JsString::from("state"),
            PropertyDescriptor::builder()
                .value(JsString::from("prompt"))
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
            ctx,
        )?;
        status.define_property_or_throw(
            JsString::from("onchange"),
            PropertyDescriptor::builder()
                .value(JsValue::null())
                .writable(true)
                .enumerable(true)
                .configurable(true)
                .build(),
            ctx,
        )?;
        
        let promise = JsPromise::resolve(JsValue::from(status), ctx);
        Ok(promise.into())
    });
    permissions.define_property_or_throw(
        JsString::from("query"),
        PropertyDescriptor::builder()
            .value(query.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(permissions))
}

fn create_media_devices_object(context: &mut Context) -> JsResult<JsValue> {
    let media_devices = JsObject::with_null_proto();
    
    // enumerateDevices()
    let enumerate_devices = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        log::debug!("[MediaDevices] enumerateDevices called (stub)");
        let promise = JsPromise::resolve(JsValue::from(JsArray::new(ctx)), ctx);
        Ok(promise.into())
    });
    media_devices.define_property_or_throw(
        JsString::from("enumerateDevices"),
        PropertyDescriptor::builder()
            .value(enumerate_devices.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // getUserMedia()
    let get_user_media = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        log::debug!("[MediaDevices] getUserMedia called (stub - not allowed)");
        let (promise, resolvers) = JsPromise::new_pending(ctx);
        let error = JsValue::from(JsString::from("NotAllowedError: Permission denied"));
        resolvers.reject.call(&JsValue::undefined(), &[error], ctx)?;
        Ok(promise.into())
    });
    media_devices.define_property_or_throw(
        JsString::from("getUserMedia"),
        PropertyDescriptor::builder()
            .value(get_user_media.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // getDisplayMedia()
    let get_display_media = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        log::debug!("[MediaDevices] getDisplayMedia called (stub - not allowed)");
        let (promise, resolvers) = JsPromise::new_pending(ctx);
        let error = JsValue::from(JsString::from("NotAllowedError: Permission denied"));
        resolvers.reject.call(&JsValue::undefined(), &[error], ctx)?;
        Ok(promise.into())
    });
    media_devices.define_property_or_throw(
        JsString::from("getDisplayMedia"),
        PropertyDescriptor::builder()
            .value(get_display_media.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // ondevicechange
    media_devices.define_property_or_throw(
        JsString::from("ondevicechange"),
        PropertyDescriptor::builder()
            .value(JsValue::null())
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(media_devices))
}

fn create_service_worker_container(context: &mut Context) -> JsResult<JsValue> {
    let container = JsObject::with_null_proto();
    
    // controller (null - no active service worker)
    container.define_property_or_throw(
        JsString::from("controller"),
        PropertyDescriptor::builder()
            .value(JsValue::null())
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // ready (promise that resolves to registration)
    let ready_promise = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        log::debug!("[ServiceWorker] ready accessed (stub)");
        // Return a pending promise (never resolves in stub)
        let (promise, _) = JsPromise::new_pending(ctx);
        Ok(promise.into())
    });
    container.define_property_or_throw(
        JsString::from("ready"),
        PropertyDescriptor::builder()
            .value(ready_promise.to_js_function(context.realm()))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // register()
    let register = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let script_url = args.get_or_undefined(0)
            .to_string(ctx)?
            .to_std_string_escaped();
        log::debug!("[ServiceWorker] register: {} (stub)", script_url);
        
        // Return a promise that rejects (not supported)
        let (promise, resolvers) = JsPromise::new_pending(ctx);
        let error = JsValue::from(JsString::from("SecurityError: Service workers are not supported"));
        resolvers.reject.call(&JsValue::undefined(), &[error], ctx)?;
        Ok(promise.into())
    });
    container.define_property_or_throw(
        JsString::from("register"),
        PropertyDescriptor::builder()
            .value(register.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // getRegistration()
    let get_registration = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        log::debug!("[ServiceWorker] getRegistration called (stub)");
        let promise = JsPromise::resolve(JsValue::undefined(), ctx);
        Ok(promise.into())
    });
    container.define_property_or_throw(
        JsString::from("getRegistration"),
        PropertyDescriptor::builder()
            .value(get_registration.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // getRegistrations()
    let get_registrations = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        log::debug!("[ServiceWorker] getRegistrations called (stub)");
        let promise = JsPromise::resolve(JsValue::from(JsArray::new(ctx)), ctx);
        Ok(promise.into())
    });
    container.define_property_or_throw(
        JsString::from("getRegistrations"),
        PropertyDescriptor::builder()
            .value(get_registrations.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(container))
}

fn create_network_information(context: &mut Context) -> JsResult<JsValue> {
    let connection = JsObject::with_null_proto();
    
    // effectiveType
    connection.define_property_or_throw(
        JsString::from("effectiveType"),
        PropertyDescriptor::builder()
            .value(JsString::from("4g"))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // type
    connection.define_property_or_throw(
        JsString::from("type"),
        PropertyDescriptor::builder()
            .value(JsString::from("wifi"))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // downlink (estimated bandwidth in Mbps)
    connection.define_property_or_throw(
        JsString::from("downlink"),
        PropertyDescriptor::builder()
            .value(JsValue::from(10.0))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // rtt (round-trip time in ms)
    connection.define_property_or_throw(
        JsString::from("rtt"),
        PropertyDescriptor::builder()
            .value(JsValue::from(50))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // saveData
    connection.define_property_or_throw(
        JsString::from("saveData"),
        PropertyDescriptor::builder()
            .value(JsValue::from(false))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // onchange
    connection.define_property_or_throw(
        JsString::from("onchange"),
        PropertyDescriptor::builder()
            .value(JsValue::null())
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(connection))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;
    
    #[test]
    fn test_navigator_registration() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof navigator"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "object");
    }
    
    #[test]
    fn test_navigator_user_agent() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "navigator.userAgent.includes('Ferro')"
        )).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
    
    #[test]
    fn test_navigator_platform() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof navigator.platform"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "string");
    }
    
    #[test]
    fn test_navigator_language() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "navigator.language"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "en-US");
    }
    
    #[test]
    fn test_navigator_languages() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "Array.isArray(navigator.languages)"
        )).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
    
    #[test]
    fn test_navigator_online() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "navigator.onLine"
        )).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
    
    #[test]
    fn test_navigator_hardware_concurrency() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "navigator.hardwareConcurrency > 0"
        )).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
    
    #[test]
    fn test_navigator_clipboard() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(r#"
            typeof navigator.clipboard === 'object' &&
            typeof navigator.clipboard.readText === 'function' &&
            typeof navigator.clipboard.writeText === 'function'
        "#)).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
    
    #[test]
    fn test_navigator_geolocation() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof navigator.geolocation.getCurrentPosition === 'function'"
        )).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
    
    #[test]
    fn test_navigator_permissions() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof navigator.permissions.query === 'function'"
        )).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
    
    #[test]
    fn test_navigator_media_devices() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(r#"
            typeof navigator.mediaDevices === 'object' &&
            typeof navigator.mediaDevices.enumerateDevices === 'function' &&
            typeof navigator.mediaDevices.getUserMedia === 'function'
        "#)).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
    
    #[test]
    fn test_navigator_service_worker() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(r#"
            typeof navigator.serviceWorker === 'object' &&
            typeof navigator.serviceWorker.register === 'function'
        "#)).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
    
    #[test]
    fn test_navigator_connection() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "navigator.connection.effectiveType === '4g'"
        )).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
}

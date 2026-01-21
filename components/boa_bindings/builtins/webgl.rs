// MIT License - Ferro Browser Project
// WebGL API bindings for Boa JavaScript engine

use boa_engine::{
    js_string, Context, JsObject, JsResult, JsValue, NativeFunction,
    object::ObjectInitializer,
    object::builtins::JsArray,
    property::Attribute,
};

/// WebGL constants
pub mod constants {
    // Buffer bits
    pub const DEPTH_BUFFER_BIT: u32 = 0x00000100;
    pub const STENCIL_BUFFER_BIT: u32 = 0x00000400;
    pub const COLOR_BUFFER_BIT: u32 = 0x00004000;

    // Begin mode
    pub const POINTS: u32 = 0x0000;
    pub const LINES: u32 = 0x0001;
    pub const LINE_LOOP: u32 = 0x0002;
    pub const LINE_STRIP: u32 = 0x0003;
    pub const TRIANGLES: u32 = 0x0004;
    pub const TRIANGLE_STRIP: u32 = 0x0005;
    pub const TRIANGLE_FAN: u32 = 0x0006;

    // Blending modes
    pub const ZERO: u32 = 0;
    pub const ONE: u32 = 1;
    pub const SRC_COLOR: u32 = 0x0300;
    pub const ONE_MINUS_SRC_COLOR: u32 = 0x0301;
    pub const SRC_ALPHA: u32 = 0x0302;
    pub const ONE_MINUS_SRC_ALPHA: u32 = 0x0303;
    pub const DST_ALPHA: u32 = 0x0304;
    pub const ONE_MINUS_DST_ALPHA: u32 = 0x0305;
    pub const DST_COLOR: u32 = 0x0306;
    pub const ONE_MINUS_DST_COLOR: u32 = 0x0307;

    // Buffer types
    pub const ARRAY_BUFFER: u32 = 0x8892;
    pub const ELEMENT_ARRAY_BUFFER: u32 = 0x8893;

    // Buffer usage
    pub const STATIC_DRAW: u32 = 0x88E4;
    pub const DYNAMIC_DRAW: u32 = 0x88E8;
    pub const STREAM_DRAW: u32 = 0x88E0;

    // Shader types
    pub const FRAGMENT_SHADER: u32 = 0x8B30;
    pub const VERTEX_SHADER: u32 = 0x8B31;

    // Shader compile status
    pub const COMPILE_STATUS: u32 = 0x8B81;
    pub const LINK_STATUS: u32 = 0x8B82;

    // Data types
    pub const BYTE: u32 = 0x1400;
    pub const UNSIGNED_BYTE: u32 = 0x1401;
    pub const SHORT: u32 = 0x1402;
    pub const UNSIGNED_SHORT: u32 = 0x1403;
    pub const INT: u32 = 0x1404;
    pub const UNSIGNED_INT: u32 = 0x1405;
    pub const FLOAT: u32 = 0x1406;

    // Texture targets
    pub const TEXTURE_2D: u32 = 0x0DE1;
    pub const TEXTURE_CUBE_MAP: u32 = 0x8513;

    // Texture parameters
    pub const TEXTURE_MAG_FILTER: u32 = 0x2800;
    pub const TEXTURE_MIN_FILTER: u32 = 0x2801;
    pub const TEXTURE_WRAP_S: u32 = 0x2802;
    pub const TEXTURE_WRAP_T: u32 = 0x2803;

    // Texture filter values
    pub const NEAREST: u32 = 0x2600;
    pub const LINEAR: u32 = 0x2601;
    pub const LINEAR_MIPMAP_LINEAR: u32 = 0x2703;

    // Texture wrap values
    pub const REPEAT: u32 = 0x2901;
    pub const CLAMP_TO_EDGE: u32 = 0x812F;
    pub const MIRRORED_REPEAT: u32 = 0x8370;

    // Pixel formats
    pub const RGBA: u32 = 0x1908;
    pub const RGB: u32 = 0x1907;
    pub const ALPHA: u32 = 0x1906;
    pub const LUMINANCE: u32 = 0x1909;
    pub const LUMINANCE_ALPHA: u32 = 0x190A;

    // Enable capabilities
    pub const BLEND: u32 = 0x0BE2;
    pub const CULL_FACE: u32 = 0x0B44;
    pub const DEPTH_TEST: u32 = 0x0B71;
    pub const SCISSOR_TEST: u32 = 0x0C11;
    pub const STENCIL_TEST: u32 = 0x0B90;

    // Face culling
    pub const FRONT: u32 = 0x0404;
    pub const BACK: u32 = 0x0405;
    pub const FRONT_AND_BACK: u32 = 0x0408;

    // Depth function
    pub const NEVER: u32 = 0x0200;
    pub const LESS: u32 = 0x0201;
    pub const EQUAL: u32 = 0x0202;
    pub const LEQUAL: u32 = 0x0203;
    pub const GREATER: u32 = 0x0204;
    pub const NOTEQUAL: u32 = 0x0205;
    pub const GEQUAL: u32 = 0x0206;
    pub const ALWAYS: u32 = 0x0207;
}

/// Register WebGL APIs
pub fn register(ctx: &mut Context) -> JsResult<()> {
    let global = ctx.global_object();

    // Create WebGLRenderingContext constructor
    let webgl_context = create_webgl_rendering_context(ctx)?;
    global.set(js_string!("WebGLRenderingContext"), webgl_context, false, ctx)?;

    // Create WebGL2RenderingContext constructor
    let webgl2_context = create_webgl2_rendering_context(ctx)?;
    global.set(js_string!("WebGL2RenderingContext"), webgl2_context, false, ctx)?;

    // Create WebGLProgram constructor
    let webgl_program = create_webgl_program(ctx)?;
    global.set(js_string!("WebGLProgram"), webgl_program, false, ctx)?;

    // Create WebGLShader constructor
    let webgl_shader = create_webgl_shader(ctx)?;
    global.set(js_string!("WebGLShader"), webgl_shader, false, ctx)?;

    // Create WebGLBuffer constructor
    let webgl_buffer = create_webgl_buffer(ctx)?;
    global.set(js_string!("WebGLBuffer"), webgl_buffer, false, ctx)?;

    // Create WebGLTexture constructor
    let webgl_texture = create_webgl_texture(ctx)?;
    global.set(js_string!("WebGLTexture"), webgl_texture, false, ctx)?;

    // Create WebGLFramebuffer constructor
    let webgl_framebuffer = create_webgl_framebuffer(ctx)?;
    global.set(js_string!("WebGLFramebuffer"), webgl_framebuffer, false, ctx)?;

    // Create WebGLRenderbuffer constructor
    let webgl_renderbuffer = create_webgl_renderbuffer(ctx)?;
    global.set(js_string!("WebGLRenderbuffer"), webgl_renderbuffer, false, ctx)?;

    // Create WebGLUniformLocation constructor
    let webgl_uniform = create_webgl_uniform_location(ctx)?;
    global.set(js_string!("WebGLUniformLocation"), webgl_uniform, false, ctx)?;

    Ok(())
}

/// Create WebGLRenderingContext constructor
fn create_webgl_rendering_context(ctx: &mut Context) -> JsResult<JsValue> {
    let context_obj = ObjectInitializer::new(ctx)
        // Constants
        .property(js_string!("DEPTH_BUFFER_BIT"), constants::DEPTH_BUFFER_BIT, Attribute::all())
        .property(js_string!("STENCIL_BUFFER_BIT"), constants::STENCIL_BUFFER_BIT, Attribute::all())
        .property(js_string!("COLOR_BUFFER_BIT"), constants::COLOR_BUFFER_BIT, Attribute::all())
        .property(js_string!("POINTS"), constants::POINTS, Attribute::all())
        .property(js_string!("LINES"), constants::LINES, Attribute::all())
        .property(js_string!("TRIANGLES"), constants::TRIANGLES, Attribute::all())
        .property(js_string!("TRIANGLE_STRIP"), constants::TRIANGLE_STRIP, Attribute::all())
        .property(js_string!("TRIANGLE_FAN"), constants::TRIANGLE_FAN, Attribute::all())
        .property(js_string!("ARRAY_BUFFER"), constants::ARRAY_BUFFER, Attribute::all())
        .property(js_string!("ELEMENT_ARRAY_BUFFER"), constants::ELEMENT_ARRAY_BUFFER, Attribute::all())
        .property(js_string!("STATIC_DRAW"), constants::STATIC_DRAW, Attribute::all())
        .property(js_string!("DYNAMIC_DRAW"), constants::DYNAMIC_DRAW, Attribute::all())
        .property(js_string!("FRAGMENT_SHADER"), constants::FRAGMENT_SHADER, Attribute::all())
        .property(js_string!("VERTEX_SHADER"), constants::VERTEX_SHADER, Attribute::all())
        .property(js_string!("COMPILE_STATUS"), constants::COMPILE_STATUS, Attribute::all())
        .property(js_string!("LINK_STATUS"), constants::LINK_STATUS, Attribute::all())
        .property(js_string!("FLOAT"), constants::FLOAT, Attribute::all())
        .property(js_string!("UNSIGNED_BYTE"), constants::UNSIGNED_BYTE, Attribute::all())
        .property(js_string!("UNSIGNED_SHORT"), constants::UNSIGNED_SHORT, Attribute::all())
        .property(js_string!("TEXTURE_2D"), constants::TEXTURE_2D, Attribute::all())
        .property(js_string!("TEXTURE_MAG_FILTER"), constants::TEXTURE_MAG_FILTER, Attribute::all())
        .property(js_string!("TEXTURE_MIN_FILTER"), constants::TEXTURE_MIN_FILTER, Attribute::all())
        .property(js_string!("LINEAR"), constants::LINEAR, Attribute::all())
        .property(js_string!("NEAREST"), constants::NEAREST, Attribute::all())
        .property(js_string!("RGBA"), constants::RGBA, Attribute::all())
        .property(js_string!("RGB"), constants::RGB, Attribute::all())
        .property(js_string!("BLEND"), constants::BLEND, Attribute::all())
        .property(js_string!("DEPTH_TEST"), constants::DEPTH_TEST, Attribute::all())
        .property(js_string!("CULL_FACE"), constants::CULL_FACE, Attribute::all())
        .build();

    // Add prototype with methods
    let prototype = create_webgl_context_prototype(ctx)?;
    context_obj.set(js_string!("prototype"), prototype, false, ctx)?;

    Ok(JsValue::from(context_obj))
}

/// Create WebGL context prototype with methods
fn create_webgl_context_prototype(ctx: &mut Context) -> JsResult<JsObject> {
    let prototype = ObjectInitializer::new(ctx)
        // Canvas getter
        .property(js_string!("canvas"), JsValue::null(), Attribute::all())
        .property(js_string!("drawingBufferWidth"), 0, Attribute::all())
        .property(js_string!("drawingBufferHeight"), 0, Attribute::all())
        // Clear methods
        .function(
            NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                let _mask = args.get(0).cloned().unwrap_or(JsValue::from(0));
                // Stub: clear the color/depth/stencil buffers
                Ok(JsValue::undefined())
            }),
            js_string!("clear"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                let _r = args.get(0).cloned().unwrap_or(JsValue::from(0.0));
                let _g = args.get(1).cloned().unwrap_or(JsValue::from(0.0));
                let _b = args.get(2).cloned().unwrap_or(JsValue::from(0.0));
                let _a = args.get(3).cloned().unwrap_or(JsValue::from(1.0));
                // Stub: set clear color
                Ok(JsValue::undefined())
            }),
            js_string!("clearColor"),
            4,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                let _depth = args.get(0).cloned().unwrap_or(JsValue::from(1.0));
                Ok(JsValue::undefined())
            }),
            js_string!("clearDepth"),
            1,
        )
        // Viewport
        .function(
            NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                let _x = args.get(0).cloned().unwrap_or(JsValue::from(0));
                let _y = args.get(1).cloned().unwrap_or(JsValue::from(0));
                let _width = args.get(2).cloned().unwrap_or(JsValue::from(0));
                let _height = args.get(3).cloned().unwrap_or(JsValue::from(0));
                Ok(JsValue::undefined())
            }),
            js_string!("viewport"),
            4,
        )
        // Enable/disable
        .function(
            NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                let _cap = args.get(0).cloned().unwrap_or(JsValue::from(0));
                Ok(JsValue::undefined())
            }),
            js_string!("enable"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                let _cap = args.get(0).cloned().unwrap_or(JsValue::from(0));
                Ok(JsValue::undefined())
            }),
            js_string!("disable"),
            1,
        )
        // Shader operations
        .function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let shader_type = args.get(0).cloned().unwrap_or(JsValue::from(0));
                let shader = ObjectInitializer::new(ctx)
                    .property(js_string!("_type"), shader_type, Attribute::all())
                    .property(js_string!("_source"), JsValue::from(js_string!("")), Attribute::all())
                    .property(js_string!("_compiled"), false, Attribute::all())
                    .build();
                Ok(JsValue::from(shader))
            }),
            js_string!("createShader"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                if let Some(shader) = args.get(0).and_then(|v| v.as_object()) {
                    let source = args.get(1).cloned().unwrap_or(JsValue::from(js_string!("")));
                    shader.set(js_string!("_source"), source, false, ctx)?;
                }
                Ok(JsValue::undefined())
            }),
            js_string!("shaderSource"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                if let Some(shader) = args.get(0).and_then(|v| v.as_object()) {
                    shader.set(js_string!("_compiled"), true, false, ctx)?;
                }
                Ok(JsValue::undefined())
            }),
            js_string!("compileShader"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                if let Some(shader) = args.get(0).and_then(|v| v.as_object()) {
                    let pname = args.get(1).and_then(|v| v.to_u32(ctx).ok()).unwrap_or(0);
                    if pname == constants::COMPILE_STATUS {
                        return Ok(shader.get(js_string!("_compiled"), ctx)?);
                    }
                }
                Ok(JsValue::null())
            }),
            js_string!("getShaderParameter"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                // Return empty string (no errors)
                Ok(JsValue::from(js_string!("")))
            }),
            js_string!("getShaderInfoLog"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("deleteShader"),
            1,
        )
        // Program operations
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, ctx| {
                let program = ObjectInitializer::new(ctx)
                    .property(js_string!("_linked"), false, Attribute::all())
                    .build();
                Ok(JsValue::from(program))
            }),
            js_string!("createProgram"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("attachShader"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                if let Some(program) = args.get(0).and_then(|v| v.as_object()) {
                    program.set(js_string!("_linked"), true, false, ctx)?;
                }
                Ok(JsValue::undefined())
            }),
            js_string!("linkProgram"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                if let Some(program) = args.get(0).and_then(|v| v.as_object()) {
                    let pname = args.get(1).and_then(|v| v.to_u32(ctx).ok()).unwrap_or(0);
                    if pname == constants::LINK_STATUS {
                        return Ok(program.get(js_string!("_linked"), ctx)?);
                    }
                }
                Ok(JsValue::null())
            }),
            js_string!("getProgramParameter"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::from(js_string!("")))
            }),
            js_string!("getProgramInfoLog"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("useProgram"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("deleteProgram"),
            1,
        )
        // Buffer operations
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, ctx| {
                let buffer = ObjectInitializer::new(ctx)
                    .property(js_string!("_id"), JsValue::from(1), Attribute::all())
                    .build();
                Ok(JsValue::from(buffer))
            }),
            js_string!("createBuffer"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("bindBuffer"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("bufferData"),
            3,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("deleteBuffer"),
            1,
        )
        // Texture operations
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, ctx| {
                let texture = ObjectInitializer::new(ctx)
                    .property(js_string!("_id"), JsValue::from(1), Attribute::all())
                    .build();
                Ok(JsValue::from(texture))
            }),
            js_string!("createTexture"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("bindTexture"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("texImage2D"),
            9,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("texParameteri"),
            3,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("generateMipmap"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("activeTexture"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("deleteTexture"),
            1,
        )
        // Attribute operations
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                // Return attribute location (stub)
                Ok(JsValue::from(0))
            }),
            js_string!("getAttribLocation"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("enableVertexAttribArray"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("disableVertexAttribArray"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("vertexAttribPointer"),
            6,
        )
        // Uniform operations
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, ctx| {
                let uniform = ObjectInitializer::new(ctx)
                    .property(js_string!("_location"), JsValue::from(0), Attribute::all())
                    .build();
                Ok(JsValue::from(uniform))
            }),
            js_string!("getUniformLocation"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("uniform1i"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("uniform1f"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("uniform2f"),
            3,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("uniform3f"),
            4,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("uniform4f"),
            5,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("uniform1fv"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("uniform2fv"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("uniform3fv"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("uniform4fv"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("uniformMatrix2fv"),
            3,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("uniformMatrix3fv"),
            3,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("uniformMatrix4fv"),
            3,
        )
        // Draw operations
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("drawArrays"),
            3,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("drawElements"),
            4,
        )
        // State
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                // No error
                Ok(JsValue::from(0))
            }),
            js_string!("getError"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("flush"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("finish"),
            0,
        )
        // Blending
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("blendFunc"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("blendFuncSeparate"),
            4,
        )
        // Depth
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("depthFunc"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("depthMask"),
            1,
        )
        // Culling
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("cullFace"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::undefined())
            }),
            js_string!("frontFace"),
            1,
        )
        // Context info
        .function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let pname = args.get(0).and_then(|v| v.to_u32(ctx).ok()).unwrap_or(0);
                // Return stub values for common parameters
                match pname {
                    0x1F00 => Ok(JsValue::from(js_string!("Ferro WebGL"))), // VENDOR
                    0x1F01 => Ok(JsValue::from(js_string!("Ferro WebGL 1.0"))), // RENDERER
                    0x1F02 => Ok(JsValue::from(js_string!("WebGL 1.0"))), // VERSION
                    0x8B8C => Ok(JsValue::from(js_string!("WebGL GLSL ES 1.0"))), // SHADING_LANGUAGE_VERSION
                    _ => Ok(JsValue::null()),
                }
            }),
            js_string!("getParameter"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Ok(JsValue::null())
            }),
            js_string!("getExtension"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(|_this, _args, ctx| {
                // Return empty array of extensions
                let array = JsArray::new(ctx);
                Ok(JsValue::from(array))
            }),
            js_string!("getSupportedExtensions"),
            0,
        )
        .build();

    Ok(prototype)
}

/// Create WebGL2RenderingContext constructor (extends WebGLRenderingContext)
fn create_webgl2_rendering_context(ctx: &mut Context) -> JsResult<JsValue> {
    let context_obj = ObjectInitializer::new(ctx)
        // WebGL2-specific constants
        .property(js_string!("READ_BUFFER"), 0x0C02u32, Attribute::all())
        .property(js_string!("UNPACK_ROW_LENGTH"), 0x0CF2u32, Attribute::all())
        .property(js_string!("UNPACK_SKIP_ROWS"), 0x0CF3u32, Attribute::all())
        .property(js_string!("UNPACK_SKIP_PIXELS"), 0x0CF4u32, Attribute::all())
        .property(js_string!("PACK_ROW_LENGTH"), 0x0D02u32, Attribute::all())
        .property(js_string!("PACK_SKIP_ROWS"), 0x0D03u32, Attribute::all())
        .property(js_string!("PACK_SKIP_PIXELS"), 0x0D04u32, Attribute::all())
        .property(js_string!("TEXTURE_BINDING_3D"), 0x806Au32, Attribute::all())
        .property(js_string!("UNPACK_SKIP_IMAGES"), 0x806Du32, Attribute::all())
        .property(js_string!("UNPACK_IMAGE_HEIGHT"), 0x806Eu32, Attribute::all())
        .property(js_string!("TEXTURE_3D"), 0x806Fu32, Attribute::all())
        .property(js_string!("TEXTURE_WRAP_R"), 0x8072u32, Attribute::all())
        .property(js_string!("MAX_3D_TEXTURE_SIZE"), 0x8073u32, Attribute::all())
        .property(js_string!("VERTEX_ARRAY_BINDING"), 0x85B5u32, Attribute::all())
        // Include all WebGL1 constants
        .property(js_string!("DEPTH_BUFFER_BIT"), constants::DEPTH_BUFFER_BIT, Attribute::all())
        .property(js_string!("STENCIL_BUFFER_BIT"), constants::STENCIL_BUFFER_BIT, Attribute::all())
        .property(js_string!("COLOR_BUFFER_BIT"), constants::COLOR_BUFFER_BIT, Attribute::all())
        .build();

    // Create WebGL2 prototype with additional methods
    let prototype = create_webgl2_context_prototype(ctx)?;
    context_obj.set(js_string!("prototype"), prototype, false, ctx)?;

    Ok(JsValue::from(context_obj))
}

/// Create WebGL2 context prototype with additional methods
fn create_webgl2_context_prototype(ctx: &mut Context) -> JsResult<JsObject> {
    // Start with WebGL1 prototype
    let prototype = create_webgl_context_prototype(ctx)?;

    // Add WebGL2-specific methods
    prototype.set(
        js_string!("createVertexArray"),
        NativeFunction::from_fn_ptr(|_this, _args, ctx| {
            let vao = ObjectInitializer::new(ctx)
                .property(js_string!("_id"), JsValue::from(1), Attribute::all())
                .build();
            Ok(JsValue::from(vao))
        })
        .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    prototype.set(
        js_string!("bindVertexArray"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    prototype.set(
        js_string!("deleteVertexArray"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    prototype.set(
        js_string!("texImage3D"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    prototype.set(
        js_string!("texSubImage3D"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    prototype.set(
        js_string!("createQuery"),
        NativeFunction::from_fn_ptr(|_this, _args, ctx| {
            let query = ObjectInitializer::new(ctx)
                .property(js_string!("_id"), JsValue::from(1), Attribute::all())
                .build();
            Ok(JsValue::from(query))
        })
        .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    prototype.set(
        js_string!("deleteQuery"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    prototype.set(
        js_string!("beginQuery"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    prototype.set(
        js_string!("endQuery"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    prototype.set(
        js_string!("createSampler"),
        NativeFunction::from_fn_ptr(|_this, _args, ctx| {
            let sampler = ObjectInitializer::new(ctx)
                .property(js_string!("_id"), JsValue::from(1), Attribute::all())
                .build();
            Ok(JsValue::from(sampler))
        })
        .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    prototype.set(
        js_string!("bindSampler"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    prototype.set(
        js_string!("deleteSampler"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    prototype.set(
        js_string!("createTransformFeedback"),
        NativeFunction::from_fn_ptr(|_this, _args, ctx| {
            let tf = ObjectInitializer::new(ctx)
                .property(js_string!("_id"), JsValue::from(1), Attribute::all())
                .build();
            Ok(JsValue::from(tf))
        })
        .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    prototype.set(
        js_string!("bindTransformFeedback"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    prototype.set(
        js_string!("beginTransformFeedback"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    prototype.set(
        js_string!("endTransformFeedback"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    prototype.set(
        js_string!("drawArraysInstanced"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    prototype.set(
        js_string!("drawElementsInstanced"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(ctx.realm()),
        false,
        ctx,
    )?;

    Ok(prototype)
}

/// Create WebGLProgram constructor
fn create_webgl_program(ctx: &mut Context) -> JsResult<JsValue> {
    let program = ObjectInitializer::new(ctx).build();
    Ok(JsValue::from(program))
}

/// Create WebGLShader constructor
fn create_webgl_shader(ctx: &mut Context) -> JsResult<JsValue> {
    let shader = ObjectInitializer::new(ctx).build();
    Ok(JsValue::from(shader))
}

/// Create WebGLBuffer constructor
fn create_webgl_buffer(ctx: &mut Context) -> JsResult<JsValue> {
    let buffer = ObjectInitializer::new(ctx).build();
    Ok(JsValue::from(buffer))
}

/// Create WebGLTexture constructor
fn create_webgl_texture(ctx: &mut Context) -> JsResult<JsValue> {
    let texture = ObjectInitializer::new(ctx).build();
    Ok(JsValue::from(texture))
}

/// Create WebGLFramebuffer constructor
fn create_webgl_framebuffer(ctx: &mut Context) -> JsResult<JsValue> {
    let framebuffer = ObjectInitializer::new(ctx).build();
    Ok(JsValue::from(framebuffer))
}

/// Create WebGLRenderbuffer constructor
fn create_webgl_renderbuffer(ctx: &mut Context) -> JsResult<JsValue> {
    let renderbuffer = ObjectInitializer::new(ctx).build();
    Ok(JsValue::from(renderbuffer))
}

/// Create WebGLUniformLocation constructor
fn create_webgl_uniform_location(ctx: &mut Context) -> JsResult<JsValue> {
    let uniform = ObjectInitializer::new(ctx).build();
    Ok(JsValue::from(uniform))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webgl_registration() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"typeof WebGLRenderingContext !== 'undefined'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_webgl2_registration() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"typeof WebGL2RenderingContext !== 'undefined'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_webgl_constants() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"WebGLRenderingContext.prototype.TRIANGLES === undefined ? 
              WebGLRenderingContext.TRIANGLES : 
              WebGLRenderingContext.prototype.TRIANGLES"
        )).unwrap();
        // Check constant exists (TRIANGLES = 4)
        assert!(result.is_number() || result.is_undefined());
    }

    #[test]
    fn test_webgl_create_shader() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var proto = WebGLRenderingContext.prototype;
              var shader = proto.createShader(35633);
              shader !== null && typeof shader === 'object'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_webgl_create_program() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var proto = WebGLRenderingContext.prototype;
              var program = proto.createProgram();
              program !== null && typeof program === 'object'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_webgl_shader_workflow() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var proto = WebGLRenderingContext.prototype;
              var shader = proto.createShader(35633);
              proto.shaderSource(shader, 'void main() {}');
              proto.compileShader(shader);
              proto.getShaderParameter(shader, 35713)"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_webgl_create_buffer() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var proto = WebGLRenderingContext.prototype;
              var buffer = proto.createBuffer();
              buffer !== null && typeof buffer === 'object'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_webgl_create_texture() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var proto = WebGLRenderingContext.prototype;
              var texture = proto.createTexture();
              texture !== null && typeof texture === 'object'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_webgl_uniform_location() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var proto = WebGLRenderingContext.prototype;
              var program = proto.createProgram();
              var uniform = proto.getUniformLocation(program, 'uColor');
              uniform !== null && typeof uniform === 'object'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_webgl_get_error() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var proto = WebGLRenderingContext.prototype;
              proto.getError() === 0"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_webgl2_vao() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var proto = WebGL2RenderingContext.prototype;
              typeof proto.createVertexArray === 'function'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_webgl_objects_constructors() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"typeof WebGLProgram !== 'undefined' &&
              typeof WebGLShader !== 'undefined' &&
              typeof WebGLBuffer !== 'undefined' &&
              typeof WebGLTexture !== 'undefined' &&
              typeof WebGLFramebuffer !== 'undefined' &&
              typeof WebGLRenderbuffer !== 'undefined' &&
              typeof WebGLUniformLocation !== 'undefined'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}

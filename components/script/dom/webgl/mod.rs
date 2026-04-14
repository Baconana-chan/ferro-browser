/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) mod extensions;
pub(crate) mod validations;
pub(crate) mod vertexarrayobject;
pub(crate) mod webgl2renderingcontext;
pub(crate) mod webglactiveinfo;
pub(crate) mod webglbuffer;
pub(crate) mod webglcontextevent;
pub(crate) mod webglframebuffer;
pub(crate) mod webglobject;
pub(crate) mod webglprogram;
pub(crate) mod webglquery;
pub(crate) mod webglrenderbuffer;
pub(crate) mod webglrenderingcontext;
pub(crate) mod webglsampler;
pub(crate) mod webglshader;
pub(crate) mod webglshaderprecisionformat;
pub(crate) mod webglsync;
pub(crate) mod webgltexture;
pub(crate) mod webgltransformfeedback;
pub(crate) mod webgluniformlocation;
pub(crate) mod webglvertexarrayobject;
pub(crate) mod webglvertexarrayobjectoes;

// Re-export types for use in dom::types
pub(crate) use webgl2renderingcontext::WebGL2RenderingContext;
pub(crate) use webglactiveinfo::WebGLActiveInfo;
pub(crate) use webglbuffer::WebGLBuffer;
pub(crate) use webglcontextevent::WebGLContextEvent;
pub(crate) use webglframebuffer::WebGLFramebuffer;
pub(crate) use webglobject::WebGLObject;
pub(crate) use webglprogram::WebGLProgram;
pub(crate) use webglquery::WebGLQuery;
pub(crate) use webglrenderbuffer::WebGLRenderbuffer;
pub(crate) use webglrenderingcontext::WebGLRenderingContext;
pub(crate) use webglsampler::WebGLSampler;
pub(crate) use webglshader::WebGLShader;
pub(crate) use webglshaderprecisionformat::WebGLShaderPrecisionFormat;
pub(crate) use webglsync::WebGLSync;
pub(crate) use webgltexture::WebGLTexture;
pub(crate) use webgltransformfeedback::WebGLTransformFeedback;
pub(crate) use webgluniformlocation::WebGLUniformLocation;
pub(crate) use webglvertexarrayobject::WebGLVertexArrayObject;
pub(crate) use webglvertexarrayobjectoes::WebGLVertexArrayObjectOES;

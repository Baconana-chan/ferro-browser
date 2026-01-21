// MIT License - Ferro Browser Project
// Web Audio API bindings for Boa JavaScript engine

use boa_engine::{
    js_string, Context, JsNativeError, JsObject, JsResult, JsValue, NativeFunction,
    object::ObjectInitializer,
    object::builtins::{JsArray, JsPromise},
    property::Attribute,
};
use std::cell::RefCell;
use std::time::{SystemTime, UNIX_EPOCH};

thread_local! {
    static AUDIO_CONTEXT_ID: RefCell<u32> = const { RefCell::new(0) };
}

fn next_context_id() -> u32 {
    AUDIO_CONTEXT_ID.with(|id| {
        let mut id = id.borrow_mut();
        *id += 1;
        *id
    })
}

fn current_time_seconds() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Register Web Audio API
pub fn register(ctx: &mut Context) -> JsResult<()> {
    let global = ctx.global_object();

    // AudioContext
    let audio_context = create_audio_context_constructor(ctx)?;
    global.set(js_string!("AudioContext"), audio_context, false, ctx)?;

    // OfflineAudioContext
    let offline_audio_context = create_offline_audio_context_constructor(ctx)?;
    global.set(js_string!("OfflineAudioContext"), offline_audio_context, false, ctx)?;

    // AudioBuffer
    let audio_buffer = create_audio_buffer_constructor(ctx)?;
    global.set(js_string!("AudioBuffer"), audio_buffer, false, ctx)?;

    // AudioNode types (for type checking)
    let audio_node = create_audio_node_constructor(ctx)?;
    global.set(js_string!("AudioNode"), audio_node, false, ctx)?;

    let gain_node = create_gain_node_constructor(ctx)?;
    global.set(js_string!("GainNode"), gain_node, false, ctx)?;

    let oscillator_node = create_oscillator_node_constructor(ctx)?;
    global.set(js_string!("OscillatorNode"), oscillator_node, false, ctx)?;

    let analyser_node = create_analyser_node_constructor(ctx)?;
    global.set(js_string!("AnalyserNode"), analyser_node, false, ctx)?;

    let biquad_filter_node = create_biquad_filter_node_constructor(ctx)?;
    global.set(js_string!("BiquadFilterNode"), biquad_filter_node, false, ctx)?;

    let delay_node = create_delay_node_constructor(ctx)?;
    global.set(js_string!("DelayNode"), delay_node, false, ctx)?;

    let dynamics_compressor_node = create_dynamics_compressor_node_constructor(ctx)?;
    global.set(js_string!("DynamicsCompressorNode"), dynamics_compressor_node, false, ctx)?;

    let panner_node = create_panner_node_constructor(ctx)?;
    global.set(js_string!("PannerNode"), panner_node, false, ctx)?;

    let convolver_node = create_convolver_node_constructor(ctx)?;
    global.set(js_string!("ConvolverNode"), convolver_node, false, ctx)?;

    Ok(())
}

/// Create AudioContext constructor
fn create_audio_context_constructor(ctx: &mut Context) -> JsResult<JsValue> {
    let constructor = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let options = args.get(0).and_then(|v| v.as_object());
        
        // Parse options
        let sample_rate = options.as_ref()
            .and_then(|o| o.get(js_string!("sampleRate"), ctx).ok())
            .and_then(|v| v.to_number(ctx).ok())
            .unwrap_or(44100.0);

        let latency_hint = options.as_ref()
            .and_then(|o| o.get(js_string!("latencyHint"), ctx).ok())
            .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()))
            .unwrap_or_else(|| "interactive".to_string());

        let context_id = next_context_id();
        let base_time = current_time_seconds();

        // Create destination node
        let destination = create_audio_destination_node(ctx)?;

        let audio_context = ObjectInitializer::new(ctx)
            .property(js_string!("_id"), context_id, Attribute::READONLY)
            .property(js_string!("_baseTime"), base_time, Attribute::READONLY)
            .property(js_string!("sampleRate"), sample_rate, Attribute::READONLY)
            .property(js_string!("baseLatency"), 0.01, Attribute::READONLY)
            .property(js_string!("outputLatency"), 0.02, Attribute::READONLY)
            .property(js_string!("state"), js_string!("running"), Attribute::READONLY)
            .property(js_string!("destination"), destination, Attribute::READONLY)
            .property(js_string!("_latencyHint"), js_string!(latency_hint.as_str()), Attribute::READONLY)
            // Event handlers
            .property(js_string!("onstatechange"), JsValue::null(), Attribute::all())
            .build();

        // Add currentTime getter
        add_audio_context_properties(&audio_context, ctx)?;
        // Add methods
        add_audio_context_methods(&audio_context, ctx)?;

        Ok(JsValue::from(audio_context))
    });

    Ok(JsValue::from(constructor.to_js_function(ctx.realm())))
}

/// Add AudioContext properties (getters)
fn add_audio_context_properties(context: &JsObject, ctx: &mut Context) -> JsResult<()> {
    // currentTime getter would ideally use Object.defineProperty
    // For now, we set an initial value that scripts can check
    context.set(js_string!("currentTime"), 0.0, false, ctx)?;
    
    Ok(())
}

/// Add AudioContext methods
fn add_audio_context_methods(context: &JsObject, ctx: &mut Context) -> JsResult<()> {
    // createGain()
    let create_gain = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        create_gain_node_instance(ctx)
    });
    context.set(js_string!("createGain"), create_gain.to_js_function(ctx.realm()), false, ctx)?;

    // createOscillator()
    let create_oscillator = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        create_oscillator_node_instance(ctx)
    });
    context.set(js_string!("createOscillator"), create_oscillator.to_js_function(ctx.realm()), false, ctx)?;

    // createAnalyser()
    let create_analyser = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        create_analyser_node_instance(ctx)
    });
    context.set(js_string!("createAnalyser"), create_analyser.to_js_function(ctx.realm()), false, ctx)?;

    // createBiquadFilter()
    let create_biquad_filter = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        create_biquad_filter_node_instance(ctx)
    });
    context.set(js_string!("createBiquadFilter"), create_biquad_filter.to_js_function(ctx.realm()), false, ctx)?;

    // createDelay(maxDelayTime)
    let create_delay = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let max_delay = args.get(0).and_then(|v| v.to_number(ctx).ok()).unwrap_or(1.0);
        create_delay_node_instance(ctx, max_delay)
    });
    context.set(js_string!("createDelay"), create_delay.to_js_function(ctx.realm()), false, ctx)?;

    // createDynamicsCompressor()
    let create_compressor = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        create_dynamics_compressor_node_instance(ctx)
    });
    context.set(js_string!("createDynamicsCompressor"), create_compressor.to_js_function(ctx.realm()), false, ctx)?;

    // createPanner()
    let create_panner = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        create_panner_node_instance(ctx)
    });
    context.set(js_string!("createPanner"), create_panner.to_js_function(ctx.realm()), false, ctx)?;

    // createConvolver()
    let create_convolver = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        create_convolver_node_instance(ctx)
    });
    context.set(js_string!("createConvolver"), create_convolver.to_js_function(ctx.realm()), false, ctx)?;

    // createBufferSource()
    let create_buffer_source = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        create_buffer_source_node_instance(ctx)
    });
    context.set(js_string!("createBufferSource"), create_buffer_source.to_js_function(ctx.realm()), false, ctx)?;

    // createBuffer(numberOfChannels, length, sampleRate)
    let create_buffer = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let channels = args.get(0).and_then(|v| v.to_u32(ctx).ok()).unwrap_or(2);
        let length = args.get(1).and_then(|v| v.to_u32(ctx).ok()).unwrap_or(0);
        let sample_rate = args.get(2).and_then(|v| v.to_number(ctx).ok()).unwrap_or(44100.0);
        create_audio_buffer_instance(ctx, channels, length, sample_rate)
    });
    context.set(js_string!("createBuffer"), create_buffer.to_js_function(ctx.realm()), false, ctx)?;

    // decodeAudioData(arrayBuffer, successCallback, errorCallback)
    let decode_audio_data = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let _array_buffer = args.get(0).cloned();
        let _success_callback = args.get(1).cloned();
        let _error_callback = args.get(2).cloned();
        
        // Return a Promise that resolves to an AudioBuffer
        // For stub, return a promise that resolves to empty buffer
        let promise = JsPromise::resolve(
            create_audio_buffer_instance(ctx, 2, 44100, 44100.0)?,
            ctx,
        );
        Ok(JsValue::from(promise))
    });
    context.set(js_string!("decodeAudioData"), decode_audio_data.to_js_function(ctx.realm()), false, ctx)?;

    // suspend()
    let suspend = NativeFunction::from_fn_ptr(|this, _args, ctx| {
        if let Some(this_obj) = this.as_object() {
            this_obj.set(js_string!("state"), js_string!("suspended"), false, ctx)?;
        }
        let promise = JsPromise::resolve(JsValue::undefined(), ctx);
        Ok(JsValue::from(promise))
    });
    context.set(js_string!("suspend"), suspend.to_js_function(ctx.realm()), false, ctx)?;

    // resume()
    let resume = NativeFunction::from_fn_ptr(|this, _args, ctx| {
        if let Some(this_obj) = this.as_object() {
            this_obj.set(js_string!("state"), js_string!("running"), false, ctx)?;
        }
        let promise = JsPromise::resolve(JsValue::undefined(), ctx);
        Ok(JsValue::from(promise))
    });
    context.set(js_string!("resume"), resume.to_js_function(ctx.realm()), false, ctx)?;

    // close()
    let close = NativeFunction::from_fn_ptr(|this, _args, ctx| {
        if let Some(this_obj) = this.as_object() {
            this_obj.set(js_string!("state"), js_string!("closed"), false, ctx)?;
        }
        let promise = JsPromise::resolve(JsValue::undefined(), ctx);
        Ok(JsValue::from(promise))
    });
    context.set(js_string!("close"), close.to_js_function(ctx.realm()), false, ctx)?;

    Ok(())
}

/// Create AudioDestinationNode
fn create_audio_destination_node(ctx: &mut Context) -> JsResult<JsValue> {
    let destination = ObjectInitializer::new(ctx)
        .property(js_string!("maxChannelCount"), 2, Attribute::READONLY)
        .property(js_string!("channelCount"), 2, Attribute::all())
        .property(js_string!("channelCountMode"), js_string!("explicit"), Attribute::all())
        .property(js_string!("channelInterpretation"), js_string!("speakers"), Attribute::all())
        .property(js_string!("numberOfInputs"), 1, Attribute::READONLY)
        .property(js_string!("numberOfOutputs"), 0, Attribute::READONLY)
        .build();
    
    Ok(JsValue::from(destination))
}

/// Create GainNode instance
fn create_gain_node_instance(ctx: &mut Context) -> JsResult<JsValue> {
    let gain_param = create_audio_param(ctx, "gain", 1.0, 0.0, 3.4028235e38)?;

    let gain_node = ObjectInitializer::new(ctx)
        .property(js_string!("gain"), gain_param, Attribute::READONLY)
        .property(js_string!("channelCount"), 2, Attribute::all())
        .property(js_string!("channelCountMode"), js_string!("max"), Attribute::all())
        .property(js_string!("channelInterpretation"), js_string!("speakers"), Attribute::all())
        .property(js_string!("numberOfInputs"), 1, Attribute::READONLY)
        .property(js_string!("numberOfOutputs"), 1, Attribute::READONLY)
        .build();

    add_audio_node_methods(&gain_node, ctx)?;
    Ok(JsValue::from(gain_node))
}

/// Create OscillatorNode instance
fn create_oscillator_node_instance(ctx: &mut Context) -> JsResult<JsValue> {
    let frequency_param = create_audio_param(ctx, "frequency", 440.0, -22050.0, 22050.0)?;
    let detune_param = create_audio_param(ctx, "detune", 0.0, -153600.0, 153600.0)?;

    let oscillator = ObjectInitializer::new(ctx)
        .property(js_string!("type"), js_string!("sine"), Attribute::all())
        .property(js_string!("frequency"), frequency_param, Attribute::READONLY)
        .property(js_string!("detune"), detune_param, Attribute::READONLY)
        .property(js_string!("channelCount"), 2, Attribute::all())
        .property(js_string!("numberOfInputs"), 0, Attribute::READONLY)
        .property(js_string!("numberOfOutputs"), 1, Attribute::READONLY)
        .property(js_string!("onended"), JsValue::null(), Attribute::all())
        .build();

    add_audio_node_methods(&oscillator, ctx)?;

    // start(when)
    let start = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });
    oscillator.set(js_string!("start"), start.to_js_function(ctx.realm()), false, ctx)?;

    // stop(when)
    let stop = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });
    oscillator.set(js_string!("stop"), stop.to_js_function(ctx.realm()), false, ctx)?;

    // setPeriodicWave(periodicWave)
    let set_periodic_wave = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });
    oscillator.set(js_string!("setPeriodicWave"), set_periodic_wave.to_js_function(ctx.realm()), false, ctx)?;

    Ok(JsValue::from(oscillator))
}

/// Create AnalyserNode instance
fn create_analyser_node_instance(ctx: &mut Context) -> JsResult<JsValue> {
    let analyser = ObjectInitializer::new(ctx)
        .property(js_string!("fftSize"), 2048, Attribute::all())
        .property(js_string!("frequencyBinCount"), 1024, Attribute::READONLY)
        .property(js_string!("minDecibels"), -100.0, Attribute::all())
        .property(js_string!("maxDecibels"), -30.0, Attribute::all())
        .property(js_string!("smoothingTimeConstant"), 0.8, Attribute::all())
        .property(js_string!("channelCount"), 2, Attribute::all())
        .property(js_string!("numberOfInputs"), 1, Attribute::READONLY)
        .property(js_string!("numberOfOutputs"), 1, Attribute::READONLY)
        .build();

    add_audio_node_methods(&analyser, ctx)?;

    // getFloatFrequencyData(array)
    let get_float_frequency_data = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });
    analyser.set(js_string!("getFloatFrequencyData"), get_float_frequency_data.to_js_function(ctx.realm()), false, ctx)?;

    // getByteFrequencyData(array)
    let get_byte_frequency_data = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });
    analyser.set(js_string!("getByteFrequencyData"), get_byte_frequency_data.to_js_function(ctx.realm()), false, ctx)?;

    // getFloatTimeDomainData(array)
    let get_float_time_domain_data = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });
    analyser.set(js_string!("getFloatTimeDomainData"), get_float_time_domain_data.to_js_function(ctx.realm()), false, ctx)?;

    // getByteTimeDomainData(array)
    let get_byte_time_domain_data = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });
    analyser.set(js_string!("getByteTimeDomainData"), get_byte_time_domain_data.to_js_function(ctx.realm()), false, ctx)?;

    Ok(JsValue::from(analyser))
}

/// Create BiquadFilterNode instance
fn create_biquad_filter_node_instance(ctx: &mut Context) -> JsResult<JsValue> {
    let frequency_param = create_audio_param(ctx, "frequency", 350.0, 10.0, 22050.0)?;
    let detune_param = create_audio_param(ctx, "detune", 0.0, -153600.0, 153600.0)?;
    let q_param = create_audio_param(ctx, "Q", 1.0, 0.0001, 1000.0)?;
    let gain_param = create_audio_param(ctx, "gain", 0.0, -40.0, 40.0)?;

    let filter = ObjectInitializer::new(ctx)
        .property(js_string!("type"), js_string!("lowpass"), Attribute::all())
        .property(js_string!("frequency"), frequency_param, Attribute::READONLY)
        .property(js_string!("detune"), detune_param, Attribute::READONLY)
        .property(js_string!("Q"), q_param, Attribute::READONLY)
        .property(js_string!("gain"), gain_param, Attribute::READONLY)
        .property(js_string!("channelCount"), 2, Attribute::all())
        .property(js_string!("numberOfInputs"), 1, Attribute::READONLY)
        .property(js_string!("numberOfOutputs"), 1, Attribute::READONLY)
        .build();

    add_audio_node_methods(&filter, ctx)?;

    // getFrequencyResponse(frequencyHz, magResponse, phaseResponse)
    let get_frequency_response = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });
    filter.set(js_string!("getFrequencyResponse"), get_frequency_response.to_js_function(ctx.realm()), false, ctx)?;

    Ok(JsValue::from(filter))
}

/// Create DelayNode instance
fn create_delay_node_instance(ctx: &mut Context, max_delay: f64) -> JsResult<JsValue> {
    let delay_time_param = create_audio_param(ctx, "delayTime", 0.0, 0.0, max_delay)?;

    let delay = ObjectInitializer::new(ctx)
        .property(js_string!("delayTime"), delay_time_param, Attribute::READONLY)
        .property(js_string!("channelCount"), 2, Attribute::all())
        .property(js_string!("numberOfInputs"), 1, Attribute::READONLY)
        .property(js_string!("numberOfOutputs"), 1, Attribute::READONLY)
        .build();

    add_audio_node_methods(&delay, ctx)?;
    Ok(JsValue::from(delay))
}

/// Create DynamicsCompressorNode instance
fn create_dynamics_compressor_node_instance(ctx: &mut Context) -> JsResult<JsValue> {
    let threshold_param = create_audio_param(ctx, "threshold", -24.0, -100.0, 0.0)?;
    let knee_param = create_audio_param(ctx, "knee", 30.0, 0.0, 40.0)?;
    let ratio_param = create_audio_param(ctx, "ratio", 12.0, 1.0, 20.0)?;
    let attack_param = create_audio_param(ctx, "attack", 0.003, 0.0, 1.0)?;
    let release_param = create_audio_param(ctx, "release", 0.25, 0.0, 1.0)?;

    let compressor = ObjectInitializer::new(ctx)
        .property(js_string!("threshold"), threshold_param, Attribute::READONLY)
        .property(js_string!("knee"), knee_param, Attribute::READONLY)
        .property(js_string!("ratio"), ratio_param, Attribute::READONLY)
        .property(js_string!("attack"), attack_param, Attribute::READONLY)
        .property(js_string!("release"), release_param, Attribute::READONLY)
        .property(js_string!("reduction"), 0.0, Attribute::READONLY)
        .property(js_string!("channelCount"), 2, Attribute::all())
        .property(js_string!("numberOfInputs"), 1, Attribute::READONLY)
        .property(js_string!("numberOfOutputs"), 1, Attribute::READONLY)
        .build();

    add_audio_node_methods(&compressor, ctx)?;
    Ok(JsValue::from(compressor))
}

/// Create PannerNode instance
fn create_panner_node_instance(ctx: &mut Context) -> JsResult<JsValue> {
    let position_x = create_audio_param(ctx, "positionX", 0.0, -3.4028235e38, 3.4028235e38)?;
    let position_y = create_audio_param(ctx, "positionY", 0.0, -3.4028235e38, 3.4028235e38)?;
    let position_z = create_audio_param(ctx, "positionZ", 0.0, -3.4028235e38, 3.4028235e38)?;
    let orientation_x = create_audio_param(ctx, "orientationX", 1.0, -3.4028235e38, 3.4028235e38)?;
    let orientation_y = create_audio_param(ctx, "orientationY", 0.0, -3.4028235e38, 3.4028235e38)?;
    let orientation_z = create_audio_param(ctx, "orientationZ", 0.0, -3.4028235e38, 3.4028235e38)?;

    let panner = ObjectInitializer::new(ctx)
        .property(js_string!("panningModel"), js_string!("equalpower"), Attribute::all())
        .property(js_string!("distanceModel"), js_string!("inverse"), Attribute::all())
        .property(js_string!("positionX"), position_x, Attribute::READONLY)
        .property(js_string!("positionY"), position_y, Attribute::READONLY)
        .property(js_string!("positionZ"), position_z, Attribute::READONLY)
        .property(js_string!("orientationX"), orientation_x, Attribute::READONLY)
        .property(js_string!("orientationY"), orientation_y, Attribute::READONLY)
        .property(js_string!("orientationZ"), orientation_z, Attribute::READONLY)
        .property(js_string!("refDistance"), 1.0, Attribute::all())
        .property(js_string!("maxDistance"), 10000.0, Attribute::all())
        .property(js_string!("rolloffFactor"), 1.0, Attribute::all())
        .property(js_string!("coneInnerAngle"), 360.0, Attribute::all())
        .property(js_string!("coneOuterAngle"), 360.0, Attribute::all())
        .property(js_string!("coneOuterGain"), 0.0, Attribute::all())
        .property(js_string!("channelCount"), 2, Attribute::all())
        .property(js_string!("numberOfInputs"), 1, Attribute::READONLY)
        .property(js_string!("numberOfOutputs"), 1, Attribute::READONLY)
        .build();

    add_audio_node_methods(&panner, ctx)?;

    // setPosition (deprecated but still used)
    let set_position = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });
    panner.set(js_string!("setPosition"), set_position.to_js_function(ctx.realm()), false, ctx)?;

    // setOrientation (deprecated but still used)
    let set_orientation = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });
    panner.set(js_string!("setOrientation"), set_orientation.to_js_function(ctx.realm()), false, ctx)?;

    Ok(JsValue::from(panner))
}

/// Create ConvolverNode instance
fn create_convolver_node_instance(ctx: &mut Context) -> JsResult<JsValue> {
    let convolver = ObjectInitializer::new(ctx)
        .property(js_string!("buffer"), JsValue::null(), Attribute::all())
        .property(js_string!("normalize"), true, Attribute::all())
        .property(js_string!("channelCount"), 2, Attribute::all())
        .property(js_string!("numberOfInputs"), 1, Attribute::READONLY)
        .property(js_string!("numberOfOutputs"), 1, Attribute::READONLY)
        .build();

    add_audio_node_methods(&convolver, ctx)?;
    Ok(JsValue::from(convolver))
}

/// Create AudioBufferSourceNode instance
fn create_buffer_source_node_instance(ctx: &mut Context) -> JsResult<JsValue> {
    let playback_rate = create_audio_param(ctx, "playbackRate", 1.0, -3.4028235e38, 3.4028235e38)?;
    let detune = create_audio_param(ctx, "detune", 0.0, -153600.0, 153600.0)?;

    let buffer_source = ObjectInitializer::new(ctx)
        .property(js_string!("buffer"), JsValue::null(), Attribute::all())
        .property(js_string!("playbackRate"), playback_rate, Attribute::READONLY)
        .property(js_string!("detune"), detune, Attribute::READONLY)
        .property(js_string!("loop"), false, Attribute::all())
        .property(js_string!("loopStart"), 0.0, Attribute::all())
        .property(js_string!("loopEnd"), 0.0, Attribute::all())
        .property(js_string!("channelCount"), 2, Attribute::all())
        .property(js_string!("numberOfInputs"), 0, Attribute::READONLY)
        .property(js_string!("numberOfOutputs"), 1, Attribute::READONLY)
        .property(js_string!("onended"), JsValue::null(), Attribute::all())
        .build();

    add_audio_node_methods(&buffer_source, ctx)?;

    // start(when, offset, duration)
    let start = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });
    buffer_source.set(js_string!("start"), start.to_js_function(ctx.realm()), false, ctx)?;

    // stop(when)
    let stop = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });
    buffer_source.set(js_string!("stop"), stop.to_js_function(ctx.realm()), false, ctx)?;

    Ok(JsValue::from(buffer_source))
}

/// Create AudioParam
fn create_audio_param(ctx: &mut Context, _name: &str, default_value: f64, min_value: f64, max_value: f64) -> JsResult<JsValue> {
    let audio_param = ObjectInitializer::new(ctx)
        .property(js_string!("value"), default_value, Attribute::all())
        .property(js_string!("defaultValue"), default_value, Attribute::READONLY)
        .property(js_string!("minValue"), min_value, Attribute::READONLY)
        .property(js_string!("maxValue"), max_value, Attribute::READONLY)
        .property(js_string!("automationRate"), js_string!("a-rate"), Attribute::all())
        .build();

    // Add automation methods
    let set_value_at_time = NativeFunction::from_fn_ptr(|this, args, ctx| {
        let value = args.get(0).and_then(|v| v.to_number(ctx).ok()).unwrap_or(0.0);
        let _start_time = args.get(1).and_then(|v| v.to_number(ctx).ok()).unwrap_or(0.0);
        if let Some(this_obj) = this.as_object() {
            this_obj.set(js_string!("value"), value, false, ctx)?;
        }
        Ok(this.clone())
    });
    audio_param.set(js_string!("setValueAtTime"), set_value_at_time.to_js_function(ctx.realm()), false, ctx)?;

    let linear_ramp_to_value_at_time = NativeFunction::from_fn_ptr(|this, _args, _ctx| {
        Ok(this.clone())
    });
    audio_param.set(js_string!("linearRampToValueAtTime"), linear_ramp_to_value_at_time.to_js_function(ctx.realm()), false, ctx)?;

    let exponential_ramp_to_value_at_time = NativeFunction::from_fn_ptr(|this, _args, _ctx| {
        Ok(this.clone())
    });
    audio_param.set(js_string!("exponentialRampToValueAtTime"), exponential_ramp_to_value_at_time.to_js_function(ctx.realm()), false, ctx)?;

    let set_target_at_time = NativeFunction::from_fn_ptr(|this, _args, _ctx| {
        Ok(this.clone())
    });
    audio_param.set(js_string!("setTargetAtTime"), set_target_at_time.to_js_function(ctx.realm()), false, ctx)?;

    let set_value_curve_at_time = NativeFunction::from_fn_ptr(|this, _args, _ctx| {
        Ok(this.clone())
    });
    audio_param.set(js_string!("setValueCurveAtTime"), set_value_curve_at_time.to_js_function(ctx.realm()), false, ctx)?;

    let cancel_scheduled_values = NativeFunction::from_fn_ptr(|this, _args, _ctx| {
        Ok(this.clone())
    });
    audio_param.set(js_string!("cancelScheduledValues"), cancel_scheduled_values.to_js_function(ctx.realm()), false, ctx)?;

    let cancel_and_hold_at_time = NativeFunction::from_fn_ptr(|this, _args, _ctx| {
        Ok(this.clone())
    });
    audio_param.set(js_string!("cancelAndHoldAtTime"), cancel_and_hold_at_time.to_js_function(ctx.realm()), false, ctx)?;

    Ok(JsValue::from(audio_param))
}

/// Add common AudioNode methods
fn add_audio_node_methods(node: &JsObject, ctx: &mut Context) -> JsResult<()> {
    // connect(destination, output, input)
    let connect = NativeFunction::from_fn_ptr(|_this, args, _ctx| {
        // Return the destination for chaining
        Ok(args.get(0).cloned().unwrap_or(JsValue::undefined()))
    });
    node.set(js_string!("connect"), connect.to_js_function(ctx.realm()), false, ctx)?;

    // disconnect(destination, output, input)
    let disconnect = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });
    node.set(js_string!("disconnect"), disconnect.to_js_function(ctx.realm()), false, ctx)?;

    Ok(())
}

/// Create AudioBuffer instance
fn create_audio_buffer_instance(ctx: &mut Context, channels: u32, length: u32, sample_rate: f64) -> JsResult<JsValue> {
    let duration = if sample_rate > 0.0 { length as f64 / sample_rate } else { 0.0 };

    let audio_buffer = ObjectInitializer::new(ctx)
        .property(js_string!("sampleRate"), sample_rate, Attribute::READONLY)
        .property(js_string!("length"), length, Attribute::READONLY)
        .property(js_string!("duration"), duration, Attribute::READONLY)
        .property(js_string!("numberOfChannels"), channels, Attribute::READONLY)
        .property(js_string!("_channels"), channels, Attribute::READONLY)
        .build();

    // getChannelData(channel) - simplified without move closure
    let get_channel_data = NativeFunction::from_fn_ptr(|this, args, ctx| {
        let channel = args.get(0).and_then(|v| v.to_u32(ctx).ok()).unwrap_or(0);
        // Get numberOfChannels from this
        let num_channels = this.as_object()
            .and_then(|o| o.get(js_string!("_channels"), ctx).ok())
            .and_then(|v| v.to_u32(ctx).ok())
            .unwrap_or(2);
        if channel >= num_channels {
            return Err(JsNativeError::range()
                .with_message("Channel index out of bounds")
                .into());
        }
        // Return empty Float32Array (stub)
        let array = JsArray::new(ctx);
        Ok(JsValue::from(array))
    });
    audio_buffer.set(js_string!("getChannelData"), get_channel_data.to_js_function(ctx.realm()), false, ctx)?;

    // copyFromChannel(destination, channelNumber, bufferOffset)
    let copy_from_channel = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });
    audio_buffer.set(js_string!("copyFromChannel"), copy_from_channel.to_js_function(ctx.realm()), false, ctx)?;

    // copyToChannel(source, channelNumber, bufferOffset)
    let copy_to_channel = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::undefined())
    });
    audio_buffer.set(js_string!("copyToChannel"), copy_to_channel.to_js_function(ctx.realm()), false, ctx)?;

    Ok(JsValue::from(audio_buffer))
}

/// Create OfflineAudioContext constructor
fn create_offline_audio_context_constructor(ctx: &mut Context) -> JsResult<JsValue> {
    let constructor = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let channels = args.get(0).and_then(|v| v.to_u32(ctx).ok()).unwrap_or(2);
        let length = args.get(1).and_then(|v| v.to_u32(ctx).ok()).unwrap_or(0);
        let sample_rate = args.get(2).and_then(|v| v.to_number(ctx).ok()).unwrap_or(44100.0);

        let destination = create_audio_destination_node(ctx)?;

        let offline_context = ObjectInitializer::new(ctx)
            .property(js_string!("sampleRate"), sample_rate, Attribute::READONLY)
            .property(js_string!("length"), length, Attribute::READONLY)
            .property(js_string!("state"), js_string!("suspended"), Attribute::READONLY)
            .property(js_string!("destination"), destination, Attribute::READONLY)
            .property(js_string!("_channels"), channels, Attribute::READONLY)
            .property(js_string!("oncomplete"), JsValue::null(), Attribute::all())
            .build();

        add_audio_context_methods(&offline_context, ctx)?;

        // startRendering()
        let start_rendering = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
            // Return Promise that resolves to AudioBuffer
            let buffer = create_audio_buffer_instance(ctx, 2, 44100, 44100.0)?;
            let promise = JsPromise::resolve(buffer, ctx);
            Ok(JsValue::from(promise))
        });
        offline_context.set(js_string!("startRendering"), start_rendering.to_js_function(ctx.realm()), false, ctx)?;

        Ok(JsValue::from(offline_context))
    });

    Ok(JsValue::from(constructor.to_js_function(ctx.realm())))
}

/// Create AudioBuffer constructor
fn create_audio_buffer_constructor(ctx: &mut Context) -> JsResult<JsValue> {
    let constructor = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let options = args.get(0).and_then(|v| v.as_object());
        
        let channels = options.as_ref()
            .and_then(|o| o.get(js_string!("numberOfChannels"), ctx).ok())
            .and_then(|v| v.to_u32(ctx).ok())
            .unwrap_or(2);
        let length = options.as_ref()
            .and_then(|o| o.get(js_string!("length"), ctx).ok())
            .and_then(|v| v.to_u32(ctx).ok())
            .unwrap_or(0);
        let sample_rate = options.as_ref()
            .and_then(|o| o.get(js_string!("sampleRate"), ctx).ok())
            .and_then(|v| v.to_number(ctx).ok())
            .unwrap_or(44100.0);

        create_audio_buffer_instance(ctx, channels, length, sample_rate)
    });

    Ok(JsValue::from(constructor.to_js_function(ctx.realm())))
}

// Constructor stubs for type checking
fn create_audio_node_constructor(ctx: &mut Context) -> JsResult<JsValue> {
    let constructor = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Err(JsNativeError::typ()
            .with_message("AudioNode cannot be constructed directly")
            .into())
    });
    Ok(JsValue::from(constructor.to_js_function(ctx.realm())))
}

fn create_gain_node_constructor(ctx: &mut Context) -> JsResult<JsValue> {
    let constructor = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let _context = args.get(0);
        create_gain_node_instance(ctx)
    });
    Ok(JsValue::from(constructor.to_js_function(ctx.realm())))
}

fn create_oscillator_node_constructor(ctx: &mut Context) -> JsResult<JsValue> {
    let constructor = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let _context = args.get(0);
        create_oscillator_node_instance(ctx)
    });
    Ok(JsValue::from(constructor.to_js_function(ctx.realm())))
}

fn create_analyser_node_constructor(ctx: &mut Context) -> JsResult<JsValue> {
    let constructor = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let _context = args.get(0);
        create_analyser_node_instance(ctx)
    });
    Ok(JsValue::from(constructor.to_js_function(ctx.realm())))
}

fn create_biquad_filter_node_constructor(ctx: &mut Context) -> JsResult<JsValue> {
    let constructor = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let _context = args.get(0);
        create_biquad_filter_node_instance(ctx)
    });
    Ok(JsValue::from(constructor.to_js_function(ctx.realm())))
}

fn create_delay_node_constructor(ctx: &mut Context) -> JsResult<JsValue> {
    let constructor = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let _context = args.get(0);
        let max_delay = args.get(1).and_then(|v| v.to_number(ctx).ok()).unwrap_or(1.0);
        create_delay_node_instance(ctx, max_delay)
    });
    Ok(JsValue::from(constructor.to_js_function(ctx.realm())))
}

fn create_dynamics_compressor_node_constructor(ctx: &mut Context) -> JsResult<JsValue> {
    let constructor = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let _context = args.get(0);
        create_dynamics_compressor_node_instance(ctx)
    });
    Ok(JsValue::from(constructor.to_js_function(ctx.realm())))
}

fn create_panner_node_constructor(ctx: &mut Context) -> JsResult<JsValue> {
    let constructor = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let _context = args.get(0);
        create_panner_node_instance(ctx)
    });
    Ok(JsValue::from(constructor.to_js_function(ctx.realm())))
}

fn create_convolver_node_constructor(ctx: &mut Context) -> JsResult<JsValue> {
    let constructor = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let _context = args.get(0);
        create_convolver_node_instance(ctx)
    });
    Ok(JsValue::from(constructor.to_js_function(ctx.realm())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_context_registration() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"typeof AudioContext !== 'undefined'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_audio_context_creation() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var ctx = AudioContext();
              ctx.sampleRate === 44100 && ctx.state === 'running'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_audio_context_destination() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var ctx = AudioContext();
              ctx.destination !== null && ctx.destination.maxChannelCount === 2"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_create_gain_node() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var ctx = AudioContext();
              var gain = ctx.createGain();
              gain.gain.value === 1.0"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_create_oscillator() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var ctx = AudioContext();
              var osc = ctx.createOscillator();
              osc.type === 'sine' && osc.frequency.value === 440"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_create_analyser() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var ctx = AudioContext();
              var analyser = ctx.createAnalyser();
              analyser.fftSize === 2048 && analyser.frequencyBinCount === 1024"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_audio_node_connect() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var ctx = AudioContext();
              var osc = ctx.createOscillator();
              var gain = ctx.createGain();
              var connected = osc.connect(gain);
              connected === gain"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_audio_param_automation() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var ctx = AudioContext();
              var gain = ctx.createGain();
              gain.gain.setValueAtTime(0.5, 0);
              gain.gain.value === 0.5"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_create_buffer() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var ctx = AudioContext();
              var buffer = ctx.createBuffer(2, 44100, 44100);
              buffer.numberOfChannels === 2 && buffer.length === 44100"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_offline_audio_context() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var offline = OfflineAudioContext(2, 44100, 44100);
              offline.length === 44100 && offline.state === 'suspended'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_audio_context_suspend_resume() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        // Test that suspend/resume methods exist and return Promises
        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var ctx = AudioContext();
              var suspendResult = ctx.suspend();
              var resumeResult = ctx.resume();
              typeof suspendResult.then === 'function' && typeof resumeResult.then === 'function'"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_biquad_filter() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var ctx = AudioContext();
              var filter = ctx.createBiquadFilter();
              filter.type === 'lowpass' && filter.frequency.value === 350"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_dynamics_compressor() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();

        let result = ctx.eval(boa_engine::Source::from_bytes(
            b"var ctx = AudioContext();
              var compressor = ctx.createDynamicsCompressor();
              compressor.threshold.value === -24 && compressor.ratio.value === 12"
        )).unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}

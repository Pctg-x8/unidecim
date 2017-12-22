extern crate libc;
#[macro_use] extern crate lazy_static;

use std::ops::Range;
use libc::*;
mod audioplugininterface; use audioplugininterface::*;

pub struct DecimatingProcessor { locking: Vec<f32>, div: u32, div_pending: u32, phase: u32 }
impl DecimatingProcessor
{
    pub fn new() -> Self { DecimatingProcessor { locking: Vec::new(), div: 1, div_pending: 1, phase: 0 } }
}
impl DecimatingProcessor
{
    pub fn create(state: &mut UnityAudioEffectState) -> CallbackResult
    {
        state.write_effect_data(Box::into_raw(Box::new(Self::new())));
        /*let params = [Parameter("Division Rate", 1 .. 44100).description("Division Rate").into()];
        let adef = UnityAudioEffectDefinition
        {
            structsize: std::mem::size_of::<UnityAudioEffectDefinition>() as _,
            paramdefs: params.as_ptr(), numparameters: params.len() as _,

        };*/
        CallbackResult::Ok
    }
    pub fn release(state: &mut UnityAudioEffectState) -> CallbackResult
    {
        unsafe { drop(Box::from_raw(state.effect_data_mut::<Self>() as *mut _)) };
        state.write_effect_data::<Self>(std::ptr::null_mut());
        CallbackResult::Ok
    }
    pub fn process(state: &mut UnityAudioEffectState, inbuf: &[c_float], outbuf: &mut [c_float], frames: usize, channels: usize) -> CallbackResult
    {
        let this = state.effect_data_mut::<Self>();
        if this.locking.len() < channels { this.locking.reserve(channels); unsafe { this.locking.set_len(channels); } }
        for f in 0 .. frames
        {
            if this.phase == 0
            {
                for c in 0 .. channels { this.locking[c] = inbuf[c]; }
                this.div = this.div_pending;
            }
            for c in 0 .. channels { outbuf[f * channels + c] = this.locking[c]; }
            this.phase += 1;
            if this.phase >= this.div { this.phase = 0; }
        }
        CallbackResult::Ok
    }
    pub fn set_float_parameter(state: &mut UnityAudioEffectState, index: c_int, value: c_float) -> CallbackResult
    {
        let this = state.effect_data_mut::<Self>();
        if index == 0 { this.div_pending = value as _; CallbackResult::Ok }
        else { CallbackResult::Unsupported }
    }
    pub fn get_float_parameter(state: &mut UnityAudioEffectState, index: c_int, value: &mut c_float) -> CallbackResult
    {
        let this = state.effect_data_mut::<Self>();
        if index == 0 { *value = this.div_pending as _; CallbackResult::Ok }
        else { CallbackResult::Unsupported }
    }
}
pub struct Parameter(UnityAudioParameterDefinition);
impl Parameter
{
    pub fn new(name: &'static str, range: Range<c_float>) -> Self
    {
        let mut def = UnityAudioParameterDefinition
        {
            min: range.start, max: range.end, defaultval: range.start,
            displayscale: 1.0, displayexponent: 1.0, .. unsafe { std::mem::zeroed() }
        };
        def.name[..name.as_bytes().len()].copy_from_slice(unsafe { std::mem::transmute(name.as_bytes()) });
        Parameter(def)
    }
    pub fn description(mut self, desc: &'static str) -> Self
    {
        self.0.description = desc.as_ptr() as *const _; self
    }
}
impl Into<UnityAudioParameterDefinition> for Parameter { fn into(self) -> UnityAudioParameterDefinition { self.0 } }

#[repr(C)] pub enum CallbackResult { Ok = UNITY_AUDIODSP_OK as _, Unsupported = UNITY_AUDIODSP_ERR_UNSUPPORTED as _ }

extern "C" fn create_decimating_processor(state: *mut UnityAudioEffectState) -> UnityAudioDSPResult { DecimatingProcessor::create(unsafe { &mut *state }) as _ }
extern "C" fn release_decimating_processor(state: *mut UnityAudioEffectState) -> UnityAudioDSPResult { DecimatingProcessor::release(unsafe { &mut *state }) as _ }
extern "C" fn process_decimating_processor(state: *mut UnityAudioEffectState, inbuf: *mut c_float, outbuf: *mut c_float, length: c_uint, inchannels: c_int, outchannels: c_int) -> UnityAudioDSPResult
{
    if inchannels != outchannels { return CallbackResult::Unsupported as _; }
    if inchannels < 0 { return CallbackResult::Unsupported as _; }
    let inbuf = unsafe { std::slice::from_raw_parts(inbuf, (length * inchannels as u32) as usize) };
    let outbuf = unsafe { std::slice::from_raw_parts_mut(outbuf, (length * outchannels as u32) as usize) };
    DecimatingProcessor::process(unsafe { &mut *state }, inbuf, outbuf, length as _, inchannels as _) as _
}
extern "C" fn set_decimating_processor_float_parameter(state: *mut UnityAudioEffectState, index: c_int, value: c_float) -> UnityAudioDSPResult
{
    DecimatingProcessor::set_float_parameter(unsafe { &mut *state }, index, value) as _
}
extern "C" fn get_decimating_processor_float_parameter(state: *mut UnityAudioEffectState, index: c_int, value: *mut c_float, valuestr: *mut c_char) -> UnityAudioDSPResult
{
    let r = DecimatingProcessor::get_float_parameter(unsafe { &mut *state }, index, unsafe { &mut *value }) as _;
    if !valuestr.is_null() { unsafe { *valuestr = 0; } } r
}

// entry point, returns number of effects
#[no_mangle]
pub extern "C" fn UnityGetAudioEffectDefinitions(defptr: *mut *const std::sync::atomic::AtomicPtr<UnityAudioEffectDefinition>) -> c_int
{
    lazy_static!(
        static ref PARAMS: [UnityAudioParameterDefinition; 1] = [
            Parameter::new("Division Rate", 1.0 .. 44100.0).description("Division Rate").into()
        ];
        static ref ADEF: UnityAudioEffectDefinition = UnityAudioEffectDefinition
        {
            create: Some(create_decimating_processor), release: Some(release_decimating_processor),
            process: Some(process_decimating_processor), setfloatparameter: Some(set_decimating_processor_float_parameter),
            getfloatparameter: Some(get_decimating_processor_float_parameter),
            numparameters: PARAMS.len() as _, paramdefs: PARAMS.as_ptr(),
            .. Default::default()
        };
        static ref DEFPTRS: [std::sync::atomic::AtomicPtr<UnityAudioEffectDefinition>; 1] = [std::sync::atomic::AtomicPtr::new(&ADEF as *const _ as *mut _)];
    );
    unsafe { *defptr = DEFPTRS.as_ptr(); } 1
}

#![allow(non_snake_case)]
#![feature(box_syntax, const_fn)]

extern crate libc;
#[macro_use] extern crate lazy_static;

use libc::*;
mod audioplugininterface; use audioplugininterface::*;
mod audiointerface; use audiointerface::*;

pub struct DecimatingProcessor { locking: Vec<f32>, div: f32, div_pending: f32, phase: f32, overlap: bool }
impl DecimatingProcessor
{
    pub fn new() -> Self { DecimatingProcessor { locking: Vec::new(), div: 1.0, div_pending: 1.0, phase: 0.0, overlap: true } }
}
impl DecimatingProcessor
{
    pub fn parameters() -> &'static [UnityAudioParameterDefinition]
    {
        lazy_static!(static ref V: [UnityAudioParameterDefinition; 1] = [
            Parameter::new("Division Rate", 1.0 .. 128.0).description("Latching sample length").into()
        ];);
        &*V
    }
    fn definition_constr() -> UnityAudioEffectDefinition
    {
        let mut uad = UnityAudioEffectDefinition
        {
            create: Some(create_cb::<Self>), release: Some(release_cb::<Self>), process: Some(process_cb::<Self>),
            setfloatparameter: Some(setfloatparameter_cb::<Self>), getfloatparameter: Some(getfloatparameter_cb::<Self>),
            numparameters: Self::parameters().len() as _, paramdefs: Self::parameters().as_ptr(),
            .. Default::default()
        };
        uad.name[.."Decimator".as_bytes().len()].copy_from_slice(unsafe { std::mem::transmute("Decimator".as_bytes()) });
        uad
    }
    pub fn definition() -> &'static UnityAudioEffectDefinition
    {
        lazy_static!(static ref V: UnityAudioEffectDefinition = DecimatingProcessor::definition_constr(););
        &*V
    }
}
impl PluginLifecycle for DecimatingProcessor
{
    fn create(state: &mut UnityAudioEffectState) -> CallbackResult
    {
        state.write_effect_data(Box::into_raw(box Self::new()));
        CallbackResult::Ok
    }
    fn release(state: &mut UnityAudioEffectState) -> CallbackResult
    {
        unsafe { drop(Box::from_raw(state.effect_data_mut::<Self>() as *mut _)) };
        state.write_effect_data::<Self>(std::ptr::null_mut());
        CallbackResult::Ok
    }
}
impl PluginProcess for DecimatingProcessor
{
    fn process(state: &mut UnityAudioEffectState, inbuf: &[c_float], outbuf: &mut [c_float], frames: usize, channels: usize) -> CallbackResult
    {
        let this = state.effect_data_mut::<Self>();
        if this.div == 1.0 && this.div_pending == 1.0 { outbuf.copy_from_slice(inbuf); return CallbackResult::Ok; }
        if this.locking.len() < channels { this.locking.reserve(channels); unsafe { this.locking.set_len(channels); } }
        for f in 0 .. frames
        {
            if this.overlap
            {
                this.locking.copy_from_slice(&inbuf[f * channels .. (f + 1) * channels]);
                //for c in 0 .. channels { this.locking[c] = inbuf[f * channels + c]; }
                this.div = this.div_pending; this.overlap = false;
            }
            for c in 0 .. channels { outbuf[f * channels + c] = this.locking[c]; }
            this.phase += 1.0;
            if this.phase >= this.div { this.phase -= this.div; this.overlap = true; }
        }
        CallbackResult::Ok
    }
}
impl PluginParameterHandlers for DecimatingProcessor
{
    fn set_float_parameter(state: &mut UnityAudioEffectState, index: usize, value: c_float) -> CallbackResult
    {
        let this = state.effect_data_mut::<Self>();
        if index == 0 { this.div_pending = value as _; CallbackResult::Ok }
        else { CallbackResult::Unsupported }
    }
    fn get_float_parameter(state: &mut UnityAudioEffectState, index: usize, value: &mut c_float, value_str: *mut c_char) -> CallbackResult
    {
        unsafe { if let Some(c) = value_str.as_mut() { *c = 0 as _; } }
        let this = state.effect_data_mut::<Self>();
        if index == 0 { *value = this.div_pending as _; CallbackResult::Ok }
        else { CallbackResult::Unsupported }
    }
}

// entry point, returns number of effects
#[no_mangle]
pub extern "C" fn UnityGetAudioEffectDefinitions(defptr: *mut *const std::sync::atomic::AtomicPtr<UnityAudioEffectDefinition>) -> c_int
{
    lazy_static!(static ref DEFPTRS: [std::sync::atomic::AtomicPtr<UnityAudioEffectDefinition>; 1] = [
        std::sync::atomic::AtomicPtr::new(DecimatingProcessor::definition() as *const _ as *mut _)
    ];);
    unsafe { *defptr = DEFPTRS.as_ptr(); } 1
}

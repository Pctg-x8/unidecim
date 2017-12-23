//! Audio Interface definition wrapper

use audioplugininterface::*;
use libc::{c_char, c_float, c_int, c_uint};
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::ops::Range;
use std::mem::{transmute, zeroed};

#[repr(C)]
pub enum CallbackResult { Ok = UNITY_AUDIODSP_OK as _, Unsupported = UNITY_AUDIODSP_ERR_UNSUPPORTED as _ }
pub trait PluginLifecycle
{
    /// Called on creating a plugin
    fn create(state: &mut UnityAudioEffectState) -> CallbackResult;
    /// Called when a plugin is releasing
    fn release(state: &mut UnityAudioEffectState) -> CallbackResult;
}
pub trait PluginProcess
{
    /// Called when the audio engine requests processing audio
    fn process(state: &mut UnityAudioEffectState, inbuf: &[c_float], outbuf: &mut [c_float], frames: usize, channels: usize) -> CallbackResult;
}
pub trait PluginParameterHandlers
{
    /// Called when the game engine requests changing a parameter in this plugin
    fn set_float_parameter(state: &mut UnityAudioEffectState, index: usize, value: c_float) -> CallbackResult;
    /// Called when the game engine requests a parameter value in this plugin
    fn get_float_parameter(state: &mut UnityAudioEffectState, index: usize, value: &mut c_float, value_str: *mut c_char) -> CallbackResult;
}

pub extern "C" fn create_cb<T: PluginLifecycle>(state: *mut UnityAudioEffectState) -> UnityAudioDSPResult { T::create(unsafe { &mut *state }) as _ }
pub extern "C" fn release_cb<T: PluginLifecycle>(state: *mut UnityAudioEffectState) -> UnityAudioDSPResult { T::release(unsafe { &mut *state }) as _ }
pub extern "C" fn process_cb<T: PluginProcess>(state: *mut UnityAudioEffectState, inbuf: *mut c_float, outbuf: *mut c_float, length: c_uint, inchannels: c_int, outchannels: c_int) -> UnityAudioDSPResult
{
    if inchannels != outchannels { return CallbackResult::Unsupported as _; }
    if inchannels < 0 { return CallbackResult::Unsupported as _; }
    let inbuf = unsafe { from_raw_parts(inbuf, (length * inchannels as u32) as usize) };
    let outbuf = unsafe { from_raw_parts_mut(outbuf, (length * outchannels as u32) as usize) };
    T::process(unsafe { &mut *state }, inbuf, outbuf, length as _, inchannels as _) as _
}
pub extern "C" fn setfloatparameter_cb<T: PluginParameterHandlers>(state: *mut UnityAudioEffectState, index: c_int, value: c_float) -> UnityAudioDSPResult
{
    if index < 0 { return CallbackResult::Unsupported as _; }
    T::set_float_parameter(unsafe { &mut *state }, index as usize, value) as _
}
pub extern "C" fn getfloatparameter_cb<T: PluginParameterHandlers>(state: *mut UnityAudioEffectState, index: c_int, value: *mut c_float, valuestr: *mut c_char)
    -> UnityAudioDSPResult
{
    if index < 0 { return CallbackResult::Unsupported as _; }
    T::get_float_parameter(unsafe { &mut *state }, index as usize, unsafe { &mut *value }, valuestr) as _
}

pub struct Parameter(UnityAudioParameterDefinition);
impl Parameter
{
    pub fn new(name: &'static str, range: Range<c_float>) -> Self
    {
        let mut def = UnityAudioParameterDefinition
        {
            min: range.start, max: range.end, defaultval: range.start,
            displayscale: 1.0, displayexponent: 1.0, .. unsafe { zeroed() }
        };
        def.name[..name.as_bytes().len()].copy_from_slice(unsafe { transmute(name.as_bytes()) });
        Parameter(def)
    }
    pub fn description(mut self, desc: &'static str) -> Self
    {
        self.0.description = desc.as_ptr() as *const _; self
    }
}
impl Into<UnityAudioParameterDefinition> for Parameter { fn into(self) -> UnityAudioParameterDefinition { self.0 } }

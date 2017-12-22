use libc::*;
use std::mem::{size_of, transmute}; use std::ptr::null_mut;

pub const UNITY_AUDIO_PLUGIN_API_VERSION: usize = 0x010401;

// from macro
pub type UnityAudioDSPResult = c_int;
pub const UNITY_AUDIODSP_OK: UnityAudioDSPResult = 0;
pub const UNITY_AUDIODSP_ERR_UNSUPPORTED: UnityAudioDSPResult = 1;

pub type UnityAudioEffect_CreateCallback = extern "C" fn(state: *mut UnityAudioEffectState) -> UnityAudioDSPResult;
pub type UnityAudioEffect_ReleaseCallback = extern "C" fn(state: *mut UnityAudioEffectState) -> UnityAudioDSPResult;
pub type UnityAudioEffect_ResetCallback = extern "C" fn(state: *mut UnityAudioEffectState) -> UnityAudioDSPResult;
pub type UnityAudioEffect_ProcessCallback = extern "C" fn(state: *mut UnityAudioEffectState, inbuffer: *mut c_float, outbuffer: *mut c_float, length: c_uint, inchannels: c_int, outchannels: c_int) -> UnityAudioDSPResult;
pub type UnityAudioEffect_SetPositionCallback = extern "C" fn(state: *mut UnityAudioEffectState, pos: c_uint) -> UnityAudioDSPResult;
pub type UnityAudioEffect_SetFloatParameterCallback = extern "C" fn(state: *mut UnityAudioEffectState, index: c_int, value: c_float) -> UnityAudioDSPResult;
pub type UnityAudioEffect_GetFloatParameterCallback = extern "C" fn(state: *mut UnityAudioEffectState, index: c_int, value: *mut c_float, valuestr: *mut c_char) -> UnityAudioDSPResult;
pub type UnityAudioEffect_GetFloatBufferCallback = extern "C" fn(state: *mut UnityAudioEffectState, name: *const c_char, buffer: *mut c_float, numsamples: c_int) -> UnityAudioDSPResult;

#[repr(C)] pub enum UnityAudioEffectDefinitionFlags
{
    /// Does this effect need a side chain buffer and can it be targeted by a Send?
    IsSideChainTarget = 1 << 0,
    /// Should this plugin be inserted at sources and take over panning?
    IsSpatializer = 1 << 1,
    /// Should this plugin be used for ambisonic decoding? Added in Unity 2017.1, with UNITY_AUDIO_PLUGIN_API_VERSION 0x010400
    IsAmbisonicDecoder = 1 << 2,
    /// Spatializers Only: Does this spatializer apply distance-based attenuation? Added in Unity 2017.1, with UNITY_AUDIO_PLUGIN_API_VERSION 0x010400
    AppliesDistanceAttenuation = 1 << 3
}
#[repr(C)] pub enum UnityAudioEffectStateFlags
{
    /// Set when engine is in play mode. Also true while paused
    IsPlaying = 1 << 0,
    /// Set when engine is paused mode
    IsPaused = 1 << 1,
    /// Set when effect is being muted (only available in the editor)
    IsMuted = 1 << 2,
    /// Does this effect need a side chain buffer and can it be targeted by a Send?
    IsSideChainTargeted = 1 << 3
}
/// This callback can be used to override the way distance attenuation is performed on AudioSources.
/// distanceIn is the distance between the source and the listener and attenuationOut is the output volume.
/// attenuationIn is the volume-curve based attenuation that would have been applied by Unity if this callback were not set.
/// A typical attenuation curve may look like this: *attenuationOut = 1.0f / max(1.0f, distanceIn);
/// The callback may also be used to apply a secondary gain on top of the one through attenuationIn by Unity's AudioSource curve.
pub type UnityAudioEffect_DistanceAttenuationCallback = extern "C" fn(state: *mut UnityAudioEffectState, distanceIn: c_float, attenuationIn: c_float, attenuationOut: c_float) -> UnityAudioDSPResult;

#[repr(C)] pub struct UnityAudioSpatializerData
{
    /// Matrix that transforms sourcepos into local space of the listener
    pub listenermatrix: [c_float; 16],
    /// Transform matrix of audio source
    pub sourcematrix: [c_float; 16],
    /// Distance-controlled spatial blend
    pub spatialblend: c_float,
    /// Reverb zone mix leve parameter (and curve) on audio source
    pub reverbzonemix: c_float,
    /// Spread parameter of the audio source (0 .. 360 degrees)
    pub spread: c_float,
    /// Stereo panning parameter of the audio source (-1 = fully left, 1 = fully right)
    pub stereopan: c_float,
    /// The spatializer plugin may override the distance attenuation in order to influence the voice prioritization
    /// (leave this callback as `None` to use the built-in audio source attenuation curve)
    pub distanceattenuationcallback: Option<UnityAudioEffect_DistanceAttenuationCallback>,
    /// Min distance of the audio source. This value may be helpful to determine when to apply near-field effects.
    /// Added in Unity 2018.1, with UNITY_AUDIO_PLUGIN_API_VERSION 0x010401
    pub minDistance: c_float,
    /// Max distance of the audio source. Added in Unity 2018.1, with UNITY_AUDIO_PLUGIN_API_VERSION 0x010401
    pub maxDistance: c_float
}
#[repr(C)] pub struct UnityAudioAmbisonicData
{
    /// Matrix that transforms sourcepos into local space of the listener
    pub listenermatrix: [c_float; 16],
    /// Transform matrix of audio source
    pub sourcematrix: [c_float; 16],
    /// Distance-controlled spatial blend
    pub spatialblend: c_float,
    /// Reverb zone mix leve parameter (and curve) on audio source
    pub reverbzonemix: c_float,
    /// Spread parameter of the audio source (0 .. 360 degrees)
    pub spread: c_float,
    /// Stereo panning parameter of the audio source (-1 = fully left, 1 = fully right)
    pub stereopan: c_float,
    /// The spatializer plugin may override the distance attenuation in order to influence the voice prioritization
    /// (leave this callback as `None` to use the built-in audio source attenuation curve)
    pub distanceattenuationcallback: Option<UnityAudioEffect_DistanceAttenuationCallback>,
    /// This tells ambisonic decoders how many output channels will actually be used
    pub ambisonicOutChannels: c_int,
    /// Volume/mute of the audio source. If the the source ismuted, volume is set to 0.0; otherwise, it is set to the audio source's volume.
    /// Volume is applied after the ambisonic decoder, so this is just informational. Added in Unity 2018.1, with UNITY_AUDIO_PLUGIN_API_VERSION 0x010401
    pub volume: c_float
}

// This padding was historically due to PS3 SPU DMA requirement. We aren't removing it now because plugins may rely on this struct being at least this size.
#[repr(C)] pub struct UnityAudioEffectState([c_uchar; 80]);
/// Union accessor impls
impl UnityAudioEffectState
{
    /// Size of this struct
    pub fn structsize(&self) -> u32 { unsafe { transmute::<_, &[u32]>(&self.0[..])[0] } }
    /// System sample rate
    pub fn samplerate(&self) -> u32 { unsafe { transmute::<_, &[u32]>(&self.0[..])[1] } }
    /// Pointer to a sample counter marking the start of the current block being processed
    pub fn currdsptick(&self) -> u64 { unsafe { transmute::<_, &[u64]>(&self.0[..])[1] } }
    /// Used for determining when DSPs are bypassed and so sidechain info becomes invalid
    pub fn prevdsptick(&self) -> u64 { unsafe { transmute::<_, &[u64]>(&self.0[..])[2] } }
    /// Side-chain buffers to read from
    pub fn sidechainbuffer(&self) -> *const c_float { unsafe { transmute::<_, &[*const c_float]>(&self.0[size_of::<u64>() * 3..])[0] } }
    /// Internal data for the effect
    pub fn effectdata(&self) -> *const c_void { unsafe { transmute::<_, &[*const c_void]>(&self.0[size_of::<u64>() * 3 + size_of::<*const c_float>()..])[0] } }
    /// Internal data for the effect
    pub fn effectdata_mut(&mut self) -> *mut c_void { unsafe { transmute::<_, &[*mut c_void]>(&self.0[size_of::<u64>() * 3 +size_of::<*const c_float>()..])[0] } }
    /// Various flags through which information can be queried from the host
    pub fn flags(&self) -> u32 { unsafe { transmute::<_, &[u32]>(&self.0[size_of::<u64>() * 3 + size_of::<*const c_void>() * 2..])[0] } }

    /// Data for spatializers
    pub fn spatializerdata(&self) -> *const UnityAudioSpatializerData
    {
        unsafe { transmute::<_, &[*const _]>(&self.0[size_of::<u64>() * 3 + size_of::<*const c_void>() * 3 + size_of::<u32>()..])[0] }
    }
    /// Number of frames bein processed per process callback. Use this to allocate temporary buffers before processing starts.
    pub fn dspbuffersize(&self) -> u32
    {
        unsafe { transmute::<_, &[u32]>(&self.0[size_of::<u64>() * 3 + size_of::<*const c_void>() * 4 + size_of::<u32>()..])[0] }
    }
    /// Version of plugin API used by host
    pub fn hostapiversion(&self) -> u32
    {
        unsafe { transmute::<_, &[u32]>(&self.0[size_of::<u64>() * 3 + size_of::<*const c_void>() * 4 + size_of::<u32>() * 2..])[0] }
    }
    /// Data for ambisonic plugins. Added in Unity 2017.1, with UNITY_AUDIO_PLUGIN_API_VERSION 0x010400
    pub fn ambisonicdata(&self) -> *const UnityAudioAmbisonicData
    {
        unsafe { transmute::<_, &[*const _]>(&self.0[size_of::<u64>() * 3 + size_of::<*const c_void>() * 4 + size_of::<u32>() * 3..])[0] }
    }

    pub fn effect_data<T>(&self) -> &T
    {
        assert!(!self.effectdata().is_null());
        unsafe { &*(self.effectdata() as *const T) }
    }
    pub fn effect_data_mut<T>(&mut self) -> &mut T
    {
        assert!(!self.effectdata().is_null());
        unsafe { &mut *(self.effectdata_mut() as *mut T) }
    }
    pub fn write_effect_data<T>(&mut self, ptr: *mut T)
    {
        unsafe { transmute::<_, &mut [*mut c_void]>(&mut self.0[size_of::<u64>() * 3 + size_of::<*const c_float>()..])[0] = ptr as _; }
    }
}

#[repr(C)] pub struct UnityAudioParameterDefinition
{
    /// Display name on the GUI
    pub name: [c_char; 16],
    /// Scientific unit of parameter to eb appended after the value in textboxes
    pub unit: [c_char; 16],
    /// Description of parameter (displayed in tool tips, automatically generated documentation, etc.)
    pub description: *const c_char,
    /// Minimum value of the parameter
    pub min: c_float,
    /// Maximum value of the parameter
    pub max: c_float,
    /// Default and initial value of the parameter
    pub defaultval: c_float,
    /// Scale factor used only for the display of parameters (i.e. 100 for a percentage value ranging from 0 to 1)
    pub displayscale: c_float,
    /// Exponent for mapping parameters to sliders
    pub displayexponent: c_float
}
#[repr(C)] pub struct UnityAudioEffectDefinition
{
    /// Size of this struct
    pub structsize: u32,
    /// Size of paramdesc fields
    pub paramstructsize: u32,
    /// Plugin API version
    pub apiversion: u32,
    /// Version of this plugin
    pub pluginversion: u32,
    /// Number of channels. Effects should set this to 0 and process any number of input/output channels they get in the process callback.
    /// Generator elements should specify a >0 value here.
    pub channels: u32,
    /// The number of parameters exposed by this plugin
    pub numparameters: u32,
    /// Various capabilities an requirements of the plugin
    pub flags: u64,
    /// Name used for registration of the effect. This name will also be displayed in the GUI
    pub name: [c_char; 32],
    /// The create callback is called when DSP unit is created and can be null.
    pub create: Option<UnityAudioEffect_CreateCallback>,
    /// The release callback is called just before the plugin is freed and should free any data associated with this specific instance of the plugin.
    /// No further callbacks related to the instance will happen after this function has been called.
    pub release: Option<UnityAudioEffect_ReleaseCallback>,
    /// The reset callback is called by the user to bring back the plugin instance into its initial state. Use to avoid clicks or artifacts.
    pub reset: Option<UnityAudioEffect_ResetCallback>,
    /// The processing callback is repeatedly called with a block of input audio to read from and an output block to write to.
    pub process: Option<UnityAudioEffect_ProcessCallback>,
    /// The position callback can be used for implementing seek operations.
    pub setposition: Option<UnityAudioEffect_SetPositionCallback>,
    /// A pointer to the definitions of the parameters exposed by this plugin. This data pointed to must remain valid for the whole lifetime of the dynamic library
    /// (ideally it's static)
    pub paramdefs: *const UnityAudioParameterDefinition,
    /// This is called whenever one of the exposed parameters is changed
    pub setfloatparameter: Option<UnityAudioEffect_SetFloatParameterCallback>,
    /// This is called to query parameter values
    pub getfloatparameter: Option<UnityAudioEffect_GetFloatParameterCallback>,
    /// Get N samples of named buffer. Used for displaying analysis data from the runtime.
    pub getfloatbuffer: Option<UnityAudioEffect_GetFloatBufferCallback>
}
unsafe impl Sync for UnityAudioParameterDefinition {}
unsafe impl Sync for UnityAudioEffectDefinition {}
impl Default for UnityAudioEffectDefinition
{
    fn default() -> Self
    {
        UnityAudioEffectDefinition
        {
            structsize: size_of::<UnityAudioEffectDefinition>() as _, paramstructsize: size_of::<UnityAudioParameterDefinition>() as _,
            apiversion: UNITY_AUDIO_PLUGIN_API_VERSION as _, pluginversion: 0x010000,
            channels: 2, numparameters: 0, flags: 0, name: [0; 32],
            create: None, release: None, reset: None, process: None, setposition: None, paramdefs: null_mut(),
            setfloatparameter: None, getfloatparameter: None, getfloatbuffer: None
        }
    }
}

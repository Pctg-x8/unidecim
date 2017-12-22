extern crate libc;

use libc::*;
mod audioplugininterface; use audioplugininterface::*;

#[no_mangle]
pub extern "C" fn UnityGetAudioEffectDefinitions(defptr: *mut *mut *mut UnityAudioEffectDefinition) -> c_int
{

}

#[repr(C)]
pub struct LR<T>(T, T);
impl<T: Clone> Clone for LR<T> { fn clone(&self) -> Self { LR(self.0.clone(), self.1.clone()) } }
impl<T: Default> Default for LR<T> { fn default() -> Self { LR(T::default(), T::default()) } }
pub struct DecimatingProcessor<T> { locking: T, div: u32, phase: u32 }
impl<T: Default> DecimatingProcessor<T>
{
    pub fn new(div: u32) -> Self { DecimatingProcessor { locking: T::default(), div, phase: 0 } }
}

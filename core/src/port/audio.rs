use crate::port::{AtomicPortType, PortType};
use std::cell::Cell;
use std::ffi::c_void;
use std::ptr::NonNull;
use urid::UriBound;

/// A port connected to an array of float audio samples. Using this port **requires** the `inPlaceBroken` feature.
///
/// Ports of this type are connected to a buffer of float audio samples, represented as a slice.
///
/// Audio samples are normalized between -1.0 and 1.0, though there is no requirement for samples to be strictly within this range.
///
/// See the [LV2 reference](https://lv2plug.in/ns/lv2core#AudioPort) for more information.
///
/// # Example
///
/// This very simple amplifier plugin multiplies the input sample by 2 and outputs the result.
///
/// ```
/// # use lv2_core::prelude::*;
/// # use urid::*;
/// # #[uri("http://lv2plug.in/plugins.rs/simple_amp")]
/// # struct SimpleAmp;
/// #[derive(PortCollection)]
/// struct SimpleAmpPorts {
///     input: InputPort<Audio>,
///     output: OutputPort<Audio>,
/// }
///
/// impl Plugin for SimpleAmp {
///     type Ports = SimpleAmpPorts;
/// # type InitFeatures = ();
/// # type AudioFeatures = ();
/// # fn new(plugin_info: &PluginInfo,features: &mut Self::InitFeatures) -> Option<Self> {
/// #         unimplemented!()
/// # }
///     // some implementation details elided…
///
///     fn run(&mut self, ports: &mut SimpleAmpPorts, _: &mut (), _: u32) {
///         // Input and Output dereference to `&[f32]` and `&mut [f32]`, respectively.
///         let input = ports.input.iter();
///         let output = ports.output.iter_mut();
///
///         for (input_sample, output_sample) in input.zip(output) {
///             *output_sample = *input_sample * 2.0;
///         }
///     }
/// }
///
///
/// ```
///
/// # Safety
///
/// Using this port type requires the `inPlaceBroken` LV2 feature in your plugin. Because this port
/// type uses shared (`&[f32]`) and exclusive (`&mut [f32]`) references to its data, LV2 hosts
/// MUST NOT use the same buffer for both the input and the output.
/// However, do note that some hosts (Ardour, Zrythm, etc.) do not support `inPlaceBroken` plugins.
///
/// Use [`InPlaceAudio`] instead if you do not want to enforce this restriction on hosts,
/// and do not need references pointing into the buffer's contents.
pub struct Audio;

unsafe impl UriBound for Audio {
    const URI: &'static [u8] = ::lv2_sys::LV2_CORE__AudioPort;
}

pub mod handles {
    use crate::port::PortTypeHandle;
    use std::cell::Cell;

    pub struct AudioInputType;

    impl<'a> PortTypeHandle<'a> for AudioInputType {
        type Handle = &'a [f32];
    }

    pub struct AudioOutputType;

    impl<'a> PortTypeHandle<'a> for AudioOutputType {
        type Handle = &'a mut [f32];
    }

    pub struct AudioInputOutputType;

    impl<'a> PortTypeHandle<'a> for AudioInputOutputType {
        type Handle = (&'a [Cell<f32>], &'a [Cell<f32>]);
    }
}

impl PortType for Audio {
    type InputPortType = handles::AudioInputType;
    type OutputPortType = handles::AudioOutputType;

    #[inline]
    unsafe fn input_from_raw<'a>(pointer: NonNull<c_void>, sample_count: u32) -> &'a [f32] {
        core::slice::from_raw_parts(pointer.as_ptr() as *const f32, sample_count as usize)
    }

    #[inline]
    unsafe fn output_from_raw<'a>(pointer: NonNull<c_void>, sample_count: u32) -> &'a mut [f32] {
        core::slice::from_raw_parts_mut(pointer.as_ptr() as *mut f32, sample_count as usize)
    }
}

impl AtomicPortType for Audio {
    type InputOutputPortType = handles::AudioInputOutputType;

    #[inline]
    unsafe fn input_output_from_raw<'a>(
        input: NonNull<c_void>,
        output: NonNull<c_void>,
        sample_count: u32,
    ) -> (&'a [Cell<f32>], &'a [Cell<f32>]) {
        let input =
            core::slice::from_raw_parts_mut(input.as_ptr() as *mut f32, sample_count as usize);
        let output =
            core::slice::from_raw_parts_mut(output.as_ptr() as *mut f32, sample_count as usize);

        (
            Cell::from_mut(input).as_slice_of_cells(),
            Cell::from_mut(output).as_slice_of_cells(),
        )
    }
}

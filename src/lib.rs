use std::{f32::consts::PI, sync::Arc};

use nih_plug::prelude::*;

struct Effect {
    params: Arc<EffectParams>,
    sample_rate: f32,
}

const GAIN_MAX: f32 = 0.75;

#[derive(Params)]
struct EffectParams {
    #[id = "effect_gain"]
    pub gain: FloatParam,
}

impl Default for Effect {
    fn default() -> Self {
        Self {
            params: Arc::new(EffectParams {
                // max 0.75 is plenty, around after that point, strange behavior starts to appear,
                // like comb filtering, bitcrushing etc. In short, not worth it.
                gain: FloatParam::new("Gain", 0., FloatRange::Linear { min: 0., max: GAIN_MAX }),
            }),
            sample_rate: 44100f32,
        }
    }
}
impl Plugin for Effect {
    const NAME: &'static str = "Test Plugin 1";
    const VENDOR: &'static str = "arekkukula";
    const URL: &'static str = "...";
    const EMAIL: &'static str = "arkadiusz.kukula@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while the other one is
        // given the name 'Mono' based no the number of input and output channels.
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    // Setting this to `true` will tell the wrapper to split the buffer up into smaller blocks
    // whenever there are inter-buffer parameter changes. This way no changes to the plugin are
    // required to support sample accurate automation and the wrapper handles all of the boring
    // stuff like making sure transport and other timing information stays consistent between the
    // splits.
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;

        true
    }

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    // This plugin doesn't need any special initialization, but if you need to do anything expensive
    // then this would be the place. State is kept around when the host reconfigures the
    // plugin. If we do need special initialization, we could implement the `initialize()` and/or
    // `reset()` methods
    fn reset(&mut self) {}

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut buffer_iter = buffer.iter_samples();
        let mut first_sample = buffer_iter.next().unwrap();
        // FIXME: this is really unsafe. Potentially could crash DAW.
        let mut prev_sample: Vec<f32> = vec![*first_sample.get_mut(0).unwrap(), *first_sample.get_mut(1).unwrap()];

        for  channel_samples in buffer_iter {
            let gain = self.params.gain.smoothed.next();
            for (index, sample) in channel_samples.into_iter().enumerate() {
                // simple weighted average of current sample and previous sample
                let new_sample_value = (1f32 - gain) * *sample + gain * prev_sample[index];
                *sample = new_sample_value;
                prev_sample[index] = new_sample_value;
            }
        }
        ProcessStatus::Normal
    }

    // This can be used for cleaning up special resources like socket connections whenever the
    // plugin is deactivated. Most plugins won't need to do anything here.
    fn deactivate(&mut self) {}
}



impl Vst3Plugin for Effect {
    const VST3_CLASS_ID: [u8; 16] = *b"TestEffect1Kukul";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
    ];
}

nih_export_vst3!(Effect);

use std::{f32::consts::PI, sync::Arc};

use nih_plug::prelude::*;

struct Effect {
    params: Arc<EffectParams>,
    sample_rate: f32,
}

#[derive(Params)]
struct EffectParams {
    #[id = "makeup"]
    pub makeup: FloatParam,
    #[id = "butterworth_lp"]
    pub butterworth_freq: FloatParam,
}

impl Default for Effect {
    fn default() -> Self {
        Self {
            params: Arc::new(EffectParams {
                // max 0.75 is plenty, around after that point, strange behavior starts to appear,
                // like comb filtering, bitcrushing etc. In short, not worth it.
                makeup: FloatParam::new(
                    "Makeup",
                    util::db_to_gain(0.),
                    FloatRange::Skewed {
                        min: util::db_to_gain(-24.),
                        max: util::db_to_gain(24.),
                        factor: FloatRange::gain_skew_factor(-24., 24.),
                    },
                )
                .with_smoother(SmoothingStyle::Logarithmic(50.0))
                .with_unit(" dB")
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db()),
                butterworth_freq: FloatParam::new(
                    "Butterworth LP Freq",
                    20_000f32,
                    FloatRange::Linear {
                        min: 20.,
                        max: 20_000.,
                    },
                ),
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
        ..AudioIOLayout::const_default()
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
        let buffer_slices = buffer.as_slice();

        let (left_channel, right_channel) = buffer_slices.split_at_mut(1);
        let lp_left = left_channel.get_mut(0).unwrap();
        let lp_right = right_channel.get_mut(0).unwrap();
        lowpass_filter(
            lp_left,
            self.sample_rate,
            self.params.butterworth_freq.smoothed.next(),
        );
        lowpass_filter(
            lp_right,
            self.sample_rate,
            self.params.butterworth_freq.smoothed.next(),
        );

        makeup(lp_left, self.params.makeup.smoothed.next());
        makeup(lp_right, self.params.makeup.smoothed.next());

        ProcessStatus::Normal
    }

    // This can be used for cleaning up special resources like socket connections whenever the
    // plugin is deactivated. Most plugins won't need to do anything here.
    fn deactivate(&mut self) {}
}

fn makeup(data: &mut [f32], db: f32) {
    for sample in data {
        *sample *= db;
    }
}

pub fn lowpass_filter(data: &mut [f32], sampling_rate: f32, cutoff_frequency: f32) {
    // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
    let rc = 1.0 / (cutoff_frequency * 2.0 * PI);
    // time per sample
    let dt = 1.0 / sampling_rate;
    let alpha = dt / (rc + dt);

    data[0] *= alpha;
    for i in 1..data.len() {
        data[i] = data[i - 1] + alpha * (data[i] - data[i - 1]);
    }
}

impl Vst3Plugin for Effect {
    const VST3_CLASS_ID: [u8; 16] = *b"TestEffect1Kukul";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx];
}

nih_export_vst3!(Effect);

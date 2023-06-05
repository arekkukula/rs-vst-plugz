use nih_plug::prelude::*;
use std::sync::Arc;
mod butterworth_lp;
use butterworth_lp::lowpass_filter;
mod makeup;
use makeup::makeup;

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
                    FloatRange::Skewed {
                        min: 20.,
                        max: 20_000.,
                        factor: 0.3,
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

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
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

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let (left_channel, right_channel) = buffer.as_slice().split_at_mut(1);

        lowpass_filter(
            left_channel[0],
            self.sample_rate,
            self.params.butterworth_freq.value(),
        );

        lowpass_filter(
            right_channel[0],
            self.sample_rate,
            self.params.butterworth_freq.value(),
        );

        // makeup(lp_left, self.params.makeup.value());
        // makeup(lp_right, self.params.makeup.value());

        ProcessStatus::Normal
    }

    fn deactivate(&mut self) {}
}

impl Vst3Plugin for Effect {
    const VST3_CLASS_ID: [u8; 16] = *b"TestEffect1Kukul";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx];
}

nih_export_vst3!(Effect);

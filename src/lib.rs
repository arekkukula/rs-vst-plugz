use nih_plug::prelude::*;
use std::sync::Arc;
mod butterworth_lp;
use butterworth_lp::{lowpass_filter, lowpass_two_samples};
mod makeup;
use makeup::makeup;

struct Effect {
    params: Arc<EffectParams>,
    sample_rate: f32,

    last: [f32; 2],
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
            last: [0f32, 0f32],
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
                    25_000f32,
                    FloatRange::Skewed {
                        min: 20.,
                        max: 25_000.,
                        factor: 0.3,
                    },
                ),
            }),
            sample_rate: 0.,
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

    fn reset(&mut self) {
        self.last[0] = 0f32;
        self.last[1] = 0f32;
    }
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
        for mut channel_samples in buffer.iter_samples() {
            for (channel, sample) in channel_samples.iter_mut().enumerate() {
                *sample = lowpass_two_samples(
                    *sample,
                    self.last[channel],
                    self.sample_rate,
                    self.params.butterworth_freq.value(),
                );

                self.last[channel] = *sample;

                makeup(sample, self.params.makeup.value());
            }
        }

        ProcessStatus::Normal
    }

    fn deactivate(&mut self) {}
}

impl Vst3Plugin for Effect {
    const VST3_CLASS_ID: [u8; 16] = *b"TestEffect1Kukul";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx];
}

nih_export_vst3!(Effect);

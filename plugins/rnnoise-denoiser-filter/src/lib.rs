use nnnoiseless::DenoiseState;
use obs_wrapper::{media::audio, obs_register_module, obs_string, prelude::*, source::*};

use std::collections::VecDeque;

use dasp::{
    interpolate::linear::Linear,
    signal::{self, interpolate::Converter, Signal},
};

const RNNOISE_SAMPLE_RATE: f64 = 48000.;
const WAV_COEFFICIENT: f32 = 32767.0;
const FRAME_SIZE: usize = DenoiseState::FRAME_SIZE;

struct Output {
    buffer: VecDeque<f32>,
    last_input: (f32, f32),
    last_output: (f32, f32),
}

struct RnnoiseDenoiserFilter {
    output: Output,
    input: VecDeque<f32>,
    state: Box<DenoiseState>,
    temp: [f32; FRAME_SIZE],
    temp_out: [f32; FRAME_SIZE],
    sample_rate: f64,
    channels: usize,
}

struct TheModule {
    context: ModuleContext,
}

impl Sourceable for RnnoiseDenoiserFilter {
    fn get_id() -> ObsString {
        obs_string!("rnnoise_noise_suppression_filter")
    }
    fn get_type() -> SourceType {
        SourceType::FILTER
    }
    fn create(create: &mut CreatableSourceContext<Self>, _source: SourceContext) -> Self {
        let (sample_rate, channels) =
            create.with_audio(|audio| (audio.sample_rate(), audio.channels()));

        Self {
            input: VecDeque::with_capacity(FRAME_SIZE * 3),
            output: Output {
                buffer: VecDeque::with_capacity(FRAME_SIZE * 3),
                last_output: (0., 0.),
                last_input: (0., 0.),
            },
            temp: [0.; FRAME_SIZE],
            temp_out: [0.; FRAME_SIZE],
            state: DenoiseState::new(),
            sample_rate: sample_rate as f64,
            channels,
        }
    }
}

impl GetNameSource for RnnoiseDenoiserFilter {
    fn get_name() -> ObsString {
        obs_string!("Rnnoise Noise Suppression Filter")
    }
}

impl UpdateSource for RnnoiseDenoiserFilter {
    fn update(&mut self, _settings: &mut DataObj, context: &mut GlobalContext) {
        let sample_rate = context.with_audio(|audio| audio.sample_rate());
        self.sample_rate = sample_rate as f64;
    }
}

impl FilterAudioSource for RnnoiseDenoiserFilter {
    fn filter_audio(&mut self, audio: &mut audio::AudioDataContext) {
        let data = self;
        let state = &mut data.state;
        let input_ring_buffer = &mut data.input;
        let output_state = &mut data.output;

        let temp = &mut data.temp;
        let temp_out = &mut data.temp_out;

        if let Some(base) = audio.get_channel_as_mut_slice(0) {
            for channel in 1..data.channels {
                let buffer = audio
                    .get_channel_as_mut_slice(channel)
                    .expect("Channel count said there was a buffer here.");

                for (output, input) in base.iter_mut().zip(buffer.iter()) {
                    *output = (*output + *input) / 2.;
                }
            }

            let audio_chunks = base.chunks_mut(FRAME_SIZE);

            for buffer in audio_chunks {
                for sample in buffer.iter() {
                    input_ring_buffer.push_back(*sample);
                }

                let output_buffer = &mut output_state.buffer;
                let last_input = &mut output_state.last_input;
                let last_output = &mut output_state.last_output;

                let start_last_input = (last_input.0, last_input.1);
                let start_last_output = (last_output.0, last_output.1);

                if input_ring_buffer.len() >= FRAME_SIZE {
                    let mut converter = Converter::from_hz_to_hz(
                        signal::from_iter((0..FRAME_SIZE).map(|_| {
                            let s = input_ring_buffer
                                .pop_front()
                                .expect("There should be a sample there!");

                            last_input.0 = last_input.1;
                            last_input.1 = s;

                            s
                        })),
                        Linear::new(start_last_input.1, start_last_input.0),
                        data.sample_rate,
                        RNNOISE_SAMPLE_RATE,
                    );

                    for sample in temp.iter_mut() {
                        *sample = converter.next() * WAV_COEFFICIENT;
                    }

                    state.process_frame(temp_out, temp);

                    for sample in temp_out.iter_mut() {
                        *sample /= WAV_COEFFICIENT;
                        last_output.0 = last_output.1;
                        last_output.1 = *sample;
                    }

                    let converter = Converter::from_hz_to_hz(
                        signal::from_iter(temp_out.iter().copied()),
                        Linear::new(start_last_output.0, start_last_output.1),
                        RNNOISE_SAMPLE_RATE,
                        data.sample_rate,
                    );

                    for sample in converter.until_exhausted() {
                        output_buffer.push_back(sample);
                    }
                }

                if output_state.buffer.len() >= buffer.len() {
                    for sample in buffer.iter_mut() {
                        *sample = output_state
                            .buffer
                            .pop_front()
                            .expect("There should be a sample there!");
                    }
                }
            }

            for channel in 1..data.channels {
                let buffer = audio
                    .get_channel_as_mut_slice(channel)
                    .expect("Channel count said there was a buffer here.");

                for (output, input) in buffer.iter_mut().zip(base.iter()) {
                    *output = *input;
                }
            }
        }
    }
}

impl Module for TheModule {
    fn new(context: ModuleContext) -> Self {
        Self { context }
    }
    fn get_ctx(&self) -> &ModuleContext {
        &self.context
    }

    fn load(&mut self, load_context: &mut LoadContext) -> bool {
        let source = load_context
            .create_source_builder::<RnnoiseDenoiserFilter>()
            .enable_get_name()
            .enable_update()
            .enable_filter_audio()
            .build();

        load_context.register_source(source);

        true
    }

    fn description() -> ObsString {
        obs_string!("A filter that removes background noise from your microphone using the rnnoise neural network noise suppression model.")
    }
    fn name() -> ObsString {
        obs_string!("Rnnoise Noise Suppression Filter")
    }
    fn author() -> ObsString {
        obs_string!("Bennett Hardwick")
    }
}

obs_register_module!(TheModule);

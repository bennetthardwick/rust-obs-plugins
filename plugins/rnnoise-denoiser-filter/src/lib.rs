use obs_wrapper::{graphics::*, obs_register_module, obs_string, prelude::*, source::*};
use rnnoise_c::{DenoiseState, FRAME_SIZE};

struct Data {
    left_over: Vec<Vec<f32>>,
    state: Vec<DenoiseState>,
    sample_rate: usize,
    channels: usize,
}

struct RnnoiseDenoiserFilter {
    context: ModuleContext,
}

impl Sourceable for RnnoiseDenoiserFilter {
    fn get_id() -> ObsString {
        obs_string!("rnnoise_denoiser_filter")
    }
    fn get_type() -> SourceType {
        SourceType::FILTER
    }
}

impl GetNameSource<Data> for RnnoiseDenoiserFilter {
    fn get_name() -> ObsString {
        obs_string!("Rnnoise Denoiser Filter")
    }
}

impl CreatableSource<Data> for RnnoiseDenoiserFilter {
    fn create(
        settings: &mut SettingsContext,
        mut source: SourceContext,
        context: &mut GlobalContext,
    ) -> Data {
        let (sample_rate, channels) =
            context.with_audio(|audio| (audio.output_sample_rate(), audio.output_channels()));

        Data {
            left_over: vec![vec![0.; FRAME_SIZE]; channels],
            state: (0..channels).map(|_| DenoiseState::new()).collect(),
            sample_rate,
            channels,
        }
    }
}

impl UpdateSource<Data> for RnnoiseDenoiserFilter {
    fn update(
        data: &mut Option<Data>,
        settings: &mut SettingsContext,
        context: &mut GlobalContext,
    ) {
        if let Some(data) = data {
            let (sample_rate, channels) =
                context.with_audio(|audio| (audio.output_sample_rate(), audio.output_channels()));

            data.sample_rate = sample_rate;

            if data.channels != channels {
                data.left_over = vec![vec![0.; FRAME_SIZE]; channels];
                data.state = (0..channels).map(|_| DenoiseState::new()).collect();
            }
        }
    }
}

impl FilterAudioSource<Data> for RnnoiseDenoiserFilter {
    fn filter_audio(_data: &mut Option<Data>, audio: &mut audio::AudioDataContext) {
        let data = audio
            .get_channel_as_mut_slice(1)
            .expect("There was not a second channel!");

        for sample in data {
            *sample = 0.;
        }
    }
}

impl Module for RnnoiseDenoiserFilter {
    fn new(context: ModuleContext) -> Self {
        Self { context }
    }
    fn get_ctx(&self) -> &ModuleContext {
        &self.context
    }

    fn load(&mut self, load_context: &mut LoadContext) -> bool {
        let source = load_context
            .create_source_builder::<RnnoiseDenoiserFilter, Data>()
            .enable_get_name()
            .enable_create()
            .enable_update()
            .enable_filter_audio()
            .build();

        load_context.register_source(source);

        true
    }

    fn description() -> ObsString {
        obs_string!("A filter that focused the currently focused Xorg window.")
    }
    fn name() -> ObsString {
        obs_string!("Rnnoise Denoiser Filter")
    }
    fn author() -> ObsString {
        obs_string!("Bennett Hardwick")
    }
}

obs_register_module!(RnnoiseDenoiserFilter);

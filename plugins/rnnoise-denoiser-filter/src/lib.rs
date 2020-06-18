use obs_wrapper::{graphics::*, obs_register_module, obs_string, prelude::*, source::*};
use rnnoise_c::{DenoiseState, FRAME_SIZE};

struct Data {
    left_over: Vec<f32>,
    state: DenoiseState,
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
        _context: &mut GlobalContext,
    ) -> Data {
        let state = DenoiseState::new();

        todo!()
    }
}

impl UpdateSource<Data> for RnnoiseDenoiserFilter {
    fn update(
        data: &mut Option<Data>,
        settings: &mut SettingsContext,
        _context: &mut GlobalContext,
    ) {
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
            .build();

        load_context.register_source(source);

        true
    }

    fn description() -> ObsString {
        obs_string!("A filter that focused the currently focused Xorg window.")
    }
    fn name() -> ObsString {
        obs_string!("Scroll Focus Filter")
    }
    fn author() -> ObsString {
        obs_string!("Bennett Hardwick")
    }
}

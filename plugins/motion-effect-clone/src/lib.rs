use obs_wrapper::{graphics::*, obs_register_module, obs_string, prelude::*, source::*};

struct Data {
}

struct MotionEffect {
    context: ModuleContext,
}

impl Sourceable for MotionEffect {
    fn get_id() -> ObsString {
        obs_string!("scroll_focus_filter")
    }
    fn get_type() -> SourceType {
        SourceType::TRANSITION
    }
}

impl GetNameSource<Data> for MotionEffect {
    fn get_name() -> ObsString {
        obs_string!("Scroll Focus Filter")
    }
}

impl Module for MotionEffect {
    fn new(context: ModuleContext) -> Self {
        Self { context }
    }
    fn get_ctx(&self) -> &ModuleContext {
        &self.context
    }

    fn load(&mut self, load_context: &mut LoadContext) -> bool {
        let source = load_context
            .create_source_builder::<MotionEffect, Data>()
            .enable_get_name()
            .enable_create()
            .enable_get_properties()
            .enable_update()
            .enable_video_render()
            .enable_video_tick()
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

obs_register_module!(MotionEffect);

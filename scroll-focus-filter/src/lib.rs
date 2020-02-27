use obs_rs::{
    info, obs_register_module, obs_string,
    source::{
        properties::{Properties, SettingsContext},
        traits::*,
        SourceContext, SourceType,
    },
    warning, ActiveContext, LoadContext, Module, ModuleContext, ObsString,
};

struct Data {
    context: SourceContext,
}

struct ScrollFocusFilter {
    context: ModuleContext,
}

impl Sourceable for ScrollFocusFilter {
    fn get_id() -> ObsString {
        obs_string!("scroll_focus_filter")
    }
    fn get_type() -> SourceType {
        SourceType::FILTER
    }
}

impl GetNameSource for ScrollFocusFilter {
    fn get_name() -> ObsString {
        obs_string!("Scroll Focus Filter")
    }
}

impl Module for ScrollFocusFilter {
    fn new(context: ModuleContext) -> Self {
        Self { context }
    }
    fn get_ctx(&self) -> &ModuleContext {
        &self.context
    }

    fn load(&mut self, load_context: &mut LoadContext) -> bool {
        let source = load_context
            .create_source_builder::<ScrollFocusFilter, Data>()
            .enable_get_name()
            // .enable_create()
            // .enable_get_properties()
            // .enable_update()
            // .enable_transition_start()
            // .enable_transition_stop()
            .build();

        load_context.register_source(source);

        true
    }

    fn description() -> ObsString {
        obs_string!("A great thing")
    }
    fn name() -> ObsString {
        obs_string!("Scroll Focus Filter")
    }
    fn author() -> ObsString {
        obs_string!("Benny")
    }
}

obs_register_module!(ScrollFocusFilter);

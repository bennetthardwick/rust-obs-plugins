use obs_rs::{
    graphics::*,
    info, obs_register_module, obs_string,
    source::{
        properties::{Properties, SettingsContext},
        traits::*,
        SourceContext, SourceType,
    },
    warning, ActiveContext, LoadContext, Module, ModuleContext, ObsString,
};

struct Data {
    source: SourceContext,
    effect: GraphicsEffect,
    mul_val: GraphicsEffectParam,
    add_val: GraphicsEffectParam,
    image: GraphicsEffectParam,
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

impl CreatableSource<Data> for ScrollFocusFilter {
    fn create(settings: &SettingsContext, source: SourceContext) -> Data {
        if let Some(mut effect) = GraphicsEffect::from_effect_string(
            obs_string!(include_str!("./crop_filter.effect")),
            obs_string!("crop_filter.effect"),
        ) {
            if let Some(add_val) = effect.get_effect_param_by_name(obs_string!("add_val")) {
                if let Some(mul_val) = effect.get_effect_param_by_name(obs_string!("mul_val")) {
                    if let Some(image) = effect.get_effect_param_by_name(obs_string!("image")) {
                        source.update_source_settings(settings);
                        return Data {
                            source,
                            effect,
                            add_val,
                            mul_val,
                            image,
                        };
                    }
                }
            }

            panic!("Failed to find correct effect params!");
        } else {
            panic!("Could not load crop filter effect!");
        }
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
            .enable_create()
            .with_output_flags(1)
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

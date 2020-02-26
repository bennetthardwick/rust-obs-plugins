use obs_rs::{
    obs_register_module, obs_string,
    source::{traits::*, SettingsContext, SourceContext, SourceType},
    LoadContext, Module, ModuleContext, ObsString,
};

struct TransitionData {
    context: SourceContext,
}

struct MotionTransition {
    ctx: ModuleContext,
}

impl GetNameSource for MotionTransition {
    fn get_name() -> ObsString {
        obs_string!("Motion")
    }
}

impl Sourceable for MotionTransition {
    fn get_id() -> ObsString {
        obs_string!("motion-transition")
    }

    fn get_type() -> SourceType {
        SourceType::INPUT
    }
}

struct Creatable {}

impl CreatableSource<Creatable> for MotionTransition {
    fn create(_: &SettingsContext, _: SourceContext) -> Creatable {
        Creatable {}
    }
}

impl Module for MotionTransition {
    fn new(ctx: ModuleContext) -> Self {
        Self { ctx }
    }
    fn get_ctx(&self) -> &ModuleContext {
        &self.ctx
    }

    fn load(&mut self, load_context: &mut LoadContext) -> bool {
        let source = load_context
            .create_source_builder::<MotionTransition, Creatable>()
            .enable_get_name()
            .enable_create()
            .build();

        load_context.register_source(source);

        true
    }

    fn description() -> ObsString {
        obs_string!("A great thing")
    }
    fn name() -> ObsString {
        obs_string!("Motion Effects")
    }
    fn author() -> ObsString {
        obs_string!("Benny")
    }
}

obs_register_module!(MotionTransition);

use obs_rs::{
    obs_register_module,
    source::{traits::*, SourceContext, SourceType},
    LoadContext, Module, ModuleContext,
};

struct TransitionData {
    context: SourceContext,
}

struct MotionTransition {
    ctx: ModuleContext,
}

impl Sourceable for MotionTransition {
    fn get_id() -> &'static str {
        "motion-transition"
    }
    fn get_type() -> SourceType {
        SourceType::TRANSITION
    }
}

impl GetNameSource for MotionTransition {
    fn get_name() -> &'static str {
        "Motion"
    }
}

impl Module for MotionTransition {
    fn new(ctx: ModuleContext) -> Self {
        Self { ctx }
    }
    fn get_ctx(&self) -> &ModuleContext {
        &self.ctx
    }

    fn load(&mut self, load_context: &LoadContext) -> bool {
        let source = load_context
            .create_source_builder::<MotionTransition, ()>("motion-transition", SourceType::TRANSITION)
            .enable_get_name()
            .build();

        load_context.register_source(source);

        true
    }

    fn description() -> &'static str {
        "A great thing"
    }
    fn name() -> &'static str {
        "Motion Effects"
    }
    fn author() -> &'static str {
        "Benny"
    }
}

obs_register_module!(MotionTransition);

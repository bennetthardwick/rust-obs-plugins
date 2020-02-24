use obs_rs::{obs_register_module, Module, ModuleContext};

struct Wow {
    ctx: ModuleContext,
}

impl Module for Wow {
    fn new(ctx: ModuleContext) -> Self {
        Self { ctx }
    }
    fn get_ctx(&self) -> &ModuleContext {
        &self.ctx
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

obs_register_module!(Wow);

use obs_rs::{
    info, obs_register_module, obs_string,
    source::{
        properties::{Properties, SettingsContext},
        traits::*,
        SourceContext, SourceType,
    },
    warning, ActiveContext, LoadContext, Module, ModuleContext, ObsString,
};

struct TransitionData {
    acc_x: f64,
    acc_y: f64,
    scene_transitioning: bool,
    transitioning: bool,
}

type D = TransitionData;

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

impl GetPropertiesSource<D> for MotionTransition {
    fn get_properties(_data: &mut Option<D>, properties: &mut Properties) {
        properties.add_float_slider(
            obs_string!("bezier_x"),
            obs_string!("Acceleration.X"),
            -0.5,
            0.5,
            0.01,
        );
        properties.add_float_slider(
            obs_string!("bezier_y"),
            obs_string!("Acceleration.Y"),
            -0.5,
            0.5,
            0.01,
        );
    }
}

impl CreatableSource<D> for MotionTransition {
    fn create(_: &SettingsContext, _: SourceContext) -> D {
        TransitionData {
            acc_x: 0.,
            acc_y: 0.,
            scene_transitioning: false,
            transitioning: false,
        }
    }
}

impl UpdateSource<D> for MotionTransition {
    fn update(data: &mut Option<D>, settings: &SettingsContext, context: &mut ActiveContext) {
        if let Some(data) = data {
            if let Some(acc_y) = settings.get_float(obs_string!("bezier_y")) {
                data.acc_y = acc_y;
            }

            if let Some(acc_x) = settings.get_float(obs_string!("bezier_x")) {
                data.acc_x = acc_x;
            }
        } else {
            warning!("MotionTransition data was None");
        }
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
            .create_source_builder::<MotionTransition, D>()
            .enable_get_name()
            .enable_create()
            .enable_get_properties()
            .enable_update()
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

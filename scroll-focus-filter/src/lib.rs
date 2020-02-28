mod server;

use obs_rs::{
    graphics::*,
    info, obs_register_module, obs_string,
    source::{
        context::VideoRenderContext,
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

    zoom: f64,
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

impl GetPropertiesSource<Data> for ScrollFocusFilter {
    fn get_properties(data: &mut Option<Data>, properties: &mut Properties) {
        properties.add_float_slider(
            obs_string!("zoom"),
            obs_string!("Amount to zoom in window"),
            0.,
            2.,
            0.001,
        );
    }
}

impl VideoRenderSource<Data> for ScrollFocusFilter {
    fn video_render(
        data: &mut Option<Data>,
        context: &mut ActiveContext,
        render: &mut VideoRenderContext,
    ) {
        if let Some(data) = data {
            let effect = &mut data.effect;
            let source = &mut data.source;
            let param_add = &mut data.add_val;
            let param_mul = &mut data.mul_val;
            let zoom = data.zoom;

            let mut cx: u32 = 0;
            let mut cy: u32 = 0;

            source.do_with_target(|target| {
                cx = target.get_base_width();
                cy = target.get_base_height();
            });

            source.process_filter(
                render,
                effect,
                cx,
                cy,
                GraphicsColorFormat::RGBA,
                GraphicsAllowDirectRendering::NoDirectRendering,
                |context, effect| {
                    let amount = 1. - (zoom / 4.);

                    param_add.set_vec2(context, &Vec2::new(0., 0.));
                    param_mul.set_vec2(context, &Vec2::new(amount as f32, amount as f32));
                },
            );
        }
    }
}

impl CreatableSource<Data> for ScrollFocusFilter {
    fn create(settings: &SettingsContext, mut source: SourceContext) -> Data {
        if let Some(mut effect) = GraphicsEffect::from_effect_string(
            obs_string!(include_str!("./crop_filter.effect")),
            obs_string!("crop_filter.effect"),
        ) {
            if let Some(add_val) = effect.get_effect_param_by_name(obs_string!("add_val")) {
                if let Some(mul_val) = effect.get_effect_param_by_name(obs_string!("mul_val")) {
                    if let Some(image) = effect.get_effect_param_by_name(obs_string!("image")) {
                        let zoom = if let Some(zoom) = settings.get_float(obs_string!("zoom")) {
                            zoom
                        } else {
                            0.
                        };

                        source.update_source_settings(settings);
                        return Data {
                            source,
                            effect,
                            add_val,
                            mul_val,
                            image,
                            zoom,
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

impl UpdateSource<Data> for ScrollFocusFilter {
    fn update(data: &mut Option<Data>, settings: &SettingsContext, context: &mut ActiveContext) {
        if let Some(data) = data {
            if let Some(zoom) = settings.get_float(obs_string!("zoom")) {
                data.zoom = zoom;
            }
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
            .enable_get_properties()
            .enable_update()
            .enable_video_render()
            .with_output_flags(1)
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

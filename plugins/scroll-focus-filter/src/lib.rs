mod server;

use server::{Server, WindowSnapshot};

use obs_wrapper::{graphics::*, obs_register_module, obs_string, prelude::*, source::*};

use crossbeam_channel::{unbounded, Receiver, Sender};

enum FilterMessage {
    CloseConnection,
}

enum ServerMessage {
    Snapshot(WindowSnapshot),
}

struct Data {
    source: SourceContext,
    effect: GraphicsEffect,

    base_dimension: GraphicsEffectVec2Param,
    base_dimension_i: GraphicsEffectVec2Param,

    mul_val: GraphicsEffectVec2Param,
    add_val: GraphicsEffectVec2Param,

    image: GraphicsEffectTextureParam,

    sampler: GraphicsSamplerState,

    send: Sender<FilterMessage>,
    receive: Receiver<ServerMessage>,

    current: Vec2,
    from: Vec2,
    target: Vec2,

    animation_time: f64,

    current_zoom: f64,
    from_zoom: f64,
    target_zoom: f64,
    internal_zoom: f64,
    padding: f64,

    progress: f64,

    screen_width: u32,
    screen_height: u32,
    screen_x: u32,
    screen_y: u32,
}

impl Drop for Data {
    fn drop(&mut self) {
        self.send.send(FilterMessage::CloseConnection).unwrap_or(());
    }
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

impl GetNameSource<Data> for ScrollFocusFilter {
    fn get_name() -> ObsString {
        obs_string!("Scroll Focus Filter")
    }
}

impl GetPropertiesSource<Data> for ScrollFocusFilter {
    fn get_properties(_data: &mut Option<Data>, properties: &mut Properties) {
        properties
            .add(
                obs_string!("zoom"),
                obs_string!("Amount to zoom in window"),
                NumberProp::new_float(0.001)
                    .with_range((1.)..=5.)
                    .with_slider(),
            )
            .add(
                obs_string!("screen_x"),
                obs_string!("Offset relative to top left screen - x"),
                NumberProp::new_int().with_range(..=3840 * 3u32),
            )
            .add(
                obs_string!("screen_y"),
                obs_string!("Offset relative to top left screen - y"),
                NumberProp::new_int().with_range(..=3840 * 3u32),
            )
            .add(
                obs_string!("padding"),
                obs_string!("Padding around each window"),
                NumberProp::new_float(0.001)
                    .with_range(..=0.5)
                    .with_slider(),
            )
            .add(
                obs_string!("screen_width"),
                obs_string!("Screen width"),
                NumberProp::new_int().with_range(..=3840 * 3u32),
            )
            .add(
                obs_string!("screen_height"),
                obs_string!("Screen height"),
                NumberProp::new_int().with_range(..=3840 * 3u32),
            )
            .add(
                obs_string!("animation_time"),
                obs_string!("Animation Time (s)"),
                NumberProp::new_float(0.001).with_range(0.3..=10.),
            );
    }
}

fn smooth_step(x: f32) -> f32 {
    let t = ((x / 1.).max(0.)).min(1.);
    t * t * (3. - 2. * t)
}

impl VideoTickSource<Data> for ScrollFocusFilter {
    fn video_tick(data: &mut Option<Data>, seconds: f32) {
        if let Some(data) = data {
            for ServerMessage::Snapshot(snapshot) in data.receive.try_iter() {
                let window_zoom = ((snapshot.width / (data.screen_width as f32))
                    .max(snapshot.height / (data.screen_height as f32))
                    as f64
                    + data.padding)
                    .max(data.internal_zoom)
                    .min(1.);

                if snapshot.x > (data.screen_width + data.screen_x) as f32
                    || snapshot.x < data.screen_x as f32
                    || snapshot.y < data.screen_y as f32
                    || snapshot.y > (data.screen_height + data.screen_y) as f32
                {
                    if data.target_zoom != 1. && data.target.x() != 0. && data.target.y() != 0. {
                        data.progress = 0.;
                        data.from_zoom = data.current_zoom;
                        data.target_zoom = 1.;

                        data.from.set(data.current.x(), data.current.y());
                        data.target.set(0., 0.);
                    }
                } else {
                    let x = (snapshot.x + (snapshot.width / 2.) - (data.screen_x as f32))
                        / (data.screen_width as f32);
                    let y = (snapshot.y + (snapshot.height / 2.) - (data.screen_y as f32))
                        / (data.screen_height as f32);

                    let target_x = (x - (0.5 * window_zoom as f32))
                        .min(1. - window_zoom as f32)
                        .max(0.);

                    let target_y = (y - (0.5 * window_zoom as f32))
                        .min(1. - window_zoom as f32)
                        .max(0.);

                    if (target_y - data.target.y()).abs() > 0.001
                        || (target_x - data.target.x()).abs() > 0.001
                        || (window_zoom - data.target_zoom).abs() > 0.001
                    {
                        data.progress = 0.;

                        data.from_zoom = data.current_zoom;
                        data.target_zoom = window_zoom;

                        data.from.set(data.current.x(), data.current.y());

                        data.target.set(target_x, target_y);
                    }
                }
            }

            data.progress = (data.progress + seconds as f64 / data.animation_time).min(1.);

            let adjusted_progress = smooth_step(data.progress as f32);

            data.current.set(
                data.from.x() + (data.target.x() - data.from.x()) * adjusted_progress,
                data.from.y() + (data.target.y() - data.from.y()) * adjusted_progress,
            );

            data.current_zoom =
                data.from_zoom + (data.target_zoom - data.from_zoom) * adjusted_progress as f64;
        }
    }
}

impl VideoRenderSource<Data> for ScrollFocusFilter {
    fn video_render(
        data: &mut Option<Data>,
        _context: &mut GlobalContext,
        render: &mut VideoRenderContext,
    ) {
        if let Some(data) = data {
            let effect = &mut data.effect;
            let source = &mut data.source;
            let param_add = &mut data.add_val;

            let param_mul = &mut data.mul_val;

            let param_base = &mut data.base_dimension;
            let param_base_i = &mut data.base_dimension_i;

            let image = &mut data.image;
            let sampler = &mut data.sampler;

            let current = &mut data.current;

            let zoom = data.current_zoom as f32;

            let mut target_cx: u32 = 1;
            let mut target_cy: u32 = 1;

            let cx = source.get_base_width();
            let cy = source.get_base_height();

            source.do_with_target(|target| {
                target_cx = target.get_base_width();
                target_cy = target.get_base_height();
            });

            source.process_filter_tech(
                render,
                effect,
                (target_cx, target_cy),
                GraphicsColorFormat::RGBA,
                GraphicsAllowDirectRendering::NoDirectRendering,
                obs_string!("DrawUndistort"),
                |context, _effect| {
                    param_add.set_vec2(context, &Vec2::new(current.x(), current.y()));
                    param_mul.set_vec2(context, &Vec2::new(zoom, zoom));

                    param_base.set_vec2(context, &Vec2::new(cx as _, cy as _));
                    param_base_i.set_vec2(context, &Vec2::new(1. / (cx as f32), 1. / (cy as f32)));

                    image.set_next_sampler(context, sampler);
                },
            );
        }
    }
}

impl CreatableSource<Data> for ScrollFocusFilter {
    fn create(create: &mut CreatableSourceContext<Data>, mut source: SourceContext) -> Data {
        let mut effect = GraphicsEffect::from_effect_string(
            obs_string!(include_str!("./crop_filter.effect")),
            obs_string!("crop_filter.effect"),
        )
        .expect("Could not load crop filter effect!");

        let settings = &mut create.settings;

        if let (
            Some(image),
            Some(add_val),
            Some(base_dimension),
            Some(base_dimension_i),
            Some(mul_val),
        ) = (
            effect.get_effect_param_by_name(obs_string!("image")),
            effect.get_effect_param_by_name(obs_string!("add_val")),
            effect.get_effect_param_by_name(obs_string!("base_dimension")),
            effect.get_effect_param_by_name(obs_string!("base_dimension_i")),
            effect.get_effect_param_by_name(obs_string!("mul_val")),
        ) {
            let zoom = 1. / settings.get(obs_string!("zoom")).unwrap_or(1.);

            let sampler = GraphicsSamplerState::from(GraphicsSamplerInfo::default());

            let screen_width = settings.get(obs_string!("screen_width")).unwrap_or(1920) as u32;
            let screen_height = settings.get(obs_string!("screen_height")).unwrap_or(1080) as u32;

            let screen_x = settings.get(obs_string!("screen_x")).unwrap_or(0) as u32;
            let screen_y = settings.get(obs_string!("screen_y")).unwrap_or(0) as u32;

            let animation_time = settings.get(obs_string!("animation_time")).unwrap_or(0.3);

            let (send_filter, receive_filter) = unbounded::<FilterMessage>();
            let (send_server, receive_server) = unbounded::<ServerMessage>();

            std::thread::spawn(move || {
                let mut server = Server::new().unwrap();

                loop {
                    if let Some(snapshot) = server.wait_for_event() {
                        send_server
                            .send(ServerMessage::Snapshot(snapshot))
                            .unwrap_or(());
                    }

                    if let Ok(msg) = receive_filter.try_recv() {
                        match msg {
                            FilterMessage::CloseConnection => {
                                return;
                            }
                        }
                    }
                }
            });

            source.update_source_settings(settings);

            return Data {
                source,
                effect,
                add_val,
                mul_val,

                base_dimension,
                base_dimension_i,

                image,

                sampler,

                animation_time,

                current_zoom: zoom,
                from_zoom: zoom,
                target_zoom: zoom,
                internal_zoom: zoom,

                send: send_filter,
                receive: receive_server,

                current: Vec2::new(0., 0.),
                from: Vec2::new(0., 0.),
                target: Vec2::new(0., 0.),
                padding: 0.1,

                progress: 1.,

                screen_height,
                screen_width,
                screen_x,
                screen_y,
            };
        }

        panic!("Failed to find correct effect params!");
    }
}

impl UpdateSource<Data> for ScrollFocusFilter {
    fn update(data: &mut Option<Data>, settings: &mut DataObj, _context: &mut GlobalContext) {
        if let Some(data) = data {
            if let Some(zoom) = settings.get::<f64, _>(obs_string!("zoom")) {
                data.from_zoom = data.current_zoom;
                data.internal_zoom = 1. / zoom;
                data.target_zoom = 1. / zoom;
            }

            if let Some(screen_width) = settings.get::<i64, _>(obs_string!("screen_width")) {
                data.screen_width = screen_width as u32;
            }

            if let Some(padding) = settings.get(obs_string!("padding")) {
                data.padding = padding;
            }

            if let Some(animation_time) = settings.get(obs_string!("animation_time")) {
                data.animation_time = animation_time;
            }

            if let Some(screen_height) = settings.get::<i64, _>(obs_string!("screen_height")) {
                data.screen_height = screen_height as u32;
            }
            if let Some(screen_x) = settings.get::<i64, _>(obs_string!("screen_x")) {
                data.screen_x = screen_x as u32;
            }
            if let Some(screen_y) = settings.get::<i64, _>(obs_string!("screen_y")) {
                data.screen_y = screen_y as u32;
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

obs_register_module!(ScrollFocusFilter);

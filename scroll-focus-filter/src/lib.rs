mod server;

use server::{Server, WindowSnapshot};

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

use crossbeam_channel::{unbounded, Receiver, Sender};
use std::thread::JoinHandle;

enum FilterMessage {
    CloseConnection,
}

enum ServerMessage {
    Snapshot(WindowSnapshot),
}

struct Data {
    source: SourceContext,
    effect: GraphicsEffect,
    mul_val: GraphicsEffectParam,
    add_val: GraphicsEffectParam,
    image: GraphicsEffectParam,

    thread: Option<JoinHandle<()>>,
    send: Sender<FilterMessage>,
    receive: Receiver<ServerMessage>,

    current: Vec2,
    target: Vec2,

    current_zoom: f64,
    target_zoom: f64,

    screen_width: u32,
    screen_height: u32,
    screen_x: u32,
    screen_y: u32,
}

impl Drop for Data {
    fn drop(&mut self) {
        self.send.send(FilterMessage::CloseConnection).unwrap();
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
        properties.add_int(
            obs_string!("offset_x"),
            obs_string!("Offset relative to top left screen - x"),
            0,
            3840 * 3,
            1,
        );
        properties.add_int(
            obs_string!("offset_y"),
            obs_string!("Offset relative to top left screen - y"),
            0,
            3840 * 3,
            1,
        );
        properties.add_int(
            obs_string!("screen_width"),
            obs_string!("Screen width"),
            0,
            3840 * 3,
            1,
        );
        properties.add_int(
            obs_string!("screen_height"),
            obs_string!("Screen height"),
            0,
            3840 * 3,
            1,
        );
    }
}

impl VideoTickSource<Data> for ScrollFocusFilter {
    fn video_tick(data: &mut Option<Data>, seconds: f32) {
        if let Some(data) = data {
            for message in data.receive.try_iter() {
                match message {
                    ServerMessage::Snapshot(snapshot) => {
                        let mut x = ((snapshot.x + (snapshot.width / 2.) - (data.screen_x as f32))
                            / (data.screen_width as f32))
                            .max(0.25)
                            .min(0.75)
                            - 0.25;
                        let mut y = ((snapshot.y + (snapshot.height / 2.)
                            - (data.screen_y as f32))
                            / (data.screen_height as f32))
                            .max(0.25)
                            .min(0.75)
                            - 0.25;

                        println!("X {}, Y {}", x, y);

                        if x < 0. || x > 1. || y > 1. || y < 0. {
                            x = 0.;
                            y = 0.;
                            data.target_zoom = 0.;
                        }

                        data.target
                            .set(x * data.current_zoom as f32, y * data.current_zoom as f32);
                    }
                }
            }

            data.current.set(
                data.current.x() + ((data.target.x() - data.current.x()) / 0.15) * seconds,
                data.current.y() + ((data.target.y() - data.current.y()) / 0.15) * seconds,
            )
        }
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

            let current = &mut data.current;

            let zoom = data.current_zoom;

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
                    let amount = zoom;

                    param_add.set_vec2(context, &Vec2::new(current.x(), current.y()));
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
                        let zoom = settings.get_float(obs_string!("zoom")).unwrap_or(0.);

                        let screen_width = settings
                            .get_int(obs_string!("screen_width"))
                            .unwrap_or(1920) as u32;
                        let screen_height = settings
                            .get_int(obs_string!("screen_height"))
                            .unwrap_or(1080) as u32;

                        let screen_x =
                            settings.get_int(obs_string!("screen_x")).unwrap_or(0) as u32;
                        let screen_y =
                            settings.get_int(obs_string!("screen_y")).unwrap_or(0) as u32;

                        let (send_filter, receive_filter) = unbounded::<FilterMessage>();
                        let (send_server, receive_server) = unbounded::<ServerMessage>();

                        let handle = std::thread::spawn(move || {
                            let mut server = Server::new().unwrap();

                            loop {
                                if let Some(snapshot) = server.wait_for_event() {
                                    send_server.send(ServerMessage::Snapshot(snapshot)).unwrap();
                                }

                                for msg in receive_filter.try_iter() {
                                    match msg {
                                        FilterMessage::CloseConnection => {
                                            println!("Got close message");
                                            break;
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
                            image,

                            current_zoom: zoom,
                            target_zoom: zoom,

                            thread: Some(handle),
                            send: send_filter,
                            receive: receive_server,

                            current: Vec2::new(0.5, 0.5),
                            target: Vec2::new(0.5, 0.5),

                            screen_height,
                            screen_width,
                            screen_x,
                            screen_y,
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
                data.target_zoom = zoom;
                data.current_zoom = zoom;
            }

            if let Some(screen_width) = settings.get_int(obs_string!("screen_width")) {
                data.screen_width = screen_width as u32;
            }

            if let Some(screen_height) = settings.get_int(obs_string!("screen_height")) {
                data.screen_height = screen_height as u32;
            }
            if let Some(screen_x) = settings.get_int(obs_string!("screen_x")) {
                data.screen_x = screen_x as u32;
            }
            if let Some(screen_y) = settings.get_int(obs_string!("screen_y")) {
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

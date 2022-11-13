use obs_sys::{obs_key_event, obs_mouse_event};

use super::context::{CreatableSourceContext, GlobalContext, VideoRenderContext};
use super::{EnumActiveContext, EnumAllContext, SourceContext, SourceType};
use crate::data::DataObj;
use crate::media::state::MediaState;
use crate::media::{audio::AudioDataContext, video::VideoDataSourceContext};
use crate::properties::Properties;
use crate::string::ObsString;

pub trait Sourceable: Sized {
    fn get_id() -> ObsString;
    fn get_type() -> SourceType;
    fn create(create: &mut CreatableSourceContext<Self>, source: SourceContext) -> Self;
}

macro_rules! simple_trait {
    ($($f:ident => $t:ident $(-> $ret:ty)?)*) => ($(
        pub trait $t: Sized {
            fn $f(&mut self) $(-> $ret)?;
        }
    )*)
}

pub trait GetNameSource {
    fn get_name() -> ObsString;
}

simple_trait!(
    get_width => GetWidthSource -> u32
    get_height => GetHeightSource -> u32
    activate => ActivateSource
    deactivate => DeactivateSource
);

pub trait UpdateSource: Sized {
    fn update(&mut self, settings: &mut DataObj, context: &mut GlobalContext);
}

pub trait MouseWheelSource: Sized {
    fn mouse_wheel(&mut self, event: obs_mouse_event, xdelta: i32, ydelta: i32);
}

pub trait MouseClickSource: Sized {
    fn mouse_click(
        &mut self,
        event: obs_mouse_event,
        button: super::MouseButton,
        pressed: bool,
        click_count: u8,
    );
}

pub trait MouseMoveSource: Sized {
    fn mouse_move(&mut self, event: obs_mouse_event, leave: bool);
}

pub trait KeyClickSource: Sized {
    fn key_click(&mut self, event: obs_key_event, pressed: bool);
}

pub trait FocusSource: Sized {
    fn focus(&mut self, focused: bool);
}

pub trait VideoRenderSource: Sized {
    fn video_render(&mut self, context: &mut GlobalContext, render: &mut VideoRenderContext);
}

pub trait AudioRenderSource: Sized {
    fn audio_render(&mut self, context: &mut GlobalContext);
}

pub trait GetPropertiesSource: Sized {
    fn get_properties(&mut self) -> Properties;
}

pub trait VideoTickSource: Sized {
    fn video_tick(&mut self, seconds: f32);
}

pub trait EnumActiveSource: Sized {
    fn enum_active_sources(&mut self, context: &EnumActiveContext);
}

pub trait EnumAllSource: Sized {
    fn enum_all_sources(&mut self, context: &EnumAllContext);
}

simple_trait!(
    transition_start => TransitionStartSource
    transition_stop => TransitionStopSource
);

pub trait FilterAudioSource: Sized {
    fn filter_audio(&mut self, audio: &mut AudioDataContext);
}

pub trait FilterVideoSource: Sized {
    fn filter_video(&mut self, video: &mut VideoDataSourceContext);
}

pub trait MediaPlayPauseSource: Sized {
    fn play_pause(&mut self, pause: bool);
}

pub trait MediaGetStateSource: Sized {
    fn get_state(&mut self) -> MediaState;
}

pub trait MediaSetTimeSource: Sized {
    fn set_time(&mut self, milliseconds: i64);
}

pub trait GetDefaultsSource {
    fn get_defaults(settings: &mut DataObj);
}

simple_trait!(
    restart => MediaRestartSource
    stop => MediaStopSource
    next => MediaNextSource
    previous => MediaPreviousSource
    get_duration => MediaGetDurationSource -> i64
    get_time => MediaGetTimeSource -> i64
);

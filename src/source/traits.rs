use super::context::{CreatableSourceContext, GlobalContext, VideoRenderContext};
use crate::properties::Properties;
use super::{audio::AudioDataContext, media::MediaState};
use super::{video::VideoDataContext};
use super::{EnumActiveContext, EnumAllContext, SourceContext, SourceType};
use crate::data::DataObj;
use crate::string::ObsString;

pub trait Sourceable {
    fn get_id() -> ObsString;
    fn get_type() -> SourceType;
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

pub trait CreatableSource: Sized {
    fn create(create: &mut CreatableSourceContext<Self>, source: SourceContext) -> Self;
}

pub trait UpdateSource: Sized {
    fn update(&mut self, settings: &mut DataObj, context: &mut GlobalContext);
}

pub trait VideoRenderSource: Sized {
    fn video_render(
        &mut self,
        context: &mut GlobalContext,
        render: &mut VideoRenderContext,
    );
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
    fn filter_video(&mut self, audio: &mut VideoDataContext);
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

use super::context::{GlobalContext, VideoRenderContext};
use super::properties::Properties;
use super::{audio::AudioDataContext, media::MediaState};
use super::{EnumActiveContext, EnumAllContext, SourceContext, SourceType};
use crate::data::DataObj;
use crate::string::ObsString;

pub trait Sourceable {
    fn get_id() -> ObsString;
    fn get_type() -> SourceType;
}

macro_rules! simple_trait {
    ($($f:ident => $t:ident $(-> $ret:ty)?)*) => ($(
        pub trait $t<D> {
            fn $f(data: &mut Option<D>) $(-> $ret)?;
        }
    )*)
}

pub trait GetNameSource<D> {
    fn get_name() -> ObsString;
}

simple_trait!(
    get_width => GetWidthSource -> u32
    get_height => GetHeightSource -> u32
    activate => ActivateSource
    deactivate => DeactivateSource
);

pub trait CreatableSource<D> {
    fn create(settings: &mut DataObj, source: SourceContext, context: &mut GlobalContext) -> D;
}

pub trait UpdateSource<D> {
    fn update(data: &mut Option<D>, settings: &mut DataObj, context: &mut GlobalContext);
}

pub trait VideoRenderSource<D> {
    fn video_render(
        data: &mut Option<D>,
        context: &mut GlobalContext,
        render: &mut VideoRenderContext,
    );
}

pub trait AudioRenderSource<D> {
    fn audio_render(data: &mut Option<D>, context: &mut GlobalContext);
}

pub trait GetPropertiesSource<D> {
    fn get_properties(data: &mut Option<D>, properties: &mut Properties);
}

pub trait VideoTickSource<D> {
    fn video_tick(data: &mut Option<D>, seconds: f32);
}

pub trait EnumActiveSource<D> {
    fn enum_active_sources(data: &mut Option<D>, context: &EnumActiveContext);
}

pub trait EnumAllSource<D> {
    fn enum_all_sources(data: &mut Option<D>, context: &EnumAllContext);
}

simple_trait!(
    transition_start => TransitionStartSource
    transition_stop => TransitionStopSource
);

pub trait FilterAudioSource<D> {
    fn filter_audio(data: &mut Option<D>, audio: &mut AudioDataContext);
}

pub trait MediaPlayPauseSource<D> {
    fn play_pause(data: &mut Option<D>, pause: bool);
}

pub trait MediaGetStateSource<D> {
    fn get_state(data: &mut Option<D>) -> MediaState;
}

pub trait GetDefaultsSource<D> {
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

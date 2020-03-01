use super::context::{ActiveContext, VideoRenderContext};
use super::properties::{Properties, SettingsContext};
use super::{EnumActiveContext, EnumAllContext, SourceContext, SourceType};

use crate::ObsString;

pub trait Sourceable {
    fn get_id() -> ObsString;
    fn get_type() -> SourceType;
}

pub trait GetNameSource {
    fn get_name() -> ObsString;
}

pub trait GetWidthSource<D> {
    fn get_width(data: &mut Option<D>) -> u32;
}

pub trait GetHeightSource<D> {
    fn get_height(data: &Option<D>) -> u32;
}

pub trait CreatableSource<D> {
    fn create(settings: &mut SettingsContext, source: SourceContext) -> D;
}

pub trait UpdateSource<D> {
    fn update(data: &mut Option<D>, settings: &mut SettingsContext, context: &mut ActiveContext);
}

pub trait VideoRenderSource<D> {
    fn video_render(
        data: &mut Option<D>,
        context: &mut ActiveContext,
        render: &mut VideoRenderContext,
    );
}

pub trait AudioRenderSource<D> {
    fn audio_render(data: &mut Option<D>, context: &mut ActiveContext);
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

pub trait TransitionStartSource<D> {
    fn transition_start(data: &mut Option<D>);
}

pub trait TransitionStopSource<D> {
    fn transition_stop(data: &mut Option<D>);
}

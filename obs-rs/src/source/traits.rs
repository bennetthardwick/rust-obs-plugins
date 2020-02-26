use super::context::ActiveContext;
use super::{
    EnumActiveContext, EnumAllContext, PropertiesContext, SettingsContext, SourceContext,
    SourceType,
};

use crate::ObsString;

pub trait Sourceable {
    fn get_id() -> ObsString;
    fn get_type() -> SourceType;
}

pub trait GetNameSource {
    fn get_name() -> ObsString;
}

pub trait GetWidthSource<D> {
    fn get_width(data: &D) -> u32;
}

pub trait GetHeightSource<D> {
    fn get_height(data: &D) -> u32;
}

pub trait CreatableSource<D> {
    fn create(settings: &SettingsContext, source: SourceContext) -> D;
}

pub trait UpdateSource<D> {
    fn update(data: &mut D, settings: &SettingsContext, context: &ActiveContext);
}

pub trait VideoRenderSource<D> {
    fn video_render(data: &mut D, context: &ActiveContext);
}

pub trait AudioRenderSource<D> {
    fn audio_render(data: &mut D, context: &ActiveContext);
}

pub trait GetPropertiesSource<D> {
    fn get_properties(data: &mut D, context: &PropertiesContext);
}

pub trait EnumActiveSource<D> {
    fn enum_active_sources(data: &mut D, context: &EnumActiveContext);
}

pub trait EnumAllSource<D> {
    fn enum_all_sources(data: &mut D, context: &EnumAllContext);
}

pub trait TransitionStartSource<D> {
    fn transition_start(data: &mut D);
}

pub trait TransitionStopSource<D> {
    fn transition_stop(data: &mut D);
}

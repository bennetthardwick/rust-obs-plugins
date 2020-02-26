use super::{SettingsContext, SourceContext, SourceType};

pub trait Sourceable {
    fn get_id() -> &'static str;
    fn get_type() -> SourceType;
}

pub trait GetNameSource {
    fn get_name() -> &'static str;
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

use crate::{
    source::SourceRef,
    string::{DisplayExt as _, ObsString},
    wrapper::PtrWrapper,
};
use obs_sys::{
    obs_scene_add, obs_scene_get_ref, obs_scene_get_source, obs_scene_release, obs_scene_t,
    obs_sceneitem_addref, obs_sceneitem_release, obs_sceneitem_t, obs_sceneitem_visible,
};

use super::Result;

pub struct SceneRef {
    inner: *mut obs_scene_t,
}

impl std::fmt::Debug for SceneRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SceneRef")
            .field(&self.name().display())
            .field(&self.inner)
            .finish()
    }
}

impl_ptr_wrapper!(SceneRef, obs_scene_t, obs_scene_get_ref, obs_scene_release);

impl SceneRef {
    pub fn name(&self) -> Result<ObsString> {
        self.as_source().name()
    }

    pub fn as_source(&self) -> SourceRef {
        let ptr = unsafe {
            // as doc said "The scene’s source. Does not increment the reference"
            // we should manually add_ref for it
            obs_scene_get_source(self.inner)
        };
        SourceRef::from_raw(ptr).expect("obs_scene_get_source")
    }

    pub fn add_source(&self, source: SourceRef) -> SceneItemRef {
        let ptr = unsafe {
            let ptr = obs_scene_add(self.inner, source.as_ptr_mut());
            // add ref for source, Docs said "A new scene item for a source within a scene.  Does not
            // increment the reference"
            obs_sceneitem_addref(ptr);
            ptr
        };
        SceneItemRef::from_raw(ptr).expect("obs_scene_add")
    }
}

pub struct SceneItemRef {
    inner: *mut obs_sceneitem_t,
}

unsafe fn scene_item_get_ref(ptr: *mut obs_sceneitem_t) -> *mut obs_sceneitem_t {
    obs_sceneitem_addref(ptr);
    ptr
}

impl_ptr_wrapper!(
    SceneItemRef,
    obs_sceneitem_t,
    scene_item_get_ref,
    obs_sceneitem_release
);

impl SceneItemRef {
    pub fn visible(&self) -> bool {
        unsafe { obs_sceneitem_visible(self.inner) }
    }
}

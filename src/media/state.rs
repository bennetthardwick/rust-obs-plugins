use obs_sys::{
    obs_media_state, obs_media_state_OBS_MEDIA_STATE_BUFFERING,
    obs_media_state_OBS_MEDIA_STATE_ENDED, obs_media_state_OBS_MEDIA_STATE_ERROR,
    obs_media_state_OBS_MEDIA_STATE_NONE, obs_media_state_OBS_MEDIA_STATE_OPENING,
    obs_media_state_OBS_MEDIA_STATE_PAUSED, obs_media_state_OBS_MEDIA_STATE_PLAYING,
    obs_media_state_OBS_MEDIA_STATE_STOPPED,
};

use crate::native_enum;

native_enum!(
/// OBS media state
MediaState, obs_media_state {
    None => OBS_MEDIA_STATE_NONE,
    Playing => OBS_MEDIA_STATE_PLAYING,
    Opening => OBS_MEDIA_STATE_OPENING,
    Buffering => OBS_MEDIA_STATE_BUFFERING,
    Paused => OBS_MEDIA_STATE_PAUSED,
    Stopped => OBS_MEDIA_STATE_STOPPED,
    Ended => OBS_MEDIA_STATE_ENDED,
    Error => OBS_MEDIA_STATE_ERROR,
});

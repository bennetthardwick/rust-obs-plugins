use obs_sys::{
    obs_media_state, obs_media_state_OBS_MEDIA_STATE_BUFFERING,
    obs_media_state_OBS_MEDIA_STATE_ENDED, obs_media_state_OBS_MEDIA_STATE_ERROR,
    obs_media_state_OBS_MEDIA_STATE_NONE, obs_media_state_OBS_MEDIA_STATE_OPENING,
    obs_media_state_OBS_MEDIA_STATE_PAUSED, obs_media_state_OBS_MEDIA_STATE_PLAYING,
    obs_media_state_OBS_MEDIA_STATE_STOPPED,
};

/// OBS media state
pub enum MediaState {
    None,
    Playing,
    Opening,
    Buffering,
    Paused,
    Stopped,
    Ended,
    Error,
}

impl MediaState {
    #[allow(dead_code)]
    pub(crate) fn from_native(state: obs_media_state) -> Option<Self> {
        match state {
            obs_media_state_OBS_MEDIA_STATE_NONE => Some(Self::None),
            obs_media_state_OBS_MEDIA_STATE_PLAYING => Some(Self::Playing),
            obs_media_state_OBS_MEDIA_STATE_OPENING => Some(Self::Opening),
            obs_media_state_OBS_MEDIA_STATE_BUFFERING => Some(Self::Buffering),
            obs_media_state_OBS_MEDIA_STATE_PAUSED => Some(Self::Paused),
            obs_media_state_OBS_MEDIA_STATE_STOPPED => Some(Self::Stopped),
            obs_media_state_OBS_MEDIA_STATE_ENDED => Some(Self::Ended),
            obs_media_state_OBS_MEDIA_STATE_ERROR => Some(Self::Error),
            _ => None,
        }
    }

    pub(crate) fn to_native(self) -> obs_media_state {
        match self {
            Self::None => obs_media_state_OBS_MEDIA_STATE_NONE,
            Self::Playing => obs_media_state_OBS_MEDIA_STATE_PLAYING,
            Self::Opening => obs_media_state_OBS_MEDIA_STATE_OPENING,
            Self::Buffering => obs_media_state_OBS_MEDIA_STATE_BUFFERING,
            Self::Paused => obs_media_state_OBS_MEDIA_STATE_PAUSED,
            Self::Stopped => obs_media_state_OBS_MEDIA_STATE_STOPPED,
            Self::Ended => obs_media_state_OBS_MEDIA_STATE_ENDED,
            Self::Error => obs_media_state_OBS_MEDIA_STATE_ERROR,
        }
    }
}

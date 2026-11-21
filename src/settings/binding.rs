//! GPUI global binding for shared media preferences.
use crate::media::{ItemSortPreferences, PlaybackLanguagePreferences, SubtitleStyle};
use gpui::{App, Global};

impl Global for PlaybackLanguagePreferences {}
impl Global for ItemSortPreferences {}
impl Global for SubtitleStyle {}

impl ItemSortPreferences {
    pub(crate) fn get(cx: &App) -> Self {
        cx.try_global::<Self>().copied().unwrap_or_default()
    }

    pub(crate) fn apply(self, cx: &mut App) {
        if cx.try_global::<Self>() != Some(&self) {
            cx.set_global(self);
        }
    }
}

impl PlaybackLanguagePreferences {
    pub(crate) fn get(cx: &App) -> Self {
        cx.try_global::<Self>().copied().unwrap_or_default()
    }

    pub(crate) fn apply(self, cx: &mut App) {
        if cx.try_global::<Self>() != Some(&self) {
            cx.set_global(self);
            cx.refresh_windows();
        }
    }
}

impl SubtitleStyle {
    pub(crate) fn get(cx: &App) -> Self {
        cx.try_global::<Self>()
            .copied()
            .unwrap_or_default()
            .normalized()
    }

    pub(crate) fn apply(self, cx: &mut App) {
        let style = self.normalized();
        if cx.try_global::<Self>() != Some(&style) {
            cx.set_global(style);
            cx.refresh_windows();
        }
    }

    /// Apply appearance changes without rolling back a vertical position that
    /// playback updated after the settings window opened its snapshot.
    pub(crate) fn apply_appearance(self, cx: &mut App) {
        let mut style = self;
        style.vertical_offset_fraction = Self::get(cx).vertical_offset_fraction;
        style.apply(cx);
    }
}

mod track_preferences;
pub(crate) use track_preferences::PlaybackTrackPreferences;

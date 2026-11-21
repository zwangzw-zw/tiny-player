//! Pure subtitle presentation preferences shared by settings and playback.
use serde::{Deserialize, Serialize};

/// Clamp for persisted vertical position, expressed as a fraction of video height.
pub(crate) const MAX_VERTICAL_OFFSET_FRACTION: f32 = 1.0;
const MIN_PERSISTED_OFFSET_FRACTION: f32 = 0.0005;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubtitleTextColor {
    #[default]
    White,
    WarmWhite,
    Yellow,
    LightGray,
}

impl SubtitleTextColor {
    pub(crate) const ALL: [Self; 4] = [Self::White, Self::WarmWhite, Self::Yellow, Self::LightGray];

    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::White => "subtitle-color-white",
            Self::WarmWhite => "subtitle-color-warm-white",
            Self::Yellow => "subtitle-color-yellow",
            Self::LightGray => "subtitle-color-light-gray",
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::White => "白色",
            Self::WarmWhite => "暖白色",
            Self::Yellow => "黄色",
            Self::LightGray => "浅灰色",
        }
    }

    pub(crate) fn rgb_hex(self) -> u32 {
        match self {
            Self::White => 0xffffff,
            Self::WarmWhite => 0xf5f0e1,
            Self::Yellow => 0xffe066,
            Self::LightGray => 0xd0d0d0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubtitleOutline {
    #[default]
    Off,
    Thin,
    Medium,
    Thick,
}

impl SubtitleOutline {
    pub(crate) const ALL: [Self; 4] = [Self::Off, Self::Thin, Self::Medium, Self::Thick];

    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::Off => "subtitle-outline-off",
            Self::Thin => "subtitle-outline-thin",
            Self::Medium => "subtitle-outline-medium",
            Self::Thick => "subtitle-outline-thick",
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Off => "关闭",
            Self::Thin => "细",
            Self::Medium => "中",
            Self::Thick => "粗",
        }
    }

    /// Outline radius in window pixels at 1x subtitle scale.
    pub(crate) fn width_px(self) -> f32 {
        match self {
            Self::Off => 0.0,
            Self::Thin => 1.0,
            Self::Medium => 2.0,
            Self::Thick => 3.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubtitleBackground {
    #[default]
    Off,
    Subtle,
    Strong,
}

impl SubtitleBackground {
    pub(crate) const ALL: [Self; 3] = [Self::Off, Self::Subtle, Self::Strong];

    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::Off => "subtitle-background-off",
            Self::Subtle => "subtitle-background-subtle",
            Self::Strong => "subtitle-background-strong",
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Off => "关闭",
            Self::Subtle => "半透明",
            Self::Strong => "深色",
        }
    }

    /// RGBA color for the text background box, if enabled.
    pub(crate) fn rgba_hex(self) -> Option<u32> {
        match self {
            Self::Off => None,
            Self::Subtle => Some(0x00000073),
            Self::Strong => Some(0x000000bf),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SubtitleStyle {
    #[serde(default)]
    pub(crate) text_color: SubtitleTextColor,
    #[serde(default)]
    pub(crate) outline: SubtitleOutline,
    #[serde(default)]
    pub(crate) background: SubtitleBackground,
    /// Fraction of the video height; positive moves subtitles up. `None` keeps
    /// the player's default bottom alignment, including bitmap content lifting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) vertical_offset_fraction: Option<f32>,
}

impl SubtitleStyle {
    /// Clamp persisted values. A position that returns to zero is stored as
    /// `None` so the default alignment (not an explicit zero offset) is used.
    pub(crate) fn normalized(mut self) -> Self {
        self.vertical_offset_fraction = self.vertical_offset_fraction.and_then(|value| {
            if !value.is_finite() {
                return None;
            }
            let value = value.clamp(-MAX_VERTICAL_OFFSET_FRACTION, MAX_VERTICAL_OFFSET_FRACTION);
            (value.abs() >= MIN_PERSISTED_OFFSET_FRACTION).then_some(value)
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_preserve_the_existing_subtitle_appearance() {
        let style = SubtitleStyle::default();
        assert_eq!(style.text_color, SubtitleTextColor::White);
        assert_eq!(style.outline, SubtitleOutline::Off);
        assert_eq!(style.background, SubtitleBackground::Off);
        assert_eq!(style.vertical_offset_fraction, None);
        assert_eq!(style.text_color.rgb_hex(), 0xffffff);
        assert_eq!(style.outline.width_px(), 0.0);
        assert_eq!(style.background.rgba_hex(), None);
    }

    #[test]
    fn normalization_discards_non_finite_and_zero_offsets() {
        for value in [Some(f32::NAN), Some(f32::INFINITY), Some(0.0), Some(0.0001)] {
            let style = SubtitleStyle {
                vertical_offset_fraction: value,
                ..Default::default()
            }
            .normalized();
            assert_eq!(style.vertical_offset_fraction, None, "{value:?}");
        }
        let high = SubtitleStyle {
            vertical_offset_fraction: Some(9.0),
            ..Default::default()
        }
        .normalized();
        assert_eq!(
            high.vertical_offset_fraction,
            Some(MAX_VERTICAL_OFFSET_FRACTION)
        );
        let low = SubtitleStyle {
            vertical_offset_fraction: Some(-9.0),
            ..Default::default()
        }
        .normalized();
        assert_eq!(
            low.vertical_offset_fraction,
            Some(-MAX_VERTICAL_OFFSET_FRACTION)
        );
    }

    #[test]
    fn older_settings_without_style_deserialize_to_defaults() {
        let style: SubtitleStyle = serde_json::from_str("{}").unwrap();
        assert_eq!(style, SubtitleStyle::default());
    }

    #[test]
    fn presets_map_to_expected_colors() {
        assert_eq!(SubtitleTextColor::WarmWhite.rgb_hex(), 0xf5f0e1);
        assert_eq!(SubtitleTextColor::Yellow.rgb_hex(), 0xffe066);
        assert_eq!(SubtitleTextColor::LightGray.rgb_hex(), 0xd0d0d0);
        assert_eq!(SubtitleBackground::Subtle.rgba_hex(), Some(0x00000073));
        assert_eq!(SubtitleBackground::Strong.rgba_hex(), Some(0x000000bf));
        assert_eq!(SubtitleOutline::Thin.width_px(), 1.0);
        assert_eq!(SubtitleOutline::Thick.width_px(), 3.0);
    }
}

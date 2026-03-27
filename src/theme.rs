use fs_render::{Color, FsTheme};
use iced::Theme;

/// iced-backed theme — bridges `FsTheme` to `iced::Theme`.
///
/// On construction the `iced::Theme` is stored.  All `FsTheme` queries derive
/// their values from the underlying iced palette so the two stay in sync.
#[derive(Debug, Clone)]
pub struct IcedTheme {
    inner: Theme,
}

impl IcedTheme {
    /// Wrap an existing `iced::Theme`.
    pub fn new(inner: Theme) -> Self {
        Self { inner }
    }

    /// The `FreeSynergy` Default theme: dark background, cyan primary.
    pub fn fs_default() -> Self {
        Self::new(Theme::Dark)
    }

    /// Access the underlying `iced::Theme` for use inside iced views.
    pub fn iced_theme(&self) -> &Theme {
        &self.inner
    }
}

impl Default for IcedTheme {
    fn default() -> Self {
        Self::fs_default()
    }
}

impl FsTheme for IcedTheme {
    fn name(&self) -> &str {
        match &self.inner {
            Theme::Light => "iced-light",
            Theme::Dark => "FreeSynergy Default (iced-dark)",
            Theme::Dracula => "iced-dracula",
            Theme::Nord => "iced-nord",
            Theme::SolarizedLight => "iced-solarized-light",
            Theme::SolarizedDark => "iced-solarized-dark",
            Theme::GruvboxLight => "iced-gruvbox-light",
            Theme::GruvboxDark => "iced-gruvbox-dark",
            Theme::CatppuccinLatte => "iced-catppuccin-latte",
            Theme::CatppuccinFrappe => "iced-catppuccin-frappe",
            Theme::CatppuccinMacchiato => "iced-catppuccin-macchiato",
            Theme::CatppuccinMocha => "iced-catppuccin-mocha",
            Theme::TokyoNight => "iced-tokyo-night",
            Theme::TokyoNightStorm => "iced-tokyo-night-storm",
            Theme::TokyoNightLight => "iced-tokyo-night-light",
            Theme::KanagawaWave => "iced-kanagawa-wave",
            Theme::KanagawaDragon => "iced-kanagawa-dragon",
            Theme::KanagawaLotus => "iced-kanagawa-lotus",
            Theme::Moonfly => "iced-moonfly",
            Theme::Nightfly => "iced-nightfly",
            Theme::Oxocarbon => "iced-oxocarbon",
            Theme::Ferra => "iced-ferra",
            Theme::Custom(_) => "iced-custom",
        }
    }

    /// `FreeSynergy` Default primary: `#00DEDE` cyan.
    fn primary_color(&self) -> Color {
        Color::rgb(0.0, 0.87, 0.87)
    }

    fn background_color(&self) -> Color {
        let palette = self.inner.palette();
        let bg = palette.background;
        Color::rgba(bg.r, bg.g, bg.b, 1.0)
    }

    fn text_color(&self) -> Color {
        Color::WHITE
    }

    fn accent_color(&self) -> Color {
        Color::rgb(0.0, 0.70, 0.70)
    }

    fn border_radius(&self) -> f32 {
        6.0
    }

    fn font_size_base(&self) -> f32 {
        14.0
    }
}

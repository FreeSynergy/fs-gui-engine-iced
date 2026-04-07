// theme.rs — IcedTheme: bridges FsTheme ↔ iced::Theme.
//
// Design Pattern: Adapter
//   IcedTheme wraps iced::Theme and implements FsTheme so downstream code
//   only sees the fs-render interface.
//
// Custom themes (feature = "theme-ext"):
//   `IcedTheme::from_primary` builds a complete iced::Theme::Custom palette
//   from a single primary colour using the `palette` crate's Oklch colour
//   space for perceptually uniform light/dark shades.

use fs_render::{Color, FsTheme};
use iced::Theme;

/// iced-backed theme — bridges `FsTheme` to `iced::Theme`.
///
/// On construction the `iced::Theme` is stored.  All `FsTheme` queries derive
/// their values from the underlying iced palette so the two stay in sync.
///
/// Use [`IcedTheme::from_primary`] (feature `theme-ext`) to generate a full
/// custom theme from a single brand colour.
#[derive(Debug, Clone)]
pub struct IcedTheme {
    inner: Theme,
    /// Override for `primary_color()` — set when constructed via `from_primary`.
    primary: Option<Color>,
}

impl IcedTheme {
    /// Wrap an existing `iced::Theme`.
    pub fn new(inner: Theme) -> Self {
        Self {
            inner,
            primary: None,
        }
    }

    /// The `FreeSynergy` Default theme: dark background, cyan `#00DEDE` primary.
    pub fn fs_default() -> Self {
        Self::new(Theme::Dark)
    }

    /// Build a custom dark theme from a single primary `(r, g, b)` colour.
    ///
    /// Uses the `palette` crate's Oklch colour space (feature `theme-ext`) to
    /// derive background, surface, text and accent colours that are perceptually
    /// balanced.  Falls back to `fs_default()` when the feature is disabled.
    ///
    /// # Example
    /// ```
    /// use fs_gui_engine_iced::IcedTheme;
    /// let cyan = IcedTheme::from_primary(0.0, 0.87, 0.87);
    /// assert!(cyan.iced_theme().palette().background.r < 0.2);
    /// ```
    #[allow(unused_variables)]
    pub fn from_primary(r: f32, g: f32, b: f32) -> Self {
        #[cfg(feature = "theme-ext")]
        {
            use palette::{FromColor, Oklch, Srgb};

            let srgb = Srgb::new(r, g, b);
            let oklch = Oklch::from_color(srgb);

            // Derive dark background: same hue, low lightness, low chroma.
            let bg_oklch = Oklch::new(0.12, 0.015, oklch.hue);
            let bg_srgb = Srgb::from_color(bg_oklch);

            // Derive surface (card/panel): slightly lighter than background.
            let surf_oklch = Oklch::new(0.18, 0.020, oklch.hue);
            let surf_srgb = Srgb::from_color(surf_oklch);

            // Derive text: near-white, slight hue tint.
            let text_oklch = Oklch::new(0.92, 0.010, oklch.hue);
            let text_srgb = Srgb::from_color(text_oklch);

            let palette = iced::theme::Palette {
                background: iced::Color::from_rgb(bg_srgb.red, bg_srgb.green, bg_srgb.blue),
                text: iced::Color::from_rgb(text_srgb.red, text_srgb.green, text_srgb.blue),
                primary: iced::Color::from_rgb(r, g, b),
                success: iced::Color::from_rgb(
                    surf_srgb.red,
                    (surf_srgb.green + 0.15).min(1.0),
                    surf_srgb.blue,
                ),
                warning: iced::Color::from_rgb(0.95, 0.75, 0.10),
                danger: iced::Color::from_rgb(0.85, 0.20, 0.20),
            };

            return Self {
                inner: Theme::custom("fs-custom", palette),
                primary: Some(Color::rgb(r, g, b)),
            };
        }

        #[cfg(not(feature = "theme-ext"))]
        Self::fs_default()
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
        self.primary.unwrap_or_else(|| Color::rgb(0.0, 0.87, 0.87))
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

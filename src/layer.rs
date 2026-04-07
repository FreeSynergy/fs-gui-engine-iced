// layer.rs — Wayland Layer Shell abstractions for FreeSynergy desktop.
//
// Design Pattern: Facade + Builder
//   Wraps iced_layershell's Settings/LayerShellSettings into simple,
//   semantic constructors that fs-desktop uses to create panels, docks
//   and overlay windows without importing iced_layershell directly.
//
// Feature gate: only compiled when feature = "wayland" is active.
// fs-desktop enables this via `fs-gui-engine-iced = { features = ["desktop"] }`.
//
// Layer windows are positioned by the Wayland compositor — they sit outside
// the normal window stack and are used for panels, docks, corner-menus, and
// notification overlays.

#![cfg(feature = "wayland")]

pub use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
pub use iced_layershell::settings::{LayerShellSettings, Settings as LayerSettings};

// ── Edge anchor ───────────────────────────────────────────────────────────────

/// Which screen edge a layer window is anchored to.
///
/// Used by [`LayerWindowConfig`] to build the correct `Anchor` bitflags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenEdge {
    Top,
    Bottom,
    Left,
    Right,
}

impl ScreenEdge {
    /// Convert to the `Anchor` bitflag that stretches the panel along the edge.
    fn anchor(self) -> Anchor {
        match self {
            Self::Top => Anchor::Top | Anchor::Left | Anchor::Right,
            Self::Bottom => Anchor::Bottom | Anchor::Left | Anchor::Right,
            Self::Left => Anchor::Left | Anchor::Top | Anchor::Bottom,
            Self::Right => Anchor::Right | Anchor::Top | Anchor::Bottom,
        }
    }
}

// ── Corner ────────────────────────────────────────────────────────────────────

/// Which screen corner a layer window sits in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl ScreenCorner {
    fn anchor(self) -> Anchor {
        match self {
            Self::TopLeft => Anchor::Top | Anchor::Left,
            Self::TopRight => Anchor::Top | Anchor::Right,
            Self::BottomLeft => Anchor::Bottom | Anchor::Left,
            Self::BottomRight => Anchor::Bottom | Anchor::Right,
        }
    }
}

// ── LayerWindowConfig ─────────────────────────────────────────────────────────

/// High-level descriptor for a Wayland layer-shell window.
///
/// Convert to [`LayerShellSettings`] via [`LayerWindowConfig::into_settings`]
/// before passing to `iced_layershell::application`.
///
/// # Example
/// ```no_run
/// use fs_gui_engine_iced::layer::{LayerWindowConfig, ScreenEdge};
///
/// let settings = LayerWindowConfig::panel(ScreenEdge::Bottom, 48)
///     .into_shell_settings();
/// ```
#[derive(Debug, Clone)]
pub struct LayerWindowConfig {
    anchor: Anchor,
    layer: Layer,
    /// Height (horizontal panel) or width (vertical panel) in logical pixels.
    /// `None` → compositor decides.
    size: Option<(u32, u32)>,
    /// Pixels reserved at the edge (positive) or free-floating (−1).
    exclusive_zone: i32,
    keyboard: KeyboardInteractivity,
    /// Margin (top, right, bottom, left) in logical pixels.
    margin: (i32, i32, i32, i32),
}

impl LayerWindowConfig {
    // ── Semantic constructors ─────────────────────────────────────────────────

    /// A full-width/height panel anchored to `edge` with `thickness` px.
    ///
    /// Sets `exclusive_zone = thickness` so the compositor reserves space.
    pub fn panel(edge: ScreenEdge, thickness: u32) -> Self {
        let (w, h) = match edge {
            ScreenEdge::Top | ScreenEdge::Bottom => (0, thickness),  // 0 → full width
            ScreenEdge::Left | ScreenEdge::Right => (thickness, 0),  // 0 → full height
        };
        Self {
            anchor: edge.anchor(),
            layer: Layer::Top,
            size: Some((w, h)),
            exclusive_zone: thickness as i32,
            keyboard: KeyboardInteractivity::None,
            margin: (0, 0, 0, 0),
        }
    }

    /// A dock panel: like a panel but on `Background` layer and keyboard-passive.
    pub fn dock(edge: ScreenEdge, thickness: u32) -> Self {
        Self {
            layer: Layer::Bottom,
            ..Self::panel(edge, thickness)
        }
    }

    /// A small overlay in a screen corner (e.g. Corner-Menu indicator).
    ///
    /// `size` is `(width, height)` in logical pixels.
    pub fn corner_overlay(corner: ScreenCorner, size: (u32, u32)) -> Self {
        Self {
            anchor: corner.anchor(),
            layer: Layer::Overlay,
            size: Some(size),
            exclusive_zone: 0,
            keyboard: KeyboardInteractivity::None,
            margin: (0, 0, 0, 0),
        }
    }

    /// A full-screen overlay (e.g. app launcher, lock screen).
    pub fn fullscreen_overlay() -> Self {
        Self {
            anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
            layer: Layer::Overlay,
            size: None,
            exclusive_zone: -1,
            keyboard: KeyboardInteractivity::Exclusive,
            margin: (0, 0, 0, 0),
        }
    }

    // ── Builder methods ───────────────────────────────────────────────────────

    /// Set keyboard interactivity.
    pub fn keyboard(mut self, kb: KeyboardInteractivity) -> Self {
        self.keyboard = kb;
        self
    }

    /// Set margin `(top, right, bottom, left)` in logical pixels.
    pub fn margin(mut self, top: i32, right: i32, bottom: i32, left: i32) -> Self {
        self.margin = (top, right, bottom, left);
        self
    }

    /// Override the compositor layer.
    pub fn layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    // ── Conversion ────────────────────────────────────────────────────────────

    /// Convert to `iced_layershell`'s native settings struct.
    pub fn into_shell_settings(self) -> LayerShellSettings {
        LayerShellSettings {
            anchor: self.anchor,
            layer: self.layer,
            exclusive_zone: self.exclusive_zone,
            size: self.size,
            margin: self.margin,
            keyboard_interactivity: self.keyboard,
            ..LayerShellSettings::default()
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_bottom_has_exclusive_zone() {
        let cfg = LayerWindowConfig::panel(ScreenEdge::Bottom, 48);
        let settings = cfg.into_shell_settings();
        assert_eq!(settings.exclusive_zone, 48);
        assert_eq!(settings.layer, Layer::Top);
    }

    #[test]
    fn dock_uses_background_layer() {
        let cfg = LayerWindowConfig::dock(ScreenEdge::Bottom, 64);
        let settings = cfg.into_shell_settings();
        assert_eq!(settings.layer, Layer::Bottom);
    }

    #[test]
    fn corner_overlay_no_exclusive_zone() {
        let cfg = LayerWindowConfig::corner_overlay(ScreenCorner::TopLeft, (40, 40));
        let settings = cfg.into_shell_settings();
        assert_eq!(settings.exclusive_zone, 0);
        assert_eq!(settings.size, Some((40, 40)));
    }

    #[test]
    fn fullscreen_overlay_keyboard_exclusive() {
        let cfg = LayerWindowConfig::fullscreen_overlay();
        let settings = cfg.into_shell_settings();
        assert_eq!(settings.keyboard_interactivity, KeyboardInteractivity::Exclusive);
        assert_eq!(settings.exclusive_zone, -1);
    }
}

#![deny(clippy::all, clippy::pedantic)]
#![deny(warnings)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
// fs-gui-engine-iced — iced render engine for FreeSynergy.
//
// Implements all `fs-render` traits:
//   - `RenderEngine`  — IcedEngine
//   - `FsWidget`      — IcedWidget
//   - `FsWindow`      — IcedWindow
//   - `FsTheme`       — IcedTheme
//
// Application code imports `fs-render` only — never this crate directly.
// Engine selection happens via Cargo feature flags in fs-desktop / fs-browser.
//
// # Optional features
//   - `wayland`     — layer.rs:  Wayland Layer Shell (panels, docks, overlays)
//   - `portals`     — portal.rs: XDG Portals (file picker, notifications)
//   - `theme-ext`   — theme.rs:  Oklch custom theme generation via `palette`
//   - `icon-lookup` — layout.rs: freedesktop icon-theme lookup
//   - `desktop`     — all four features combined
//
// # Re-exported iced
// `pub use iced` lets downstream crates (fs-browser, fs-desktop shell)
// use iced widgets without adding `iced` to their own Cargo.toml.

pub mod capability;
pub mod engine;
pub mod keys;
pub mod layout;
pub mod layer;
pub mod mvu;
pub mod navigation;
pub mod portal;
pub mod theme;
pub mod widget;
pub mod window;

pub use capability::{IcedCapability, CAPABILITY_ID};
pub use engine::IcedEngine;
pub use layout::{load_layout_or_default, render_element, IcedLayoutInterpreter, LayoutMessage};
pub use navigation::{
    render_corner_menu, render_side_menu, update_corner_menu, update_side_menu, CornerMenuState,
    MenuConfig, NavMessage, SideMenuState,
};
pub use theme::IcedTheme;
pub use widget::IcedWidget;
pub use window::IcedWindow;

// Feature-gated re-exports so downstream crates get clean access.
#[cfg(feature = "wayland")]
pub use layer::{LayerWindowConfig, ScreenCorner, ScreenEdge};
#[cfg(feature = "portals")]
pub use portal::{notify, open_file, open_files, save_file, NotificationLevel};

/// Re-export `iced` so downstream crates need not depend on it directly.
pub use iced;

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use fs_render::{FsTheme, FsWidget, FsWindow, RenderEngine, WindowConfig};

    use crate::{IcedEngine, IcedTheme, IcedWidget, IcedWindow};

    // ── IcedEngine ─────────────────────────────────────────────────────────────

    #[test]
    fn engine_name_is_iced() {
        let e = IcedEngine::new();
        assert_eq!(e.name(), "iced");
    }

    #[test]
    fn engine_creates_window_with_correct_title() {
        let e = IcedEngine::new();
        let cfg = WindowConfig {
            title: "Test Window".into(),
            ..WindowConfig::default()
        };
        let win = e.create_window(cfg);
        assert_eq!(win.title(), "Test Window");
    }

    #[test]
    fn engine_default_theme_is_fs_default() {
        let e = IcedEngine::default();
        assert!(e.current_theme().name().contains("FreeSynergy Default"));
    }

    // ── IcedWindow ─────────────────────────────────────────────────────────────

    #[test]
    fn window_starts_visible() {
        let win = IcedWindow::new("App");
        assert!(win.is_visible());
        assert!(!win.is_minimized());
    }

    #[test]
    fn window_hide_show() {
        let mut win = IcedWindow::new("App");
        win.hide();
        assert!(!win.is_visible());
        win.show();
        assert!(win.is_visible());
    }

    #[test]
    fn window_minimize_restore() {
        let mut win = IcedWindow::new("App");
        win.minimize();
        assert!(win.is_minimized());
        win.restore();
        assert!(!win.is_minimized());
        assert!(win.is_visible());
    }

    #[test]
    fn window_set_title() {
        let mut win = IcedWindow::new("Old");
        win.set_title("New".to_string());
        assert_eq!(win.title(), "New");
    }

    #[test]
    fn window_close_queues_event() {
        use fs_render::{FsEvent, WindowEvent};
        let mut win = IcedWindow::new("App");
        win.close();
        let events = win.drain_events();
        assert!(events.contains(&FsEvent::Window(WindowEvent::CloseRequested)));
    }

    #[test]
    fn window_on_event_queued_and_drained() {
        use fs_render::{FsEvent, WindowEvent};
        let mut win = IcedWindow::new("App");
        win.on_event(FsEvent::Window(WindowEvent::Focused));
        win.on_event(FsEvent::Window(WindowEvent::Unfocused));
        let events = win.drain_events();
        assert_eq!(events.len(), 2);
        let events2 = win.drain_events();
        assert!(events2.is_empty());
    }

    // ── IcedWidget ─────────────────────────────────────────────────────────────

    #[test]
    fn widget_id_and_enabled() {
        let w = IcedWidget::new("btn-save");
        assert_eq!(w.widget_id(), "btn-save");
        assert!(w.is_enabled());
    }

    #[test]
    fn widget_disable_enable() {
        let mut w = IcedWidget::new("btn-delete");
        w.set_enabled(false);
        assert!(!w.is_enabled());
        w.set_enabled(true);
        assert!(w.is_enabled());
    }

    // ── IcedTheme ──────────────────────────────────────────────────────────────

    #[test]
    fn theme_fs_default_is_cyan() {
        let t = IcedTheme::fs_default();
        let c = t.primary_color();
        assert!(c.g > 0.8);
        assert!(c.b > 0.8);
        assert!(c.r < 0.1);
    }

    #[test]
    fn theme_background_from_iced_palette() {
        let t = IcedTheme::fs_default();
        let bg = t.background_color();
        // iced Dark palette background is dark
        assert!(bg.r < 0.3);
        assert!(bg.g < 0.3);
        assert!(bg.b < 0.3);
    }

    #[test]
    fn theme_border_radius_and_font_size() {
        let t = IcedTheme::default();
        assert!((t.border_radius() - 6.0).abs() < f32::EPSILON);
        assert!((t.font_size_base() - 14.0).abs() < f32::EPSILON);
    }

    #[test]
    fn theme_name_contains_free_synergy() {
        let t = IcedTheme::fs_default();
        assert!(t.name().contains("FreeSynergy"));
    }

    // ── set_context ────────────────────────────────────────────────────────────

    #[test]
    fn engine_set_context_updates_locale() {
        use fs_render::{AppContext, RenderEngine};
        let e = IcedEngine::new();
        let ctx = AppContext::new("de", "FreeSynergy Default");
        e.set_context(ctx);
        assert_eq!(e.app_context().locale, "de");
    }

    #[test]
    fn engine_set_context_updates_theme_name() {
        use fs_render::{AppContext, RenderEngine};
        let e = IcedEngine::new();
        e.set_context(AppContext::new("en", "CatppuccinMocha"));
        assert_eq!(e.app_context().theme_name, "CatppuccinMocha");
    }

    // ── IcedCapability ─────────────────────────────────────────────────────────

    #[test]
    fn capability_id_is_correct() {
        use crate::CAPABILITY_ID;
        assert_eq!(CAPABILITY_ID, "render.engine.iced");
    }

    #[test]
    fn capability_descriptor_has_version() {
        use crate::IcedCapability;
        let cap = IcedCapability::descriptor();
        assert!(!cap.version.is_empty());
        assert_eq!(cap.id, "render.engine.iced");
    }
}

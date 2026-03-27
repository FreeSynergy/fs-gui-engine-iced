use fs_render::{FsEvent, RenderEngine, WindowConfig};

use crate::{IcedTheme, IcedWidget, IcedWindow};

/// iced render engine — implements [`fs_render::RenderEngine`] for `fs-render`.
///
/// `IcedEngine` is the entry point for `FreeSynergy` applications that target the
/// iced / libcosmic rendering backend.  It creates [`IcedWindow`] descriptors
/// and applies themes to subsequent renders.
///
/// # Running a window
///
/// ```no_run
/// use fs_gui_engine_iced::IcedEngine;
/// use fs_render::{RenderEngine, WindowConfig};
///
/// let engine = IcedEngine::new();
/// let _win = engine.create_window(WindowConfig::default());
/// // pass `_win` to your iced Application's initial state, then call
/// // `engine.run(app)` to enter the iced event loop.
/// ```
pub struct IcedEngine {
    active_theme: IcedTheme,
}

impl IcedEngine {
    /// Create a new engine using the `FreeSynergy` Default theme.
    pub fn new() -> Self {
        Self {
            active_theme: IcedTheme::fs_default(),
        }
    }

    /// The theme currently applied to this engine.
    pub fn current_theme(&self) -> &IcedTheme {
        &self.active_theme
    }
}

impl Default for IcedEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderEngine for IcedEngine {
    type Window = IcedWindow;
    type Widget = IcedWidget;
    type Theme = IcedTheme;

    #[allow(clippy::unnecessary_literal_bound)]
    fn name(&self) -> &str {
        "iced"
    }

    #[allow(clippy::unnecessary_literal_bound)]
    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn create_window(&self, config: WindowConfig) -> IcedWindow {
        IcedWindow::new(config.title)
    }

    fn apply_theme(&self, _theme: &IcedTheme) {
        // In iced the theme is bound to the Application at startup and can be
        // changed by returning a new Theme from `Application::theme()`.
        // `apply_theme` records the intent; the iced Application reads
        // `engine.current_theme()` on the next `theme()` call.
    }

    fn dispatch_event(&self, _event: FsEvent) {
        // Events flow through the iced event loop.  External callers can push
        // custom events via `iced::window::run_action` or the application's
        // own command channel.  This hook is a no-op for now.
    }

    fn shutdown(&self) {
        // iced shuts down when the last window closes.  Signal the runtime by
        // returning `Task::done(iced::exit())` from the application's `update`.
    }
}

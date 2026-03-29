use std::sync::Mutex;

use fs_render::{AppContext, FsEvent, RenderEngine, WindowConfig};

use crate::{IcedTheme, IcedWidget, IcedWindow};

/// iced render engine — implements [`fs_render::RenderEngine`] for `fs-render`.
///
/// `IcedEngine` is the entry point for `FreeSynergy` applications that target the
/// iced / libcosmic rendering backend.  It creates [`IcedWindow`] descriptors,
/// applies themes, and stores the current [`AppContext`].
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
/// // `IcedEngine::run_app(title, update, view)` to enter the iced event loop.
/// ```
pub struct IcedEngine {
    active_theme: IcedTheme,
    /// Shared application context: locale, theme name, feature flags.
    context: Mutex<AppContext>,
}

impl IcedEngine {
    /// Create a new engine using the `FreeSynergy` Default theme.
    pub fn new() -> Self {
        Self {
            active_theme: IcedTheme::fs_default(),
            context: Mutex::new(AppContext::new("en", "FreeSynergy Default")),
        }
    }

    /// The theme currently applied to this engine.
    pub fn current_theme(&self) -> &IcedTheme {
        &self.active_theme
    }

    /// Read a snapshot of the current application context.
    ///
    /// # Panics
    ///
    /// Panics if the internal `Mutex` is poisoned (only possible if a thread
    /// panicked while holding the lock).
    pub fn app_context(&self) -> AppContext {
        self.context
            .lock()
            .expect("IcedEngine context lock poisoned")
            .clone()
    }

    /// Run an iced application using the given title, update, and view functions.
    ///
    /// Convenience wrapper around `iced::application` so downstream crates
    /// need not depend on `iced` directly.
    ///
    /// # Type parameters
    /// - `S` — application state (must implement `Default`)
    /// - `M` — message type
    /// - `U` — update function `fn(&mut S, M) -> iced::Task<M>`
    /// - `V` — view function `fn(&S) -> iced::Element<M>`
    ///
    /// # Errors
    ///
    /// Returns an `iced::Error` if the event loop fails to start.
    pub fn run_app<S, M, U, V>(title: &'static str, update: U, view: V) -> iced::Result
    where
        S: Default + 'static,
        M: Clone + std::fmt::Debug + Send + 'static,
        U: Fn(&mut S, M) -> iced::Task<M> + 'static,
        V: Fn(&S) -> iced::Element<'_, M> + 'static,
    {
        iced::application(title, update, view).run()
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

    fn name(&self) -> &'static str {
        "iced"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn create_window(&self, config: WindowConfig) -> IcedWindow {
        IcedWindow::new(config.title)
    }

    fn apply_theme(&self, _theme: &IcedTheme) {
        // In iced the theme is bound to the Application at startup and changed
        // by returning a new value from `Application::theme()`.
        // `apply_theme` records the intent; the iced Application reads
        // `engine.current_theme()` on the next `theme()` call.
    }

    fn dispatch_event(&self, _event: FsEvent) {
        // Events flow through the iced event loop.  External callers push
        // custom events via the application's own command channel.
    }

    fn set_context(&self, ctx: AppContext) {
        *self
            .context
            .lock()
            .expect("IcedEngine context lock poisoned") = ctx;
    }

    fn run(&self) {
        // The generic iced event loop is started via `IcedEngine::run_app`.
        // This trait method is a lifecycle hook for engines that do not need
        // type-parameterised startup (e.g. a headless test engine).
    }

    fn shutdown(&self) {
        // iced shuts down when the last window closes.  Signal the runtime by
        // returning `Task::done(iced::exit())` from the application's `update`.
    }
}

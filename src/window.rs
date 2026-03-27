use fs_render::{FsEvent, FsWindow};

/// State of an iced-backed window.
///
/// This struct holds the *descriptor* state.  The actual OS window is managed
/// by the iced runtime; the runtime reads this state to decide what to render.
/// Mutations via `FsWindow` methods are queued as pending events that the iced
/// `update` loop drains on the next tick.
#[derive(Debug)]
pub struct IcedWindow {
    title: String,
    visible: bool,
    minimized: bool,
    pending_events: Vec<FsEvent>,
}

impl IcedWindow {
    /// Create a new window descriptor.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            visible: true,
            minimized: false,
            pending_events: Vec::new(),
        }
    }

    /// Drain all pending `FsEvent`s buffered via `on_event`.
    pub fn drain_events(&mut self) -> Vec<FsEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Whether the window is currently minimized.
    pub fn is_minimized(&self) -> bool {
        self.minimized
    }
}

impl FsWindow for IcedWindow {
    fn title(&self) -> &str {
        &self.title
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn show(&mut self) {
        self.visible = true;
        self.minimized = false;
    }

    fn hide(&mut self) {
        self.visible = false;
    }

    fn minimize(&mut self) {
        self.minimized = true;
        self.pending_events
            .push(FsEvent::Window(fs_render::WindowEvent::Minimized));
    }

    fn restore(&mut self) {
        self.minimized = false;
        self.visible = true;
        self.pending_events
            .push(FsEvent::Window(fs_render::WindowEvent::Restored));
    }

    fn close(&mut self) {
        self.visible = false;
        self.pending_events
            .push(FsEvent::Window(fs_render::WindowEvent::CloseRequested));
    }

    fn set_title(&mut self, title: String) {
        self.title = title;
    }

    fn on_event(&mut self, event: FsEvent) {
        self.pending_events.push(event);
    }
}

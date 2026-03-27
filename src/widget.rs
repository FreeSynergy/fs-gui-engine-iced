use fs_render::FsWidget;

/// iced-backed widget handle.
///
/// The iced view function creates concrete iced `Element`s; this struct is the
/// descriptor that `IcedEngine` / application code holds *between* renders.
/// When the view rebuilds it reads the descriptor and produces the iced widget.
#[derive(Debug, Clone)]
pub struct IcedWidget {
    id: String,
    enabled: bool,
}

impl IcedWidget {
    /// Create a new widget descriptor.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            enabled: true,
        }
    }
}

impl FsWidget for IcedWidget {
    fn widget_id(&self) -> &str {
        &self.id
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

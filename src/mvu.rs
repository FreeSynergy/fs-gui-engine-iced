// mvu.rs — Elm-style Model-View-Update pattern for fs-gui-engine-iced.
//
// MVU separates three concerns:
//   - Model  (M / S) — immutable application state snapshot
//   - View   (V)     — pure function: &State → iced::Element
//   - Update (U)     — pure function: &mut State × Message → iced::Task<Message>
//
// Consumer crates define their own `State` and `Message` types and wire them
// together with `MvuApp::new`, then launch the event loop via `MvuApp::run`.

/// Message type constraint for MVU applications.
///
/// Messages are values that describe what happened (e.g. a button was clicked).
/// The `update` function maps a `(state, message)` pair to the next state and
/// an optional `Task` (side-effect).
///
/// This is a marker trait alias: it collects all bounds iced needs on the
/// message type.
pub trait FsMessage: Clone + std::fmt::Debug + Send + 'static {}

impl<M> FsMessage for M where M: Clone + std::fmt::Debug + Send + 'static {}

/// Bundled MVU application.
///
/// Holds the title string and the two pure functions that define the
/// application's behaviour.  Call [`MvuApp::run`] to start the iced event loop.
///
/// # Example
///
/// ```no_run
/// use fs_gui_engine_iced::mvu::MvuApp;
/// use iced::{Element, Task, widget::text};
///
/// #[derive(Default)]
/// struct Counter { value: i32 }
///
/// #[derive(Debug, Clone)]
/// enum Msg { Increment, Decrement }
///
/// fn update(state: &mut Counter, msg: Msg) -> Task<Msg> {
///     match msg {
///         Msg::Increment => state.value += 1,
///         Msg::Decrement => state.value -= 1,
///     }
///     Task::none()
/// }
///
/// fn view(state: &Counter) -> Element<Msg> {
///     text(state.value.to_string()).into()
/// }
///
/// fn main() {
///     MvuApp::new("Counter", update, view).run().unwrap();
/// }
/// ```
pub struct MvuApp<S, M, U, V>
where
    S: Default + 'static,
    M: FsMessage,
    U: Fn(&mut S, M) -> iced::Task<M> + 'static,
    V: for<'a> Fn(&'a S) -> iced::Element<'a, M> + 'static,
{
    title: &'static str,
    update: U,
    view: V,
    _state: std::marker::PhantomData<(S, M)>,
}

impl<S, M, U, V> MvuApp<S, M, U, V>
where
    S: Default + 'static,
    M: FsMessage,
    U: Fn(&mut S, M) -> iced::Task<M> + 'static,
    V: for<'a> Fn(&'a S) -> iced::Element<'a, M> + 'static,
{
    /// Create a new MVU application.
    ///
    /// - `title`  — window title bar text.
    /// - `update` — pure state-transition function.
    /// - `view`   — pure rendering function.
    pub fn new(title: &'static str, update: U, view: V) -> Self {
        Self {
            title,
            update,
            view,
            _state: std::marker::PhantomData,
        }
    }

    /// Start the iced event loop.
    ///
    /// Blocks until the last window is closed.
    ///
    /// # Errors
    ///
    /// Returns an `iced::Error` if the event loop fails to start.
    pub fn run(self) -> iced::Result {
        iced::application(S::default, self.update, self.view)
            .title(self.title)
            .run()
    }
}

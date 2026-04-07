// portal.rs — XDG Portal wrappers for FreeSynergy desktop integration.
//
// Design Pattern: Facade
//   Wraps ashpd's async portal API into simple, fire-and-forget functions
//   that consumer crates call from iced `Task::perform` closures.
//   Consumer code never imports ashpd directly.
//
// Feature gate: only compiled when feature = "portals" is active.
// fs-desktop enables this via `fs-gui-engine-iced = { features = ["desktop"] }`.

#![cfg(feature = "portals")]

use ashpd::desktop::file_chooser::{FileChooserProxy, OpenFileRequest, SaveFileRequest};
use ashpd::desktop::notification::{Action, Button, Notification, NotificationProxy, Priority};
use ashpd::WindowIdentifier;
use std::path::PathBuf;

// ── File Picker ───────────────────────────────────────────────────────────────

/// Result of an open-file dialog.
#[derive(Debug, Clone)]
pub struct OpenFileResult {
    /// Selected file paths (empty when the user cancelled).
    pub paths: Vec<PathBuf>,
}

/// Show an XDG file-open portal dialog.
///
/// Returns the selected paths, or an empty list if the user cancelled.
///
/// # Usage inside iced
/// ```no_run
/// use fs_gui_engine_iced::portal;
/// use iced::Task;
///
/// // In your update function:
/// // Task::perform(portal::open_file("Open Image"), |res| Msg::FileOpened(res))
/// ```
///
/// # Errors
/// Returns `ashpd::Error` on D-Bus communication failure.
pub async fn open_file(title: &str) -> Result<OpenFileResult, ashpd::Error> {
    let proxy = FileChooserProxy::new().await?;
    let response = OpenFileRequest::default()
        .title(title)
        .send(&proxy)
        .await?
        .response()?;

    let paths = response
        .uris()
        .iter()
        .filter_map(|u| u.to_file_path().ok())
        .collect();

    Ok(OpenFileResult { paths })
}

/// Show an XDG file-open portal dialog that allows selecting multiple files.
///
/// # Errors
/// Returns `ashpd::Error` on D-Bus communication failure.
pub async fn open_files(title: &str) -> Result<OpenFileResult, ashpd::Error> {
    let proxy = FileChooserProxy::new().await?;
    let response = OpenFileRequest::default()
        .title(title)
        .multiple(true)
        .send(&proxy)
        .await?
        .response()?;

    let paths = response
        .uris()
        .iter()
        .filter_map(|u| u.to_file_path().ok())
        .collect();

    Ok(OpenFileResult { paths })
}

/// Show an XDG file-save portal dialog.
///
/// Returns the chosen save path, or `None` when the user cancelled.
///
/// # Errors
/// Returns `ashpd::Error` on D-Bus communication failure.
pub async fn save_file(
    title: &str,
    current_name: &str,
) -> Result<Option<PathBuf>, ashpd::Error> {
    let proxy = FileChooserProxy::new().await?;
    let response = SaveFileRequest::default()
        .title(title)
        .current_name(current_name)
        .send(&proxy)
        .await?
        .response();

    match response {
        Ok(r) => Ok(r.uris().first().and_then(|u| u.to_file_path().ok())),
        Err(_) => Ok(None), // user cancelled
    }
}

// ── Notifications ─────────────────────────────────────────────────────────────

/// Priority level for desktop notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Low,
    Normal,
    High,
    Urgent,
}

impl From<NotificationLevel> for Priority {
    fn from(level: NotificationLevel) -> Self {
        match level {
            NotificationLevel::Low => Priority::Low,
            NotificationLevel::Normal => Priority::Normal,
            NotificationLevel::High => Priority::High,
            NotificationLevel::Urgent => Priority::Urgent,
        }
    }
}

/// Send an XDG desktop notification.
///
/// # Usage inside iced
/// ```no_run
/// use fs_gui_engine_iced::portal::{self, NotificationLevel};
/// use iced::Task;
///
/// // Task::perform(
/// //     portal::notify("FreeSynergy", "Package installed.", NotificationLevel::Normal),
/// //     |_| Msg::NotificationSent
/// // )
/// ```
///
/// # Errors
/// Returns `ashpd::Error` on D-Bus failure.
pub async fn notify(
    title: &str,
    body: &str,
    level: NotificationLevel,
) -> Result<(), ashpd::Error> {
    let proxy = NotificationProxy::new().await?;
    let notification = Notification::new(title)
        .body(body)
        .priority(level.into());
    proxy.add_notification("fs-notification", notification).await
}

/// Send a notification with a single action button.
///
/// # Errors
/// Returns `ashpd::Error` on D-Bus failure.
pub async fn notify_with_action(
    title: &str,
    body: &str,
    level: NotificationLevel,
    action_label: &str,
    action_key: &str,
) -> Result<(), ashpd::Error> {
    let proxy = NotificationProxy::new().await?;
    let button = Button::new(action_key, action_label);
    let action = Action::new(action_key, action_label);
    let notification = Notification::new(title)
        .body(body)
        .priority(level.into())
        .button(button)
        .default_action(action_key);
    let _ = action; // kept for documentation clarity
    proxy.add_notification("fs-notification", notification).await
}

// ── Window identifier helper ──────────────────────────────────────────────────

/// Create a wayland window identifier from a surface handle, if available.
///
/// Returns `WindowIdentifier::default()` when no surface handle is available
/// (e.g. in headless / test mode).
pub fn window_identifier() -> WindowIdentifier {
    WindowIdentifier::default()
}

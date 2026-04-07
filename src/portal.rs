// portal.rs — XDG Portal wrappers for FreeSynergy desktop integration.
//
// Design Pattern: Facade
//   Wraps ashpd's async portal API into simple functions that consumer crates
//   call from iced `Task::perform` closures.
//   Consumer code never imports ashpd directly.
//
// Feature gate: only compiled when feature = "portals" is active.
// fs-desktop enables this via `fs-gui-engine-iced = { features = ["desktop"] }`.

#![cfg(feature = "portals")]

use ashpd::desktop::file_chooser::{FileChooserProxy, OpenFileOptions, SaveFileOptions};
use ashpd::desktop::notification::{Notification, NotificationProxy, Priority};
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
    let request = proxy
        .open_file(None, title, OpenFileOptions::default())
        .await?;
    let response = request.response();

    let paths = match response {
        Ok(r) => r
            .uris()
            .iter()
            .filter_map(|u| {
                let s = u.as_str();
                // Strip "file://" prefix to get a local path.
                s.strip_prefix("file://").map(PathBuf::from)
            })
            .collect(),
        Err(_) => vec![], // user cancelled
    };

    Ok(OpenFileResult { paths })
}

/// Show an XDG file-open portal dialog that allows selecting multiple files.
///
/// # Errors
/// Returns `ashpd::Error` on D-Bus communication failure.
pub async fn open_files(title: &str) -> Result<OpenFileResult, ashpd::Error> {
    let proxy = FileChooserProxy::new().await?;
    let opts = OpenFileOptions::default().set_multiple(true);
    let request = proxy.open_file(None, title, opts).await?;
    let response = request.response();

    let paths = match response {
        Ok(r) => r
            .uris()
            .iter()
            .filter_map(|u| u.as_str().strip_prefix("file://").map(PathBuf::from))
            .collect(),
        Err(_) => vec![],
    };

    Ok(OpenFileResult { paths })
}

/// Show an XDG file-save portal dialog.
///
/// Returns the chosen save path, or `None` when the user cancelled.
///
/// # Errors
/// Returns `ashpd::Error` on D-Bus communication failure.
pub async fn save_file(title: &str, current_name: &str) -> Result<Option<PathBuf>, ashpd::Error> {
    let proxy = FileChooserProxy::new().await?;
    let opts = SaveFileOptions::default().set_current_name(current_name);
    let request = proxy.save_file(None, title, opts).await?;
    let response = request.response();

    let path = match response {
        Ok(r) => r
            .uris()
            .first()
            .and_then(|u| u.as_str().strip_prefix("file://").map(PathBuf::from)),
        Err(_) => None, // user cancelled
    };

    Ok(path)
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
pub async fn notify(title: &str, body: &str, level: NotificationLevel) -> Result<(), ashpd::Error> {
    let proxy = NotificationProxy::new().await?;
    let notification = Notification::new(title)
        .body(body)
        .priority(Some(Priority::from(level)));
    proxy
        .add_notification("fs-notification", notification)
        .await
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
    use ashpd::desktop::notification::Button;
    let proxy = NotificationProxy::new().await?;
    let button = Button::new(action_label, action_key);
    let notification = Notification::new(title)
        .body(body)
        .priority(Some(Priority::from(level)))
        .button(button)
        .default_action(action_key);
    proxy
        .add_notification("fs-notification", notification)
        .await
}

// keys.rs — FTL key name constants for fs-gui-engine-iced.
//
// All user-visible strings in this crate are translated via fs-i18n.
// The matching .ftl file lives at:
//   fs-i18n/locales/{lang}/gui-engine-iced.ftl
//
// Use these constants wherever a localised string is needed:
//   fs_i18n::t(keys::CAPABILITY_NAME).to_string()

// ── Engine ────────────────────────────────────────────────────────────────────

/// Display name of this engine (localised).
pub const ENGINE_DISPLAY_NAME: &str = "gui-iced-engine-display-name";

// ── Capability ────────────────────────────────────────────────────────────────

/// Localised name shown to users when listing available render engines.
pub const CAPABILITY_NAME: &str = "gui-iced-capability-name";

// ── Errors ────────────────────────────────────────────────────────────────────

/// Internal context mutex was poisoned (thread panicked while holding it).
pub const ERROR_CONTEXT_LOCK: &str = "gui-iced-error-context-lock";

/// The iced event loop failed to start.
pub const ERROR_RUN_FAILED: &str = "gui-iced-error-run-failed";

/// A window could not be created (variable: `title`).
pub const ERROR_WINDOW_CREATE: &str = "gui-iced-error-window-create";

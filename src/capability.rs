// capability.rs — fs-registry capability descriptor for the iced render engine.
//
// Every adapter registers a capability string so the shell and other services
// can discover it at runtime without hard-coding crate names.
//
// Capability ID: "render.engine.iced"
//
// Format: "<domain>.<type>.<impl>"
//   - domain  = "render"  — rendering subsystem
//   - type    = "engine"  — marks this as a RenderEngine implementation
//   - impl    = "iced"    — concrete backend name

/// Capability identifier registered by this engine in `fs-registry`.
pub const CAPABILITY_ID: &str = "render.engine.iced";

/// Metadata about this engine's registered capability.
///
/// Passed to `fs-registry` during engine startup.  The registry stores it so
/// other services (e.g. the desktop shell, the app launcher) can query which
/// render engines are available without importing this crate.
#[derive(Debug, Clone)]
pub struct IcedCapability {
    /// Stable capability string.
    pub id: &'static str,
    /// Human-readable display name.
    pub display_name: &'static str,
    /// Engine crate version.
    pub version: &'static str,
}

impl IcedCapability {
    /// Returns the capability descriptor for this engine.
    pub fn descriptor() -> Self {
        Self {
            id: CAPABILITY_ID,
            display_name: "FreeSynergy iced Render Engine",
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

impl Default for IcedCapability {
    fn default() -> Self {
        Self::descriptor()
    }
}

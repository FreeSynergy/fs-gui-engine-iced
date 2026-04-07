// navigation.rs — CornerMenu / SideMenu iced renderer.
//
// Design Pattern: Interpreter
//   Reads CornerMenuDescriptor / SideMenuDescriptor (fs-render) and produces
//   iced Element trees.  No iced import leaks into application code — callers
//   only depend on fs-render traits and NavMessage.
//
// Visual approach:
//   Corner menus — items arranged on a quarter-circle arc using a stack of
//                  full-screen containers with corner-relative padding.
//                  Indicator = quarter-disk button at the exact corner.
//   Side menus   — Left/Right: items in a vertical Column next to the indicator.
//                  Top/Bottom: items in a horizontal Row below/above the indicator.
//   Double logo  — CompositeIcon: primary at full size, secondary at half size
//                  overlaid at the bottom-right via a local stack.
//   Magnification — item size computed via exponential falloff (HoverMagnification).
//
// All produced Elements have 'static lifetime — all captured data is Copy or
// owned, so no external borrows escape.

use fs_render::navigation::{
    Corner, CornerMenuDescriptor, MenuItemDescriptor, Side, SideMenuDescriptor,
};
use iced::border::Radius;
use iced::widget::{button, container, scrollable, stack, svg, text, Column, Row, Space, Tooltip};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Theme};

/// Number of items before the scroll fallback activates (side menus only).
const SCROLL_THRESHOLD: usize = 8;

// ── MenuConfig ────────────────────────────────────────────────────────────────

/// Runtime rendering configuration for navigation menus.
#[derive(Debug, Clone)]
pub struct MenuConfig {
    /// Base item height in logical pixels (also used as icon size).
    pub icon_size: f32,
    /// Maximum item height at the cursor position (hover magnification).
    pub max_icon_size: f32,
    /// Magnification falloff spread — higher = wider, softer effect.
    pub spread: f32,
    /// Indicator shape radius in logical pixels.
    pub indicator_radius: f32,
    /// Accent color used for indicators and active items.
    pub accent: Color,
}

impl Default for MenuConfig {
    fn default() -> Self {
        Self {
            icon_size: 32.0,
            max_icon_size: 48.0,
            spread: 2.0,
            indicator_radius: 20.0,
            accent: Color::from_rgb(0.0, 0.9, 0.9), // FreeSynergy cyan
        }
    }
}

// ── CornerMenuState ───────────────────────────────────────────────────────────

/// MVU state for a corner menu widget.
#[derive(Debug, Clone, Default)]
pub struct CornerMenuState {
    /// Whether the item list is currently expanded.
    pub open: bool,
    /// Index of the item the cursor is currently over (for magnification).
    pub hovered_idx: Option<usize>,
}

// ── SideMenuState ─────────────────────────────────────────────────────────────

/// MVU state for a side menu widget.
#[derive(Debug, Clone, Default)]
pub struct SideMenuState {
    /// Whether the item list is currently expanded.
    pub open: bool,
    /// Index of the item the cursor is currently over (for magnification).
    pub hovered_idx: Option<usize>,
}

// ── NavMessage ────────────────────────────────────────────────────────────────

/// Messages emitted by corner and side menu widgets.
#[derive(Debug, Clone)]
pub enum NavMessage {
    /// Toggle a corner menu open/closed.
    CornerMenuToggle(Corner),
    /// Pointer entered an item at the given index.
    CornerMenuItemEntered(Corner, usize),
    /// Pointer left all items in a corner menu.
    CornerMenuItemLeft(Corner),
    /// A corner menu item was activated (leaf action or sub-menu expand).
    CornerMenuAction(Corner, String),
    /// Toggle a side menu open/closed.
    SideMenuToggle(Side),
    /// Pointer entered a side menu item.
    SideMenuItemEntered(Side, usize),
    /// Pointer left all items in a side menu.
    SideMenuItemLeft(Side),
    /// A side menu item was activated.
    SideMenuAction(Side, String),
}

// ── State updaters ────────────────────────────────────────────────────────────

/// Apply a [`NavMessage`] to a [`CornerMenuState`].
///
/// Only processes messages targeted at the given `corner`; all others are
/// silently ignored so callers can fan out a single message to all menus.
pub fn update_corner_menu(state: &mut CornerMenuState, corner: Corner, msg: &NavMessage) {
    match msg {
        NavMessage::CornerMenuToggle(c) if *c == corner => {
            state.open = !state.open;
        }
        NavMessage::CornerMenuItemEntered(c, idx) if *c == corner => {
            state.hovered_idx = Some(*idx);
        }
        NavMessage::CornerMenuItemLeft(c) if *c == corner => {
            state.hovered_idx = None;
        }
        _ => {}
    }
}

/// Apply a [`NavMessage`] to a [`SideMenuState`].
pub fn update_side_menu(state: &mut SideMenuState, side: Side, msg: &NavMessage) {
    match msg {
        NavMessage::SideMenuToggle(s) if *s == side => {
            state.open = !state.open;
        }
        NavMessage::SideMenuItemEntered(s, idx) if *s == side => {
            state.hovered_idx = Some(*idx);
        }
        NavMessage::SideMenuItemLeft(s) if *s == side => {
            state.hovered_idx = None;
        }
        _ => {}
    }
}

// ── render_corner_menu ────────────────────────────────────────────────────────

/// Render a corner menu as a self-contained iced [`Element`].
///
/// Visual design:
/// - The quarter-disk indicator button is always visible at the corner.
/// - When `state.open`, items fan out in a **quarter-circle arc** away from
///   the corner, using a full-screen `stack` of positioned containers.
///   The arc radius is derived from `config.indicator_radius` and `config.icon_size`.
pub fn render_corner_menu(
    descriptor: &dyn CornerMenuDescriptor,
    state: &CornerMenuState,
    config: &MenuConfig,
) -> Element<'static, NavMessage> {
    let corner = descriptor.corner();
    let items = descriptor.items();
    let indicator = corner_indicator_button(corner, config, state.open);

    if !state.open || items.is_empty() {
        return indicator;
    }

    // ── Arc layout via full-screen stack ──────────────────────────────────────
    // The indicator occupies layer 0, pinned to its corner.
    // Each item occupies its own layer, positioned at arc coordinates.

    let indicator_layer = corner_pinned(indicator, corner);
    let mut layers: Vec<Element<'static, NavMessage>> = vec![indicator_layer];

    let n = items.len();
    let arc_radius = config.indicator_radius + config.icon_size * 2.0;

    for (idx, item) in items.iter().enumerate() {
        let sz = magnified_size(config, state.hovered_idx, idx);
        let btn = corner_item_button(item, sz, corner);

        // Distribute angles evenly from ~5° to ~85° (avoids overlapping the indicator).
        let angle_start: f32 = 0.087; // ~5°
        let angle_end: f32 = std::f32::consts::FRAC_PI_2 - 0.087; // ~85°
        #[allow(clippy::cast_precision_loss)]
        let angle = if n <= 1 {
            f32::midpoint(angle_start, angle_end)
        } else {
            angle_start + (idx as f32 / (n - 1) as f32) * (angle_end - angle_start)
        };

        let ax = arc_radius * angle.cos();
        let ay = arc_radius * angle.sin();

        let layer = arc_item_layer(btn, ax, ay, corner);
        layers.push(layer);
    }

    stack(layers)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ── render_side_menu ──────────────────────────────────────────────────────────

/// Render a side menu as a self-contained iced [`Element`].
///
/// Visual design:
/// - The half-disk indicator button is always visible on the edge.
/// - When `state.open`:
///   - **Left / Right** edges: items appear in a **vertical Column** next to
///     the half-disk indicator (Row layout: indicator | items or items | indicator).
///   - **Top / Bottom** edges: items appear in a **horizontal Row** adjacent to
///     the half-disk indicator (Column layout: indicator | items or items | indicator).
pub fn render_side_menu(
    descriptor: &dyn SideMenuDescriptor,
    state: &SideMenuState,
    config: &MenuConfig,
) -> Element<'static, NavMessage> {
    let side = descriptor.side();
    let items = descriptor.items();
    let indicator = side_indicator_button(side, config, state.open);

    if !state.open || items.is_empty() {
        return indicator;
    }

    let items_el = items_for_side(&items, state.hovered_idx, config, side);

    match side {
        // Horizontal edge ↔ indicator points inward → items expand horizontally
        Side::Left => Row::new()
            .push(indicator)
            .push(items_el)
            .spacing(4)
            .into(),
        Side::Right => Row::new()
            .push(items_el)
            .push(indicator)
            .spacing(4)
            .into(),
        // Vertical edge ↕ indicator points inward → items expand vertically
        Side::Top => Column::new()
            .push(indicator)
            .push(items_el)
            .spacing(4)
            .into(),
        Side::Bottom => Column::new()
            .push(items_el)
            .push(indicator)
            .spacing(4)
            .into(),
    }
}

// ── Indicator buttons ─────────────────────────────────────────────────────────

/// A styled button that renders as a quarter-disk anchored at `corner`.
///
/// Border-radius is applied asymmetrically: only the inward corner is rounded,
/// creating the quarter-circle silhouette without requiring the `canvas` feature.
fn corner_indicator_button(
    corner: Corner,
    config: &MenuConfig,
    open: bool,
) -> Element<'static, NavMessage> {
    let r = config.indicator_radius;
    let accent = config.accent;
    let alpha: f32 = if open { 1.0 } else { 0.72 };
    let color = Color {
        a: accent.a * alpha,
        ..accent
    };

    // Build asymmetric radius: only the inward corner is rounded.
    let radius = match corner {
        Corner::TopLeft => Radius {
            top_left: 0.0,
            top_right: 0.0,
            bottom_right: r,
            bottom_left: 0.0,
        },
        Corner::TopRight => Radius {
            top_left: 0.0,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: r,
        },
        Corner::BottomLeft => Radius {
            top_left: 0.0,
            top_right: r,
            bottom_right: 0.0,
            bottom_left: 0.0,
        },
        Corner::BottomRight => Radius {
            top_left: r,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: 0.0,
        },
    };

    button(
        Space::new()
            .width(Length::Fixed(r))
            .height(Length::Fixed(r)),
    )
    .on_press(NavMessage::CornerMenuToggle(corner))
    .style(move |_theme: &Theme, _status| iced::widget::button::Style {
        background: Some(Background::Color(color)),
        border: Border {
            radius,
            ..Border::default()
        },
        text_color: Color::TRANSPARENT,
        ..iced::widget::button::Style::default()
    })
    .padding(0)
    .into()
}

/// A styled button that renders as a half-disk on the given `side`.
///
/// Two adjacent corners are rounded to create the half-circle silhouette.
fn side_indicator_button(
    side: Side,
    config: &MenuConfig,
    open: bool,
) -> Element<'static, NavMessage> {
    let r = config.indicator_radius;
    let accent = config.accent;
    let alpha: f32 = if open { 1.0 } else { 0.72 };
    let color = Color {
        a: accent.a * alpha,
        ..accent
    };

    // Two adjacent corners rounded → half-disk pointing inward.
    let radius = match side {
        Side::Left => Radius {
            top_left: 0.0,
            top_right: r,
            bottom_right: r,
            bottom_left: 0.0,
        },
        Side::Right => Radius {
            top_left: r,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: r,
        },
        Side::Top => Radius {
            top_left: 0.0,
            top_right: 0.0,
            bottom_right: r,
            bottom_left: r,
        },
        Side::Bottom => Radius {
            top_left: r,
            top_right: r,
            bottom_right: 0.0,
            bottom_left: 0.0,
        },
    };

    let (w, h): (f32, f32) = match side {
        Side::Left | Side::Right => (r, r * 2.0),
        Side::Top | Side::Bottom => (r * 2.0, r),
    };

    button(
        Space::new()
            .width(Length::Fixed(w))
            .height(Length::Fixed(h)),
    )
    .on_press(NavMessage::SideMenuToggle(side))
    .style(move |_theme: &Theme, _status| iced::widget::button::Style {
        background: Some(Background::Color(color)),
        border: Border {
            radius,
            ..Border::default()
        },
        text_color: Color::TRANSPARENT,
        ..iced::widget::button::Style::default()
    })
    .padding(0)
    .into()
}

// ── Arc helpers ───────────────────────────────────────────────────────────────

/// Wrap `el` in a full-screen container pinned to `corner` with zero padding.
///
/// Used to position the indicator inside the open arc-menu stack.
fn corner_pinned(el: Element<'static, NavMessage>, corner: Corner) -> Element<'static, NavMessage> {
    let (ax, ay) = match corner {
        Corner::TopLeft => (Alignment::Start, Alignment::Start),
        Corner::TopRight => (Alignment::End, Alignment::Start),
        Corner::BottomLeft => (Alignment::Start, Alignment::End),
        Corner::BottomRight => (Alignment::End, Alignment::End),
    };
    container(el)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(ax)
        .align_y(ay)
        .padding(0)
        .into()
}

/// Wrap `btn` in a full-screen container offset `(ax, ay)` from `corner`.
///
/// Used to place each item at its arc coordinate within the open arc-menu stack.
fn arc_item_layer(
    btn: Element<'static, NavMessage>,
    ax: f32,
    ay: f32,
    corner: Corner,
) -> Element<'static, NavMessage> {
    // Padding pushes the icon away from the corner edge by (ax, ay) pixels.
    let padding = match corner {
        Corner::TopLeft => Padding {
            top: ay,
            left: ax,
            right: 0.0,
            bottom: 0.0,
        },
        Corner::TopRight => Padding {
            top: ay,
            right: ax,
            left: 0.0,
            bottom: 0.0,
        },
        Corner::BottomLeft => Padding {
            bottom: ay,
            left: ax,
            top: 0.0,
            right: 0.0,
        },
        Corner::BottomRight => Padding {
            bottom: ay,
            right: ax,
            top: 0.0,
            left: 0.0,
        },
    };

    let (align_x, align_y) = match corner {
        Corner::TopLeft => (Alignment::Start, Alignment::Start),
        Corner::TopRight => (Alignment::End, Alignment::Start),
        Corner::BottomLeft => (Alignment::Start, Alignment::End),
        Corner::BottomRight => (Alignment::End, Alignment::End),
    };

    container(btn)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(align_x)
        .align_y(align_y)
        .padding(padding)
        .into()
}

// ── Item collections ──────────────────────────────────────────────────────────

/// Build the item list for a side menu with the correct orientation.
///
/// - `Left` / `Right` → vertical `Column` (items stacked top-to-bottom).
/// - `Top` / `Bottom` → horizontal `Row` (items placed left-to-right).
fn items_for_side(
    items: &[MenuItemDescriptor],
    hovered_idx: Option<usize>,
    config: &MenuConfig,
    side: Side,
) -> Element<'static, NavMessage> {
    let buttons: Vec<Element<'static, NavMessage>> = items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let sz = magnified_size(config, hovered_idx, idx);
            side_item_button(item, sz, side)
        })
        .collect();

    match side {
        Side::Left | Side::Right => {
            let col = Column::from_vec(buttons).spacing(2);
            if items.len() > SCROLL_THRESHOLD {
                scrollable(col).into()
            } else {
                col.into()
            }
        }
        Side::Top | Side::Bottom => {
            let row = Row::from_vec(buttons).spacing(2);
            if items.len() > SCROLL_THRESHOLD {
                scrollable(row).into()
            } else {
                row.into()
            }
        }
    }
}

// ── Inline SVG icon data ──────────────────────────────────────────────────────

/// Inline SVG strings for the `fs:nav/*` icon namespace.
///
/// Replaces `currentColor` at call time so iced's `resvg` renderer sees a
/// concrete fill/stroke value.
const ICON_LAUNCHER: &str = r#"<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/></svg>"#;
const ICON_STORE: &str = r#"<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 2L3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4z"/><line x1="3" y1="6" x2="21" y2="6"/><path d="M16 10a4 4 0 0 1-8 0"/></svg>"#;
const ICON_BROWSER: &str = r#"<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>"#;
const ICON_LENSES: &str = r#"<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>"#;
const ICON_TASKS: &str = r#"<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 11 12 14 22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>"#;
const ICON_BOTS: &str = r#"<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="4" width="16" height="16" rx="2"/><rect x="9" y="9" width="6" height="6"/><line x1="9" y1="1" x2="9" y2="4"/><line x1="15" y1="1" x2="15" y2="4"/><line x1="9" y1="20" x2="9" y2="23"/><line x1="15" y1="20" x2="15" y2="23"/><line x1="20" y1="9" x2="23" y2="9"/><line x1="20" y1="14" x2="23" y2="14"/><line x1="1" y1="9" x2="4" y2="9"/><line x1="1" y1="14" x2="4" y2="14"/></svg>"#;
const ICON_MANAGERS: &str = r#"<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>"#;
const ICON_PROFILE: &str = r#"<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>"#;
const ICON_SETTINGS: &str = r#"<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>"#;
const ICON_HELP: &str = r#"<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>"#;
const ICON_AI: &str = r#"<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>"#;
const ICON_DESKTOP: &str = r#"<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2"/><polyline points="8 21 12 17 16 21"/></svg>"#;
const ICON_CONTAINER: &str = r#"<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>"#;

/// Resolve an `fs:nav/*` or `fs:apps/*` icon key to an inline SVG string.
///
/// Falls back to `None` when the key is unknown so callers can use a text
/// fallback instead.
fn resolve_inline_svg(icon_key: &str) -> Option<&'static str> {
    // Strip namespace prefix (e.g. "fs:nav/" or "fs:apps/") → bare name.
    let name = icon_key.split('/').next_back().unwrap_or(icon_key);
    Some(match name {
        "launcher" => ICON_LAUNCHER,
        "store" => ICON_STORE,
        "browser" => ICON_BROWSER,
        "lenses" => ICON_LENSES,
        "tasks" => ICON_TASKS,
        "bots" => ICON_BOTS,
        "managers" => ICON_MANAGERS,
        "profile" => ICON_PROFILE,
        "settings" => ICON_SETTINGS,
        "help" => ICON_HELP,
        "ai" | "assistant" => ICON_AI,
        "desktop" => ICON_DESKTOP,
        "container" => ICON_CONTAINER,
        _ => return None,
    })
}

/// Build an iced SVG handle from a raw SVG string.
///
/// Replaces `currentColor` with a concrete hex value so `resvg` renders it.
fn svg_handle_from_str(svg_str: &str, color: &str) -> svg::Handle {
    let data = svg_str
        .replace("stroke=\"currentColor\"", &format!("stroke=\"{color}\""))
        .replace("fill=\"currentColor\"", &format!("fill=\"{color}\""));
    svg::Handle::from_memory(data.into_bytes())
}

// ── Double-logo icon element ──────────────────────────────────────────────────

/// Render a [`CompositeIcon`] as an iced element.
///
/// **Double-logo strategy:**
/// - Primary icon → full size `sz × sz`, represents *who we are* (which program).
/// - Secondary icon → half size `(sz/2) × (sz/2)`, represents *what we are doing*
///   (active sub-view, selected language flag, etc.).  Placed at the bottom-right
///   corner of the primary, slightly overlapping it.
///
/// When no secondary is present, returns just the primary icon.
fn composite_icon_element(
    icon: &fs_render::navigation::CompositeIcon,
    sz: f32,
    primary_color: &str,
) -> Element<'static, NavMessage> {
    let primary_el: Element<'static, NavMessage> =
        if let Some(svg_str) = resolve_inline_svg(&icon.primary.key) {
            let handle = svg_handle_from_str(svg_str, primary_color);
            svg(handle)
                .width(Length::Fixed(sz))
                .height(Length::Fixed(sz))
                .into()
        } else {
            let short: String = icon.primary.key.chars().take(2).collect();
            text(short).size(sz / 2.0).color(Color::WHITE).into()
        };

    let Some(sec_ref) = &icon.secondary else {
        return primary_el;
    };

    let sec_sz = sz / 2.0;
    let Some(sec_svg_str) = resolve_inline_svg(&sec_ref.key) else {
        return primary_el;
    };

    // Secondary icon in cyan to visually distinguish it from the primary.
    let sec_handle = svg_handle_from_str(sec_svg_str, "#06b6d4");
    let sec_el: Element<'static, NavMessage> = svg(sec_handle)
        .width(Length::Fixed(sec_sz))
        .height(Length::Fixed(sec_sz))
        .into();

    // Position secondary at bottom-right of the primary bounding box.
    let secondary_layer: Element<'static, NavMessage> = container(sec_el)
        .width(Length::Fixed(sz))
        .height(Length::Fixed(sz))
        .align_x(Alignment::End)
        .align_y(Alignment::End)
        .padding(0)
        .into();

    stack(vec![primary_el, secondary_layer])
        .width(Length::Fixed(sz))
        .height(Length::Fixed(sz))
        .into()
}

// ── Item buttons ──────────────────────────────────────────────────────────────

/// Render a single corner-menu item as an icon button with a tooltip.
///
/// Respects the `CompositeIcon` double-logo: primary at full size, optional
/// secondary at half size overlaid at the bottom-right.
fn corner_item_button(
    item: &MenuItemDescriptor,
    height: f32,
    corner: Corner,
) -> Element<'static, NavMessage> {
    let label = item.label_key.clone();
    let action = item.action.clone();
    let has_sub = !item.sub_items.is_empty();
    let sz = height.max(24.0);

    let icon_el = composite_icon_element(&item.icon, sz, "#e2e8f0");

    let sub_badge: Element<'static, NavMessage> = if has_sub {
        text("\u{25b6}").size(8.0).color(Color::WHITE).into()
    } else {
        Space::new().into()
    };

    let btn_content: Element<'static, NavMessage> = iced::widget::row(vec![icon_el, sub_badge])
        .spacing(0)
        .into();

    let tooltip_label: Element<'static, NavMessage> =
        iced::widget::container(text(label.clone()).size(12).color(Color::WHITE))
            .padding([4, 8])
            .style(|_theme: &Theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.04, 0.06, 0.14, 0.92))),
                border: Border {
                    color: Color::from_rgba(0.02, 0.74, 0.84, 0.35),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..iced::widget::container::Style::default()
            })
            .into();

    let tooltip_pos = match corner {
        Corner::TopLeft | Corner::BottomLeft => iced::widget::tooltip::Position::Right,
        Corner::TopRight | Corner::BottomRight => iced::widget::tooltip::Position::Left,
    };

    let btn = button(btn_content)
        .on_press(NavMessage::CornerMenuAction(corner, action))
        .width(Length::Fixed(sz + 8.0))
        .height(Length::Fixed(sz + 8.0))
        .style(|_theme: &Theme, _status| iced::widget::button::Style {
            background: None,
            text_color: Color::WHITE,
            ..iced::widget::button::Style::default()
        })
        .padding(4);

    Tooltip::new(btn, tooltip_label, tooltip_pos).gap(4).into()
}

/// Render a single side-menu item as a transparent icon button with tooltip.
///
/// Square sizing so items work correctly in both vertical (Column) and
/// horizontal (Row) layouts.
fn side_item_button(
    item: &MenuItemDescriptor,
    height: f32,
    side: Side,
) -> Element<'static, NavMessage> {
    let label = item.label_key.clone();
    let action = item.action.clone();
    let sz = height.max(24.0);

    let icon_el = composite_icon_element(&item.icon, sz, "#e2e8f0");

    let tooltip_pos = match side {
        Side::Left => iced::widget::tooltip::Position::Right,
        Side::Right => iced::widget::tooltip::Position::Left,
        Side::Top => iced::widget::tooltip::Position::Bottom,
        Side::Bottom => iced::widget::tooltip::Position::Top,
    };

    let tooltip_label: Element<'static, NavMessage> =
        iced::widget::container(text(label.clone()).size(12).color(Color::WHITE))
            .padding([4, 8])
            .style(|_theme: &Theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.04, 0.06, 0.14, 0.92))),
                border: Border {
                    color: Color::from_rgba(0.02, 0.74, 0.84, 0.35),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..iced::widget::container::Style::default()
            })
            .into();

    let btn = button(icon_el)
        .on_press(NavMessage::SideMenuAction(side, action))
        // Square button so it works in both Column and Row layouts.
        .width(Length::Fixed(sz + 8.0))
        .height(Length::Fixed(sz + 8.0))
        .style(|_theme: &Theme, _status| iced::widget::button::Style {
            background: None,
            text_color: Color::WHITE,
            ..iced::widget::button::Style::default()
        })
        .padding([2, 4]);

    Tooltip::new(btn, tooltip_label, tooltip_pos).gap(4).into()
}

// ── Hover magnification ───────────────────────────────────────────────────────

/// Compute the item size for index `idx` given the hovered item's index.
///
/// Uses the same exponential falloff as [`HoverMagnification::size_at_distance`]:
/// items nearer the cursor grow toward `max_icon_size`, items farther away
/// stay near `icon_size`.
fn magnified_size(config: &MenuConfig, hovered_idx: Option<usize>, idx: usize) -> f32 {
    let Some(cursor_idx) = hovered_idx else {
        return config.icon_size;
    };
    #[allow(clippy::cast_precision_loss)]
    let distance = (idx as f32 - cursor_idx as f32).abs();
    let range = config.max_icon_size - config.icon_size;
    let factor = (-distance / config.spread.max(f32::EPSILON)).exp();
    config.icon_size + range * factor
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use fs_render::navigation::{CompositeIcon, IconRef, MenuItemDescriptor};

    // ── helpers ───────────────────────────────────────────────────────────────

    fn make_item(id: &str) -> MenuItemDescriptor {
        MenuItemDescriptor::new(
            id,
            CompositeIcon::single(IconRef::new("fs:nav/test")),
            format!("nav-{id}"),
            format!("open:{id}"),
        )
    }

    fn make_item_with_secondary(id: &str, secondary_key: &str) -> MenuItemDescriptor {
        MenuItemDescriptor::new(
            id,
            CompositeIcon::with_instance(
                IconRef::new(format!("fs:nav/{id}")),
                IconRef::new(secondary_key.to_string()),
                0.3,
            ),
            format!("nav-{id}"),
            format!("open:{id}"),
        )
    }

    struct TestCornerMenu {
        corner: Corner,
        items: Vec<MenuItemDescriptor>,
    }
    impl CornerMenuDescriptor for TestCornerMenu {
        fn corner(&self) -> Corner {
            self.corner
        }
        fn items(&self) -> Vec<MenuItemDescriptor> {
            self.items.clone()
        }
    }

    struct TestSideMenu {
        side: Side,
        items: Vec<MenuItemDescriptor>,
    }
    impl SideMenuDescriptor for TestSideMenu {
        fn side(&self) -> Side {
            self.side
        }
        fn items(&self) -> Vec<MenuItemDescriptor> {
            self.items.clone()
        }
    }

    // ── MenuConfig ────────────────────────────────────────────────────────────

    #[test]
    fn menu_config_default_accent_is_cyan() {
        let cfg = MenuConfig::default();
        assert!(cfg.accent.g > 0.8);
        assert!(cfg.accent.b > 0.8);
        assert!(cfg.accent.r < 0.1);
    }

    // ── magnified_size ────────────────────────────────────────────────────────

    #[test]
    fn magnified_size_no_hover_returns_base() {
        let cfg = MenuConfig::default();
        let size = magnified_size(&cfg, None, 3);
        assert!((size - cfg.icon_size).abs() < f32::EPSILON);
    }

    #[test]
    fn magnified_size_at_cursor_returns_max() {
        let cfg = MenuConfig::default();
        let size = magnified_size(&cfg, Some(2), 2);
        assert!((size - cfg.max_icon_size).abs() < 0.01);
    }

    #[test]
    fn magnified_size_far_item_approaches_base() {
        let cfg = MenuConfig::default();
        let size = magnified_size(&cfg, Some(0), 20);
        assert!(size < cfg.icon_size + 0.01);
        assert!(size >= cfg.icon_size - f32::EPSILON);
    }

    #[test]
    fn magnified_size_neighbours_smaller_than_cursor() {
        let cfg = MenuConfig::default();
        let at_cursor = magnified_size(&cfg, Some(3), 3);
        let neighbour = magnified_size(&cfg, Some(3), 4);
        assert!(at_cursor > neighbour);
    }

    // ── update_corner_menu ────────────────────────────────────────────────────

    #[test]
    fn corner_toggle_opens_and_closes() {
        let mut state = CornerMenuState::default();
        assert!(!state.open);
        update_corner_menu(
            &mut state,
            Corner::TopLeft,
            &NavMessage::CornerMenuToggle(Corner::TopLeft),
        );
        assert!(state.open);
        update_corner_menu(
            &mut state,
            Corner::TopLeft,
            &NavMessage::CornerMenuToggle(Corner::TopLeft),
        );
        assert!(!state.open);
    }

    #[test]
    fn corner_toggle_ignores_other_corner() {
        let mut state = CornerMenuState::default();
        update_corner_menu(
            &mut state,
            Corner::TopLeft,
            &NavMessage::CornerMenuToggle(Corner::TopRight),
        );
        assert!(!state.open);
    }

    #[test]
    fn corner_hover_sets_and_clears_index() {
        let mut state = CornerMenuState::default();
        update_corner_menu(
            &mut state,
            Corner::TopLeft,
            &NavMessage::CornerMenuItemEntered(Corner::TopLeft, 3),
        );
        assert_eq!(state.hovered_idx, Some(3));
        update_corner_menu(
            &mut state,
            Corner::TopLeft,
            &NavMessage::CornerMenuItemLeft(Corner::TopLeft),
        );
        assert_eq!(state.hovered_idx, None);
    }

    // ── update_side_menu ──────────────────────────────────────────────────────

    #[test]
    fn side_toggle_opens_and_closes() {
        let mut state = SideMenuState::default();
        update_side_menu(
            &mut state,
            Side::Left,
            &NavMessage::SideMenuToggle(Side::Left),
        );
        assert!(state.open);
        update_side_menu(
            &mut state,
            Side::Left,
            &NavMessage::SideMenuToggle(Side::Left),
        );
        assert!(!state.open);
    }

    #[test]
    fn side_toggle_ignores_other_side() {
        let mut state = SideMenuState::default();
        update_side_menu(
            &mut state,
            Side::Left,
            &NavMessage::SideMenuToggle(Side::Right),
        );
        assert!(!state.open);
    }

    #[test]
    fn side_hover_sets_and_clears_index() {
        let mut state = SideMenuState::default();
        update_side_menu(
            &mut state,
            Side::Left,
            &NavMessage::SideMenuItemEntered(Side::Left, 1),
        );
        assert_eq!(state.hovered_idx, Some(1));
        update_side_menu(
            &mut state,
            Side::Left,
            &NavMessage::SideMenuItemLeft(Side::Left),
        );
        assert_eq!(state.hovered_idx, None);
    }

    // ── render_corner_menu ────────────────────────────────────────────────────

    #[test]
    fn corner_menu_closed_renders_indicator_only() {
        let desc = TestCornerMenu {
            corner: Corner::TopLeft,
            items: vec![make_item("home"), make_item("settings")],
        };
        let state = CornerMenuState::default();
        let _el = render_corner_menu(&desc, &state, &MenuConfig::default());
    }

    #[test]
    fn corner_menu_open_renders_without_panic() {
        let desc = TestCornerMenu {
            corner: Corner::BottomRight,
            items: vec![make_item("apps"), make_item("store")],
        };
        let state = CornerMenuState {
            open: true,
            hovered_idx: Some(0),
        };
        let _el = render_corner_menu(&desc, &state, &MenuConfig::default());
    }

    #[test]
    fn corner_menu_many_items_renders_arc_without_panic() {
        // Items beyond SCROLL_THRESHOLD still render correctly on the arc.
        let items: Vec<_> = (0..12).map(|i| make_item(&i.to_string())).collect();
        let desc = TestCornerMenu {
            corner: Corner::TopRight,
            items,
        };
        let state = CornerMenuState {
            open: true,
            hovered_idx: None,
        };
        let _el = render_corner_menu(&desc, &state, &MenuConfig::default());
    }

    #[test]
    fn corner_menu_sub_item_indicator_no_panic() {
        let parent =
            make_item("parent").with_sub_items(vec![make_item("child-a"), make_item("child-b")]);
        let desc = TestCornerMenu {
            corner: Corner::TopLeft,
            items: vec![parent],
        };
        let state = CornerMenuState {
            open: true,
            hovered_idx: None,
        };
        let _el = render_corner_menu(&desc, &state, &MenuConfig::default());
    }

    #[test]
    fn all_corners_render_open_without_panic() {
        let state = CornerMenuState {
            open: true,
            hovered_idx: None,
        };
        for corner in [
            Corner::TopLeft,
            Corner::TopRight,
            Corner::BottomLeft,
            Corner::BottomRight,
        ] {
            let desc = TestCornerMenu {
                corner,
                items: vec![make_item("a"), make_item("b")],
            };
            let _el = render_corner_menu(&desc, &state, &MenuConfig::default());
        }
    }

    // ── render_side_menu ──────────────────────────────────────────────────────

    #[test]
    fn side_menu_closed_renders_indicator_only() {
        let desc = TestSideMenu {
            side: Side::Left,
            items: vec![make_item("apps")],
        };
        let _el = render_side_menu(&desc, &SideMenuState::default(), &MenuConfig::default());
    }

    #[test]
    fn side_menu_open_renders_without_panic() {
        let desc = TestSideMenu {
            side: Side::Right,
            items: vec![make_item("home"), make_item("tasks")],
        };
        let state = SideMenuState {
            open: true,
            hovered_idx: Some(1),
        };
        let _el = render_side_menu(&desc, &state, &MenuConfig::default());
    }

    #[test]
    fn side_menu_top_bottom_renders_horizontal() {
        // Top and Bottom menus should render without panic (horizontal Row layout).
        for side in [Side::Top, Side::Bottom] {
            let desc = TestSideMenu {
                side,
                items: vec![make_item("a"), make_item("b"), make_item("c")],
            };
            let state = SideMenuState {
                open: true,
                hovered_idx: None,
            };
            let _el = render_side_menu(&desc, &state, &MenuConfig::default());
        }
    }

    #[test]
    fn side_menu_scroll_fallback_beyond_threshold() {
        let items: Vec<_> = (0..10).map(|i| make_item(&i.to_string())).collect();
        let desc = TestSideMenu {
            side: Side::Bottom,
            items,
        };
        let state = SideMenuState {
            open: true,
            hovered_idx: None,
        };
        let _el = render_side_menu(&desc, &state, &MenuConfig::default());
    }

    #[test]
    fn all_sides_render_open_without_panic() {
        let state = SideMenuState {
            open: true,
            hovered_idx: None,
        };
        for side in [Side::Left, Side::Right, Side::Top, Side::Bottom] {
            let desc = TestSideMenu {
                side,
                items: vec![make_item("a"), make_item("b")],
            };
            let _el = render_side_menu(&desc, &state, &MenuConfig::default());
        }
    }

    // ── Double logo (CompositeIcon) ───────────────────────────────────────────

    #[test]
    fn corner_menu_with_secondary_icon_renders_without_panic() {
        let item = make_item_with_secondary("settings", "fs:nav/language");
        let desc = TestCornerMenu {
            corner: Corner::BottomLeft,
            items: vec![item],
        };
        let state = CornerMenuState {
            open: true,
            hovered_idx: Some(0),
        };
        let _el = render_corner_menu(&desc, &state, &MenuConfig::default());
    }

    #[test]
    fn side_menu_with_secondary_icon_renders_without_panic() {
        let item = make_item_with_secondary("settings", "fs:nav/desktop");
        let desc = TestSideMenu {
            side: Side::Left,
            items: vec![item],
        };
        let state = SideMenuState {
            open: true,
            hovered_idx: Some(0),
        };
        let _el = render_side_menu(&desc, &state, &MenuConfig::default());
    }
}

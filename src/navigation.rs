// navigation.rs — CornerMenu / SideMenu iced renderer.
//
// Design Pattern: Interpreter
//   Reads CornerMenuDescriptor / SideMenuDescriptor (fs-render) and produces
//   iced Element trees.  No iced import leaks into application code — callers
//   only depend on fs-render traits and NavMessage.
//
// Visual approach:
//   Indicators   — styled buttons with asymmetric border-radius create the
//                  quarter-disk (corner) and half-disk (side) shapes without
//                  requiring the `canvas` feature.
//   Items        — Column of transparent buttons; scrollable when count exceeds
//                  SCROLL_THRESHOLD (mobile / small-screen fallback).
//   Magnification — item height computed via exponential falloff from the cursor
//                   item; matches HoverMagnification::size_at_distance semantics.
//
// All produced Elements have 'static lifetime — all captured data is Copy or
// owned, so no external borrows escape.

use fs_render::navigation::{
    Corner, CornerMenuDescriptor, MenuItemDescriptor, Side, SideMenuDescriptor,
};
use iced::border::Radius;
use iced::widget::{button, scrollable, text, Column, Space};
use iced::{Background, Border, Color, Element, Length, Shadow, Theme};

/// Number of items before the scroll fallback activates.
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
/// The quarter-disk indicator button is always visible.  When `state.open`,
/// the item list appears adjacent to it.  Items beyond [`SCROLL_THRESHOLD`]
/// are wrapped in a `scrollable` for mobile / small-screen fallback.
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

    let items_el = items_column_corner(&items, state.hovered_idx, config, corner);

    // Top corners: indicator at top, items below.
    // Bottom corners: items above, indicator at bottom.
    let indicator_first = matches!(corner, Corner::TopLeft | Corner::TopRight);

    if indicator_first {
        Column::new()
            .push(indicator)
            .push(items_el)
            .spacing(4)
            .into()
    } else {
        Column::new()
            .push(items_el)
            .push(indicator)
            .spacing(4)
            .into()
    }
}

// ── render_side_menu ──────────────────────────────────────────────────────────

/// Render a side menu as a self-contained iced [`Element`].
///
/// The half-disk indicator button is always visible.  When `state.open`,
/// the accordion item list appears adjacent to it.
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

    let items_el = items_column_side(&items, state.hovered_idx, config, side);

    // Left/Top: indicator first, items follow.
    // Right/Bottom: items first, indicator at edge.
    let indicator_first = matches!(side, Side::Left | Side::Top);

    if indicator_first {
        Column::new()
            .push(indicator)
            .push(items_el)
            .spacing(4)
            .into()
    } else {
        Column::new()
            .push(items_el)
            .push(indicator)
            .spacing(4)
            .into()
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

    button(Space::new(Length::Fixed(r), Length::Fixed(r)))
        .on_press(NavMessage::CornerMenuToggle(corner))
        .style(move |_theme: &Theme, _status| iced::widget::button::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius,
                ..Border::default()
            },
            text_color: Color::TRANSPARENT,
            shadow: Shadow::default(),
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

    button(Space::new(Length::Fixed(w), Length::Fixed(h)))
        .on_press(NavMessage::SideMenuToggle(side))
        .style(move |_theme: &Theme, _status| iced::widget::button::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius,
                ..Border::default()
            },
            text_color: Color::TRANSPARENT,
            shadow: Shadow::default(),
        })
        .padding(0)
        .into()
}

// ── Item columns ──────────────────────────────────────────────────────────────

fn items_column_corner(
    items: &[MenuItemDescriptor],
    hovered_idx: Option<usize>,
    config: &MenuConfig,
    corner: Corner,
) -> Element<'static, NavMessage> {
    let buttons: Vec<Element<'static, NavMessage>> = items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let h = magnified_size(config, hovered_idx, idx);
            corner_item_button(item, h, corner)
        })
        .collect();

    let col = Column::from_vec(buttons).spacing(2);
    if items.len() > SCROLL_THRESHOLD {
        scrollable(col).into()
    } else {
        col.into()
    }
}

fn items_column_side(
    items: &[MenuItemDescriptor],
    hovered_idx: Option<usize>,
    config: &MenuConfig,
    side: Side,
) -> Element<'static, NavMessage> {
    let buttons: Vec<Element<'static, NavMessage>> = items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let h = magnified_size(config, hovered_idx, idx);
            side_item_button(item, h, side)
        })
        .collect();

    let col = Column::from_vec(buttons).spacing(2);
    if items.len() > SCROLL_THRESHOLD {
        scrollable(col).into()
    } else {
        col.into()
    }
}

// ── Item buttons ──────────────────────────────────────────────────────────────

/// Render a single corner-menu item as a transparent button.
///
/// The button height reflects the hover-magnification effect.
/// Sub-items are indicated with a `▶` suffix on the label key.
fn corner_item_button(
    item: &MenuItemDescriptor,
    height: f32,
    corner: Corner,
) -> Element<'static, NavMessage> {
    let label = item.label_key.clone();
    let action = item.action.clone();
    let has_sub = !item.sub_items.is_empty();

    let display: String = if has_sub {
        format!("{label} \u{25b6}")
    } else {
        label
    };

    button(text(display).size(13.0))
        .on_press(NavMessage::CornerMenuAction(corner, action))
        .height(Length::Fixed(height.max(24.0)))
        .style(|_theme: &Theme, _status| iced::widget::button::Style {
            background: None,
            text_color: Color::WHITE,
            border: Border::default(),
            shadow: Shadow::default(),
        })
        .padding([2, 8])
        .into()
}

/// Render a single side-menu item as a transparent button.
fn side_item_button(
    item: &MenuItemDescriptor,
    height: f32,
    side: Side,
) -> Element<'static, NavMessage> {
    let label = item.label_key.clone();
    let action = item.action.clone();
    let has_sub = !item.sub_items.is_empty();

    let display: String = if has_sub {
        format!("{label} \u{25b6}")
    } else {
        label
    };

    button(text(display).size(13.0))
        .on_press(NavMessage::SideMenuAction(side, action))
        .height(Length::Fixed(height.max(24.0)))
        .style(|_theme: &Theme, _status| iced::widget::button::Style {
            background: None,
            text_color: Color::WHITE,
            border: Border::default(),
            shadow: Shadow::default(),
        })
        .padding([2, 8])
        .into()
}

// ── Hover magnification ───────────────────────────────────────────────────────

/// Compute the item height for index `idx` given the hovered item's index.
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
    fn corner_menu_scroll_fallback_beyond_threshold() {
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
}

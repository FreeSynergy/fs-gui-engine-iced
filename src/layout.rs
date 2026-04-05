// layout.rs — IcedLayoutInterpreter: translates LayoutDescriptor → iced Elements.
//
// Design Pattern: Interpreter
//   Walks the LayoutDescriptor AST and produces iced native elements.
//   All string content is cloned into owned data so the resulting `Element`
//   has `'static` lifetime and can be returned from any context.
//
// Animations (iced 0.13 + iced_aw):
//   - Spinner (iced_aw): animated loading indicator for unregistered components
//   - Badge   (iced_aw): pill/count badges in notification and status displays
//   - Card    (iced_aw): styled panel with header/body for each component

use fs_render::{
    ButtonStyle, ComponentCtx, ComponentRegistry, LayoutDescriptor, LayoutElement, LayoutError,
    LayoutInterpreter, ShellConfig, ShellKind, SlotConfig, SlotKind, TextSize,
};
use iced::widget::{button, container, row, scrollable, text, Column, Row, Space};
use iced::{Alignment, Element, Length, Theme};
use iced_aw::{Badge, Card, Spinner};

// ── Message type ──────────────────────────────────────────────────────────────

/// Messages produced by rendered components inside the layout.
#[derive(Debug, Clone)]
pub enum LayoutMessage {
    /// A component button was pressed — carries the action identifier.
    Action(String),
}

// ── IcedLayoutInterpreter ─────────────────────────────────────────────────────

/// Interprets a `LayoutDescriptor` and produces an iced `Element`.
///
/// The shell renders all four containers (topbar → sidebar → bottombar → main)
/// and assembles them into a single root `Element<'static, LayoutMessage>`.
///
/// All string data is cloned into owned values so the element tree has no
/// external lifetime dependencies.
pub struct IcedLayoutInterpreter<'a> {
    registry: &'a ComponentRegistry,
    ctx: ComponentCtx,
}

impl<'a> IcedLayoutInterpreter<'a> {
    /// Create an interpreter with the given component registry and render context.
    pub fn new(registry: &'a ComponentRegistry, ctx: ComponentCtx) -> Self {
        Self { registry, ctx }
    }

    // ── Shell rendering ───────────────────────────────────────────────────────

    fn render_shell(
        &self,
        shell: &ShellConfig,
        kind: &ShellKind,
    ) -> Element<'static, LayoutMessage> {
        if !shell.enabled {
            return Space::new(0, 0).into();
        }

        let slot_elements = self.render_slots(&shell.slots, kind);
        let col: Column<'static, LayoutMessage> = Column::from_vec(slot_elements).spacing(0);
        let width = if shell.size > 0 {
            #[allow(clippy::cast_precision_loss)]
            Length::Fixed(shell.size as f32)
        } else {
            Length::Fill
        };
        container(col)
            .width(width)
            .style(|_theme: &Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(
                    0.08, 0.08, 0.10, 0.96,
                ))),
                ..iced::widget::container::Style::default()
            })
            .into()
    }

    fn render_slots(
        &self,
        slots: &SlotConfig,
        shell: &ShellKind,
    ) -> Vec<Element<'static, LayoutMessage>> {
        let mut result: Vec<Element<'static, LayoutMessage>> = Vec::new();

        for comp_ref in &slots.top {
            let id = comp_ref.id.clone();
            let mut ctx = self.ctx.clone();
            ctx.shell = shell.clone();
            ctx.slot = SlotKind::Top;
            result.push(self.render_component(&id, &ctx));
        }
        for comp_ref in &slots.fill {
            let id = comp_ref.id.clone();
            let mut ctx = self.ctx.clone();
            ctx.shell = shell.clone();
            ctx.slot = SlotKind::Fill;
            result.push(self.render_component(&id, &ctx));
        }
        for comp_ref in &slots.bottom {
            let id = comp_ref.id.clone();
            let mut ctx = self.ctx.clone();
            ctx.shell = shell.clone();
            ctx.slot = SlotKind::Bottom;
            result.push(self.render_component(&id, &ctx));
        }
        result
    }

    fn render_component(&self, id: &str, ctx: &ComponentCtx) -> Element<'static, LayoutMessage> {
        let Some(component) = self.registry.get(id) else {
            // Not yet registered — animated loading card
            let spinner: Element<'static, LayoutMessage> = Spinner::new()
                .width(Length::Fixed(16.0))
                .height(Length::Fixed(16.0))
                .into();
            let label: Element<'static, LayoutMessage> = text("Loading…").size(12).into();
            let body: Element<'static, LayoutMessage> = row![spinner, label]
                .spacing(6)
                .align_y(Alignment::Center)
                .into();
            return Card::new(text(id.to_string()).size(12), body)
                .padding(iced::Padding::new(8.0))
                .into();
        };

        let name = component.name_key().to_string();
        let elements = component.render(ctx);
        let iced_elements: Vec<Element<'static, LayoutMessage>> =
            elements.into_iter().map(render_element).collect();
        let body: Column<'static, LayoutMessage> = Column::from_vec(iced_elements).spacing(4);

        Card::new(text(name).size(13), body)
            .max_width(f32::MAX)
            .padding(iced::Padding::new(8.0))
            .into()
    }
}

impl LayoutInterpreter for IcedLayoutInterpreter<'_> {
    type Output = Element<'static, LayoutMessage>;

    fn interpret(&self, descriptor: &LayoutDescriptor) -> Element<'static, LayoutMessage> {
        let topbar = self.render_shell(&descriptor.topbar, &ShellKind::Topbar);
        let sidebar = self.render_shell(&descriptor.sidebar, &ShellKind::Sidebar);
        let bottombar = self.render_shell(&descriptor.bottombar, &ShellKind::Bottombar);
        let main_area = self.render_shell(&descriptor.main, &ShellKind::Main);

        let center: Element<'static, LayoutMessage> =
            Row::new().push(sidebar).push(main_area).spacing(0).into();

        container(
            Column::new()
                .push(topbar)
                .push(center)
                .push(bottombar)
                .spacing(0),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

// ── Element translation ───────────────────────────────────────────────────────

/// Translate a `LayoutElement` into an iced `Element<'static, LayoutMessage>`.
///
/// All content is owned (cloned) — no external lifetime dependencies.
pub fn render_element(element: LayoutElement) -> Element<'static, LayoutMessage> {
    match element {
        LayoutElement::Text {
            content,
            size,
            color,
        } => {
            let size_px = text_size_px(&size);
            let mut t = text(content).size(size_px);
            if let Some(c) = color {
                t = t.color(iced::Color::from_rgba(c.r, c.g, c.b, c.a));
            }
            t.into()
        }

        LayoutElement::Button {
            label_key,
            action,
            style,
        } => button(text(label_key).size(13))
            .on_press(LayoutMessage::Action(action))
            .style(move |theme: &Theme, status| match style {
                ButtonStyle::Primary => iced::widget::button::primary(theme, status),
                ButtonStyle::Ghost => iced::widget::button::text(theme, status),
                ButtonStyle::Danger => iced::widget::button::danger(theme, status),
            })
            .into(),

        LayoutElement::Icon { name, size } => render_icon(&name, size),

        LayoutElement::Row { children, gap } => {
            let kids: Vec<Element<'static, LayoutMessage>> =
                children.into_iter().map(render_element).collect();
            Row::from_vec(kids)
                .spacing(gap)
                .align_y(Alignment::Center)
                .into()
        }

        LayoutElement::Column { children, gap } => {
            let kids: Vec<Element<'static, LayoutMessage>> =
                children.into_iter().map(render_element).collect();
            Column::from_vec(kids).spacing(gap).into()
        }

        LayoutElement::List {
            items,
            scrollable: use_scroll,
        } => {
            let kids: Vec<Element<'static, LayoutMessage>> =
                items.into_iter().map(render_element).collect();
            let col = Column::from_vec(kids).spacing(2);
            if use_scroll {
                scrollable(col).into()
            } else {
                col.into()
            }
        }

        LayoutElement::Separator { label_key } => match label_key {
            Some(key) => text(key).size(11).into(),
            None => Space::new(Length::Fill, 1).into(),
        },

        LayoutElement::Badge { content, color: _ } => Badge::new(text(content).size(11)).into(),

        LayoutElement::Spinner => Spinner::new()
            .width(Length::Fixed(20.0))
            .height(Length::Fixed(20.0))
            .into(),

        LayoutElement::Spacer { pixels } => Space::new(0, pixels).into(),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn text_size_px(size: &TextSize) -> u16 {
    match size {
        TextSize::Tiny => 10,
        TextSize::Body => 13,
        TextSize::Label => 14,
        TextSize::Subheading => 16,
        TextSize::Heading => 20,
    }
}

/// Render an icon by name.
///
/// Tries to load an SVG from the data directory first.  Falls back to a
/// single-character emoji text widget when no SVG file is found.
fn render_icon(name: &str, size: u32) -> Element<'static, LayoutMessage> {
    // SVG search paths (icon-set artifacts are installed into these directories).
    let candidates = icon_svg_paths(name);
    for path in candidates {
        if path.exists() {
            #[allow(clippy::cast_precision_loss)]
            let px = size as f32;
            return iced::widget::svg(iced::widget::svg::Handle::from_path(path))
                .width(Length::Fixed(px))
                .height(Length::Fixed(px))
                .into();
        }
    }
    // Fallback: render the name as a small bracketed text icon.
    #[allow(clippy::cast_possible_truncation)]
    text(icon_emoji_fallback(name)).size(size as u16).into()
}

/// Candidate SVG paths for a given icon name.
fn icon_svg_paths(name: &str) -> Vec<std::path::PathBuf> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let base_name = format!("{name}.svg");
    vec![
        // User icon-set artifacts (installed from Store).
        std::path::PathBuf::from(&home)
            .join(".local/share/freesynergy/icons")
            .join(&base_name),
        // System-wide icon-set artifacts.
        std::path::PathBuf::from("/var/lib/freesynergy/icons").join(&base_name),
        // Binary-relative assets (development / bundled builds).
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("assets/icons").join(&base_name)))
            .unwrap_or_default(),
    ]
}

/// A short emoji fallback for well-known icon names.
fn icon_emoji_fallback(name: &str) -> &'static str {
    match name {
        "settings" | "preferences" => "⚙",
        "apps" | "launcher" => "⊞",
        "help" | "question" => "?",
        "ai" | "assistant" => "✦",
        "notifications" | "bell" => "🔔",
        "search" => "⌕",
        "close" | "quit" => "✕",
        "pin" => "📌",
        "unpin" => "📍",
        _ => "●",
    }
}

/// Load a layout from file, returning the default if the file is absent.
///
/// # Errors
///
/// Returns [`LayoutError`] for TOML parse errors only.
pub fn load_layout_or_default(path: &std::path::Path) -> Result<LayoutDescriptor, LayoutError> {
    match LayoutDescriptor::from_file(path) {
        Ok(d) => Ok(d),
        Err(LayoutError::NotFound(_)) => Ok(LayoutDescriptor::default()),
        Err(e) => Err(e),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use fs_render::{ComponentRegistry, LayoutDescriptor};

    #[test]
    fn interpreter_no_panic_on_empty_layout() {
        let registry = ComponentRegistry::new();
        let ctx = ComponentCtx::test(ShellKind::Main, SlotKind::Fill);
        let interpreter = IcedLayoutInterpreter::new(&registry, ctx);
        let desc = LayoutDescriptor::default();
        let _element = interpreter.interpret(&desc);
    }

    #[test]
    fn text_size_tiny_is_ten() {
        assert_eq!(text_size_px(&TextSize::Tiny), 10);
    }

    #[test]
    fn text_size_heading_is_twenty() {
        assert_eq!(text_size_px(&TextSize::Heading), 20);
    }

    #[test]
    fn load_missing_file_returns_default() {
        let result = load_layout_or_default(std::path::Path::new("/nonexistent/layout.toml"));
        assert!(result.is_ok());
        assert!(result.unwrap().topbar.enabled);
    }

    #[test]
    fn render_element_text_body() {
        let el = LayoutElement::Text {
            content: "hello".into(),
            size: TextSize::Body,
            color: None,
        };
        let _e: Element<'static, LayoutMessage> = render_element(el);
    }

    #[test]
    fn render_element_spinner() {
        let _e: Element<'static, LayoutMessage> = render_element(LayoutElement::Spinner);
    }

    #[test]
    fn render_element_badge() {
        let _e: Element<'static, LayoutMessage> = render_element(LayoutElement::Badge {
            content: "3".into(),
            color: None,
        });
    }

    #[test]
    fn render_element_row_nested() {
        let el = LayoutElement::Row {
            children: vec![
                LayoutElement::Text {
                    content: "A".into(),
                    size: TextSize::Body,
                    color: None,
                },
                LayoutElement::Spacer { pixels: 8 },
            ],
            gap: 4,
        };
        let _e: Element<'static, LayoutMessage> = render_element(el);
    }

    #[test]
    fn render_element_button_primary() {
        let _e: Element<'static, LayoutMessage> = render_element(LayoutElement::Button {
            label_key: "Open".into(),
            action: "open-app".into(),
            style: ButtonStyle::Primary,
        });
    }
}

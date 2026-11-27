//! The look of the application, see DESIGN.md: text sizes, spacing, contrast, and the shared button, badge and
//! empty-state helpers that every screen uses instead of its own colours and sizes.

use eframe::egui::{self, vec2, Color32, CornerRadius, FontFamily, FontId, RichText, Stroke, TextStyle, Ui};

/// The accent colour: primary buttons, selection, the active tab.
pub const ACCENT: Color32 = Color32::from_rgb(0x25, 0x63, 0xeb);
/// Arrow colours of Send and Receive actions, readable in light and dark mode.
pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(0x3b, 0x82, 0xf6);
pub const ACCENT_RECEIVE: Color32 = Color32::from_rgb(0x10, 0xa3, 0x7f);
/// Colour bars of the partner rows (first and second partner).
/// Not red, amber or green: those are status colours.
pub const PARTNER_COLORS: [Color32; 2] = [Color32::from_rgb(0x3b, 0x82, 0xf6), Color32::from_rgb(0x8b, 0x5c, 0xf6)];

/// Applies the theme to the light and the dark style.
pub fn apply(ctx: &egui::Context) {
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Small, FontId::new(12.0, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Heading, FontId::new(18.0, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace)),
        ]
        .into();
        let spacing = &mut style.spacing;
        spacing.item_spacing = vec2(8.0, 6.0);
        spacing.button_padding = vec2(10.0, 5.0);
        spacing.interact_size.y = 26.0;
        spacing.combo_height = 300.0;
        // Solid scroll bars: a floating bar is hidden until hovered, so a cut-off form would not show it scrolls
        spacing.scroll = egui::style::ScrollStyle::solid();

        let visuals = &mut style.visuals;
        let dark = visuals.dark_mode;
        let gray = Color32::from_gray;
        visuals.selection.bg_fill = if dark { Color32::from_rgb(0x1e, 0x40, 0x8a) } else { Color32::from_rgb(0xdb, 0xe7, 0xfd) };
        visuals.selection.stroke = Stroke::new(1.0, if dark { Color32::WHITE } else { Color32::from_rgb(0x1e, 0x3a, 0x8a) });
        visuals.extreme_bg_color = if dark { gray(24) } else { Color32::WHITE };
        visuals.faint_bg_color = if dark { gray(34) } else { gray(246) };
        visuals.panel_fill = if dark { gray(30) } else { gray(250) };
        visuals.window_fill = visuals.panel_fill;
        // Labels and disabled widgets stay readable (at least 3:1)
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, if dark { gray(215) } else { gray(40) });
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, if dark { gray(60) } else { gray(215) });
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, if dark { gray(230) } else { gray(30) });
        visuals.widgets.inactive.bg_fill = if dark { gray(52) } else { gray(236) };
        visuals.widgets.inactive.weak_bg_fill = visuals.widgets.inactive.bg_fill;
        // Fields and buttons always have a visible border
        // at least 3:1 against the panel (WCAG 1.4.11)
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, if dark { gray(112) } else { gray(138) });
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, if dark { gray(150) } else { gray(100) });
        // Disabled text keeps at least 3:1 contrast
        // A disabled button must look clearly different from an enabled one, yet stay readable (about 4:1)
        visuals.disabled_alpha = 0.5;
        // Weak text (hints, paths, secondary text) still reaches 4.5:1
        visuals.weak_text_color = Some(if dark { gray(160) } else { gray(100) });
        for widget in [
            &mut visuals.widgets.noninteractive,
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
        ] {
            widget.corner_radius = CornerRadius::same(4);
        }
    });
}

/// Adds a button that is primary when `primary` is true and a standard button otherwise, so a screen shows one
/// primary button at a time. A disabled button is never primary: a faded accent fill looks half enabled.
pub fn action_button(ui: &mut Ui, text: &str, primary: bool, enabled: bool) -> egui::Response {
    if primary && enabled {
        ui.add(primary_button(text))
    } else {
        ui.add_enabled(enabled, egui::Button::new(text.to_string()).min_size(vec2(96.0, 28.0)))
    }
}

/// The main action of a screen: accent fill, white bold text.
pub fn primary_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_string()).strong().color(Color32::WHITE))
        .fill(ACCENT)
        .min_size(vec2(96.0, 28.0))
}

/// Adds a delete button: red text when enabled, the normal disabled look otherwise (red would read as active).
pub fn danger(ui: &mut Ui, text: &str, enabled: bool) -> egui::Response {
    if enabled {
        let button = danger_button(ui, text);
        ui.add(button)
    } else {
        ui.add_enabled(false, egui::Button::new(text.to_string()))
    }
}

/// A delete button for `ui.add`: red when enabled, the normal disabled look otherwise.
pub fn danger_button_state(ui: &Ui, text: &str, enabled: bool) -> impl egui::Widget + 'static {
    let button = if enabled { danger_button(ui, text) } else { egui::Button::new(text.to_string()) };
    move |ui: &mut Ui| ui.add_enabled(enabled, button)
}

/// A delete action: red text.
pub fn danger_button(ui: &Ui, text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_string()).color(error_color(ui)))
}

pub fn ok_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode { Color32::from_rgb(0x4a, 0xde, 0x80) } else { Color32::from_rgb(0x12, 0x6b, 0x33) }
}

pub fn error_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode { Color32::from_rgb(0xf8, 0x71, 0x71) } else { Color32::from_rgb(0xb9, 0x1c, 0x1c) }
}

pub fn warning_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode { Color32::from_rgb(0xfb, 0xbf, 0x24) } else { Color32::from_rgb(0x92, 0x40, 0x0e) }
}

/// Draws a field (or any widget) with a red border: the value in it is not valid.
pub fn with_error_border<R>(ui: &mut Ui, invalid: bool, add: impl FnOnce(&mut Ui) -> R) -> R {
    if !invalid {
        return add(ui);
    }
    ui.scope(|ui| {
        let stroke = Stroke::new(1.5, error_color(ui));
        let widgets = &mut ui.visuals_mut().widgets;
        widgets.inactive.bg_stroke = stroke;
        widgets.hovered.bg_stroke = stroke;
        widgets.active.bg_stroke = stroke;
        ui.visuals_mut().selection.stroke = stroke;
        add(ui)
    })
    .inner
}

/// A short status text on a coloured background, e.g. "Valid" or "3 errors".
pub fn badge(ui: &mut Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.18))
        .stroke(Stroke::new(1.0, color))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(10, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(text).strong().color(color));
        });
}

/// A centred message for a screen or panel without content: what is missing, what to do, and an optional action.
pub fn empty_state(ui: &mut Ui, heading: &str, text: &str, action: impl FnOnce(&mut Ui)) {
    let height = ui.available_height();
    ui.vertical_centered(|ui| {
        ui.add_space((height * 0.3).max(16.0));
        ui.label(RichText::new(heading).heading().strong());
        ui.add_space(6.0);
        ui.label(RichText::new(text).weak());
        ui.add_space(14.0);
        action(ui);
    });
}

/// A light background with dark text for a notice, readable in light and dark mode.
pub fn notice(ui: &mut Ui, text: &str) {
    let (fill, stroke, color) = if ui.visuals().dark_mode {
        (Color32::from_rgb(0x42, 0x36, 0x10), Color32::from_rgb(0x85, 0x6a, 0x1a), Color32::from_rgb(0xfd, 0xe6, 0x8a))
    } else {
        (Color32::from_rgb(0xff, 0xf3, 0xcd), Color32::from_rgb(0xe6, 0xc8, 0x6e), Color32::from_rgb(0x66, 0x4d, 0x03))
    };
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, stroke))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(text).color(color));
        });
}

/// A heading above a group of fields, with a line under it.
pub fn section(ui: &mut Ui, text: &str) {
    ui.add_space(10.0);
    ui.label(RichText::new(text).size(15.0).strong().color(ui.visuals().strong_text_color()));
    ui.separator();
    ui.add_space(2.0);
}

/// Paints an arrow (shaft and head) for the direction of a message: right for sending, left for receiving.
/// Unlike a filled triangle it cannot be mistaken for the open/close toggle of a tree row.
pub fn paint_direction(painter: &egui::Painter, center: egui::Pos2, width: f32, right: bool, color: Color32) {
    let half = width / 2.0;
    let (start, end) = if right { (center.x - half, center.x + half) } else { (center.x + half, center.x - half) };
    let stroke = Stroke::new(1.6, color);
    let tip = egui::pos2(end, center.y);
    painter.line_segment([egui::pos2(start, center.y), tip], stroke);
    let back = if right { -4.0 } else { 4.0 };
    painter.line_segment([tip, tip + vec2(back, -3.5)], stroke);
    painter.line_segment([tip, tip + vec2(back, 3.5)], stroke);
}

/// The direction of a painted triangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arrow {
    Right,
    Left,
    Down,
}

/// Paints a small filled triangle; the default fonts have no arrow characters.
pub fn paint_arrow(painter: &egui::Painter, center: egui::Pos2, size: f32, arrow: Arrow, color: Color32) {
    let h = size / 2.0;
    let points = match arrow {
        Arrow::Right => vec![center + vec2(-h * 0.8, -h), center + vec2(h, 0.0), center + vec2(-h * 0.8, h)],
        Arrow::Left => vec![center + vec2(h * 0.8, -h), center + vec2(-h, 0.0), center + vec2(h * 0.8, h)],
        Arrow::Down => vec![center + vec2(-h, -h * 0.7), center + vec2(h, -h * 0.7), center + vec2(0.0, h * 0.8)],
    };
    painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
}

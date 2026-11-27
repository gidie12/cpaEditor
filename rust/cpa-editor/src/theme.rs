//! The look of the application, see DESIGN.md: text sizes, spacing, contrast, and the shared button, badge and
//! empty-state helpers that every screen uses instead of its own colours and sizes.

use eframe::egui::{self, vec2, Color32, CornerRadius, FontFamily, FontId, RichText, Stroke, TextStyle, Ui};

/// The accent colour: primary buttons, selection, the active tab.
pub const ACCENT: Color32 = Color32::from_rgb(0x25, 0x63, 0xeb);
/// Arrow colours of Send and Receive actions, readable in light and dark mode.
pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(0x3b, 0x82, 0xf6);
pub const ACCENT_RECEIVE: Color32 = Color32::from_rgb(0x10, 0xa3, 0x7f);
/// Colour bars of the partner rows (first and second partner).
pub const PARTNER_COLORS: [Color32; 2] = [Color32::from_rgb(0x3b, 0x82, 0xf6), Color32::from_rgb(0xe1, 0x1d, 0x48)];

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
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, if dark { gray(84) } else { gray(190) });
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, if dark { gray(130) } else { gray(140) });
        visuals.disabled_alpha = 0.6;
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

/// The main action of a screen: accent fill, white bold text.
pub fn primary_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_string()).strong().color(Color32::WHITE))
        .fill(ACCENT)
        .min_size(vec2(96.0, 28.0))
}

/// A delete action: red text.
pub fn danger_button(ui: &Ui, text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_string()).color(error_color(ui)))
}

pub fn ok_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode { Color32::from_rgb(0x4a, 0xde, 0x80) } else { Color32::from_rgb(0x15, 0x80, 0x3d) }
}

pub fn error_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode { Color32::from_rgb(0xf8, 0x71, 0x71) } else { Color32::from_rgb(0xb9, 0x1c, 0x1c) }
}

pub fn warning_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode { Color32::from_rgb(0xfb, 0xbf, 0x24) } else { Color32::from_rgb(0x92, 0x40, 0x0e) }
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

/// A heading above a group of fields.
pub fn section(ui: &mut Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(text).strong());
    ui.add_space(2.0);
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

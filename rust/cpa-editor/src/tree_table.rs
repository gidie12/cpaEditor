//! A tree with columns, the egui counterpart of Tk's `ttk.Treeview`: rows that can be opened and closed, one
//! selected row, resizable columns, colours per row, a context menu and double-click per cell.
//!
//! The caller passes the rows in pre-order with their depth on every frame; the widget keeps which rows are closed,
//! which row is selected and the column widths.

use std::collections::HashSet;

use eframe::egui::{
    self, pos2, vec2, Color32, CursorIcon, FontId, Rect, Response, Sense, Stroke, TextStyle, Ui,
};

/// Identifies a row across frames, for example the index of its XML node.
pub type RowKey = u64;

#[derive(Clone, Debug, Default)]
pub struct TreeRow {
    pub key: RowKey,
    pub depth: usize,
    /// Text per column; the first column is the tree column.
    pub cells: Vec<String>,
    pub text_color: Option<Color32>,
    pub background: Option<Color32>,
    pub strong: bool,
    /// A group header row: slightly larger bold text and a coloured bar on the left.
    pub header_bar: Option<Color32>,
    /// Weak text after the text of the first column.
    pub secondary: Option<String>,
    /// Text colour per column, over the row's colour (e.g. only the status cell red).
    pub cell_colors: Vec<Option<Color32>>,
    /// Columns whose coloured text is drawn as a badge (status), which stays readable on the selection.
    pub badge_columns: Vec<usize>,
    /// A direction arrow (true = right) painted before the text of the first column.
    pub icon: Option<(bool, Color32)>,
}

impl TreeRow {
    pub fn new(key: RowKey, depth: usize, cells: Vec<String>) -> Self {
        TreeRow { key, depth, cells, ..Default::default() }
    }

    pub fn text_color(mut self, color: Color32) -> Self {
        self.text_color = Some(color);
        self
    }

    pub fn background(mut self, color: Color32) -> Self {
        self.background = Some(color);
        self
    }

    pub fn strong(mut self) -> Self {
        self.strong = true;
        self
    }

    pub fn header(mut self, bar: Color32) -> Self {
        self.header_bar = Some(bar);
        self.strong = true;
        self
    }

    pub fn icon(mut self, right: bool, color: Color32) -> Self {
        self.icon = Some((right, color));
        self
    }

    pub fn secondary(mut self, text: String) -> Self {
        self.secondary = Some(text);
        self
    }
}

/// What happened in the tree during this frame.
#[derive(Default, Debug)]
pub struct TreeEvents {
    /// The selection changed to this row (by click, right-click or keyboard).
    pub selected: Option<RowKey>,
    /// A cell was double-clicked: row and column.
    pub double_clicked: Option<(RowKey, usize)>,
    /// The Delete key was pressed while the tree had a selection.
    pub delete_pressed: bool,
}

pub struct TreeTable {
    id: egui::Id,
    pub columns: Vec<(String, f32)>,
    pub closed: HashSet<RowKey>,
    pub selected: Option<RowKey>,
    scroll_to_selected: bool,
    /// The widths used in this frame: the stored widths fitted into the available width.
    widths: Vec<f32>,
    /// Columns whose text is shortened in the middle ("BROKER_DC…3PT10M"): ids that differ at the end.
    pub middle_truncate: Vec<usize>,
}

const ROW_HEIGHT: f32 = 26.0;
const INDENT: f32 = 18.0;
const MIN_COLUMN_WIDTH: f32 = 70.0;

impl TreeTable {
    pub fn new(id_source: &str, columns: &[(&str, f32)]) -> Self {
        TreeTable {
            id: egui::Id::unique(id_source),
            columns: columns.iter().map(|(name, width)| (name.to_string(), *width)).collect(),
            closed: HashSet::new(),
            selected: None,
            scroll_to_selected: false,
            widths: Vec::new(),
            middle_truncate: Vec::new(),
        }
    }

    /// Selects a row and scrolls to it on the next frame; its ancestors are opened.
    pub fn select(&mut self, key: Option<RowKey>, rows: &[TreeRow]) {
        self.selected = key;
        self.scroll_to_selected = key.is_some();
        if let Some(key) = key {
            for ancestor in ancestors(rows, key) {
                self.closed.remove(&ancestor);
            }
        }
    }

    /// Shows the tree in the available space. `context_menu` is called with the row under the mouse (or None for
    /// the empty area) when the user right-clicks; that row is selected first.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        rows: &[TreeRow],
        mut context_menu: impl FnMut(&mut Ui, Option<RowKey>),
    ) -> TreeEvents {
        let mut events = TreeEvents::default();
        let visible = self.visible_rows(rows);
        let font = TextStyle::Body.resolve(ui.style());
        let header_font = FontId { size: font.size + 2.0, ..font.clone() };
        let visuals = ui.visuals().clone();
        // The last column takes the remaining width, so its text ends with an ellipsis instead of running out of view
        let available = ui.available_width();
        self.widths = fit_widths(&self.columns.iter().map(|(_, width)| *width).collect::<Vec<_>>(), available);
        let total_width: f32 = self.widths.iter().sum();

        // The columns are fitted to the width: a horizontal scroll bar would only cost a row
        let hidden = egui::scroll_area::ScrollBarVisibility::AlwaysHidden;
        egui::ScrollArea::horizontal()
            .id_salt(self.id.with("h"))
            .auto_shrink([false, false])
            .scroll_bar_visibility(hidden)
            .show(ui, |ui| {
            ui.set_min_width(total_width);
            self.header(ui, total_width);
            // Keys belong to the tree when no text field has the focus (one tab is visible at a time)
            let tree_has_focus = ui.memory(|m| m.focused().is_none());

            // Keyboard: up and down move the selection, left and right close and open, Delete is reported
            if tree_has_focus {
                let position = self.selected.and_then(|key| visible.iter().position(|&i| rows[i].key == key));
                let (up, down, left, right, delete) = ui.input(|i| {
                    (
                        i.key_pressed(egui::Key::ArrowUp),
                        i.key_pressed(egui::Key::ArrowDown),
                        i.key_pressed(egui::Key::ArrowLeft),
                        i.key_pressed(egui::Key::ArrowRight),
                        i.key_pressed(egui::Key::Delete),
                    )
                });
                let next = match position {
                    Some(p) if up && p > 0 => Some(p - 1),
                    Some(p) if down && p + 1 < visible.len() => Some(p + 1),
                    None if (up || down) && !visible.is_empty() => Some(0),
                    _ => None,
                };
                if let Some(next) = next {
                    let key = rows[visible[next]].key;
                    self.selected = Some(key);
                    self.scroll_to_selected = true;
                    events.selected = Some(key);
                }
                if let Some(key) = self.selected {
                    if left {
                        self.closed.insert(key);
                    }
                    if right {
                        self.closed.remove(&key);
                    }
                    if delete {
                        events.delete_pressed = true;
                    }
                }
            }

            egui::ScrollArea::vertical().id_salt(self.id.with("v")).auto_shrink([false, false]).show_rows(
                ui,
                ROW_HEIGHT,
                visible.len(),
                |ui, range| {
                    let first_visible = range.start;
                    for &index in &visible[range] {
                        let row = &rows[index];
                        let has_children = rows.get(index + 1).is_some_and(|next| next.depth > row.depth);
                        let (rect, response) =
                            ui.allocate_exact_size(vec2(total_width.max(ui.available_width()), ROW_HEIGHT), Sense::click());
                        let selected = self.selected == Some(row.key);

                        // Background: selection, else the row's own colour, else alternating stripes
                        let background = if selected {
                            Some(visuals.selection.bg_fill)
                        } else if let Some(color) = row.background {
                            Some(color)
                        } else if row.header_bar.is_some() {
                            Some(visuals.widgets.noninteractive.bg_fill)
                        } else if index % 2 == 1 {
                            Some(visuals.faint_bg_color)
                        } else {
                            None
                        };
                        if let Some(color) = background {
                            ui.painter().rect_filled(rect, 0.0, color);
                        }
                        if let Some(bar) = row.header_bar {
                            let bar_rect = Rect::from_min_size(rect.min, vec2(3.0, ROW_HEIGHT));
                            ui.painter().rect_filled(bar_rect, 0.0, bar);
                        }
                        let text_color = if selected {
                            visuals.selection.stroke.color
                        } else if let Some(color) = row.text_color {
                            color
                        } else if row.background.is_some() {
                            // Coloured rows have a light background, also in dark mode
                            Color32::BLACK
                        } else if row.header_bar.is_some() {
                            visuals.strong_text_color()
                        } else {
                            visuals.text_color()
                        };

                        // The open/close triangle of the tree column
                        let mut x = rect.left() + 4.0 + row.depth as f32 * INDENT;
                        let mut toggled = false;
                        if has_children {
                            let triangle = Rect::from_min_size(pos2(x, rect.top()), vec2(INDENT, ROW_HEIGHT));
                            let open = !self.closed.contains(&row.key);
                            let arrow = if open { crate::theme::Arrow::Down } else { crate::theme::Arrow::Right };
                            crate::theme::paint_arrow(ui.painter(), triangle.center(), 8.0, arrow, text_color);
                            if response.clicked()
                                && response.interact_pointer_pos().is_some_and(|p| triangle.contains(p))
                            {
                                toggled = true;
                                if open {
                                    self.closed.insert(row.key);
                                } else {
                                    self.closed.remove(&row.key);
                                }
                            }
                        }
                        x += INDENT;
                        if let Some((right, color)) = row.icon {
                            let color = if selected { text_color } else { color };
                            crate::theme::paint_direction(ui.painter(), pos2(x + 7.0, rect.center().y), 13.0, right, color);
                            x += 20.0;
                        }

                        // The cells, each clipped to its column
                        let mut column_left = rect.left();
                        for (column, width) in self.widths.iter().enumerate() {
                            let text = row.cells.get(column).map(String::as_str).unwrap_or("");
                            let start = if column == 0 { x } else { column_left + 6.0 };
                            let clip = Rect::from_min_max(
                                pos2(start, rect.top()),
                                pos2(column_left + width - 4.0, rect.bottom()),
                            );
                            let painter = ui.painter().with_clip_rect(clip.intersect(ui.clip_rect()));
                            let font = if row.header_bar.is_some() && column == 0 { header_font.clone() } else { font.clone() };
                            // Only the first line: values can be multi-line (a KeyInfo)
                            // Only the first line, with runs of whitespace (XML indentation) shown as one space
                            let mut line = text.lines().next().unwrap_or("").split_whitespace().collect::<Vec<_>>().join(" ");
                            if self.middle_truncate.contains(&column) {
                                line = shorten_middle(&painter, &line, &font, clip.width());
                            }
                            let line = line.as_str();
                            let mut job = egui::text::LayoutJob::default();
                            let own_color = row.cell_colors.get(column).copied().flatten();
                            let badge = own_color.filter(|_| row.badge_columns.contains(&column));
                            let cell_color = match (badge, own_color) {
                                (Some(color), _) => color,
                                (None, Some(color)) if !selected => color,
                                _ => text_color,
                            };
                            let mut format = egui::TextFormat { font_id: font.clone(), color: cell_color, ..Default::default() };
                            if row.strong && column == 0 {
                                format.font_id = FontId { family: egui::FontFamily::Proportional, ..font.clone() };
                                format.color = if selected { text_color } else { visuals.strong_text_color() };
                            }
                            job.append(line, 0.0, format.clone());
                            if column == 0 {
                                if let Some(secondary) = &row.secondary {
                                    let weak = if selected { text_color } else { visuals.weak_text_color() };
                                    job.append(secondary, 8.0, egui::TextFormat { color: weak, ..format });
                                }
                            }
                            // Text that does not fit ends with an ellipsis; the tooltip has all of it
                            job.wrap = egui::text::TextWrapping::truncate_at_width((clip.width()).max(0.0));
                            let galley = painter.layout_job(job);
                            let mut position = pos2(start, rect.center().y - galley.size().y / 2.0);
                            if let Some(color) = badge {
                                position.x += 8.0;
                                let pill = Rect::from_min_size(
                                    pos2(start, rect.center().y - 10.0),
                                    vec2(galley.size().x + 16.0, 20.0),
                                );
                                let fill = if selected { visuals.extreme_bg_color } else { color.gamma_multiply(0.18) };
                                painter.rect(pill, 10.0, fill, Stroke::new(1.0, color), egui::StrokeKind::Inside);
                            }
                            // The default fonts have no bold face: a header row is drawn twice, half a pixel apart
                            if row.header_bar.is_some() && column == 0 {
                                painter.galley(position + vec2(0.6, 0.0), galley.clone(), text_color);
                            }
                            painter.galley(position, galley, text_color);
                            column_left += width;
                        }

                        let response = response.on_hover_text_at_pointer(full_text(row));
                        if !toggled && (response.clicked() || response.secondary_clicked()) {
                            if self.selected != Some(row.key) {
                                events.selected = Some(row.key);
                            }
                            self.selected = Some(row.key);
                            ui.memory_mut(|m| m.stop_text_input());
                        }
                        if response.double_clicked() {
                            let column = column_at(&self.widths, rect.left(), response.interact_pointer_pos());
                            events.double_clicked = Some((row.key, column));
                        }
                        response.context_menu(|ui| context_menu(ui, Some(row.key)));
                        if selected && self.scroll_to_selected {
                            ui.scroll_to_rect(rect, Some(egui::Align::Center));
                            self.scroll_to_selected = false;
                        }
                    }
                    // The empty area below the rows also has a context menu
                    let rest = ui.available_rect_before_wrap();
                    if rest.height() > 0.0 {
                        let response = ui.allocate_rect(rest, Sense::click());
                        response.context_menu(|ui| context_menu(ui, None));
                    }
                    // The group (partner) of the rows in view stays at the top once its own row scrolled away
                    let sticky = visible[..first_visible.min(visible.len())]
                        .iter()
                        .rev()
                        .find(|&&i| rows[i].header_bar.is_some())
                        .filter(|_| first_visible > 0);
                    if let Some(&index) = sticky {
                        let row = &rows[index];
                        let clip = ui.clip_rect();
                        let rect = Rect::from_min_size(clip.min, vec2(clip.width(), ROW_HEIGHT));
                        let painter = ui.painter().with_clip_rect(clip);
                        painter.rect_filled(rect, 0.0, visuals.panel_fill);
                        painter.rect_filled(rect, 0.0, visuals.widgets.noninteractive.bg_fill);
                        if let Some(bar) = row.header_bar {
                            painter.rect_filled(Rect::from_min_size(rect.min, vec2(3.0, ROW_HEIGHT)), 0.0, bar);
                        }
                        painter.line_segment(
                            [rect.left_bottom(), rect.right_bottom()],
                            visuals.widgets.noninteractive.bg_stroke,
                        );
                        let mut job = egui::text::LayoutJob::default();
                        let strong = egui::TextFormat {
                            font_id: header_font.clone(),
                            color: visuals.strong_text_color(),
                            ..Default::default()
                        };
                        job.append(row.cells.first().map(String::as_str).unwrap_or(""), 0.0, strong.clone());
                        if let Some(secondary) = &row.secondary {
                            job.append(secondary, 8.0, egui::TextFormat { color: visuals.weak_text_color(), ..strong });
                        }
                        let galley = painter.layout_job(job);
                        let position = pos2(
                            rect.left() + 4.0 + row.depth as f32 * INDENT + INDENT,
                            rect.center().y - galley.size().y / 2.0,
                        );
                        painter.galley(position + vec2(0.6, 0.0), galley.clone(), visuals.strong_text_color());
                        painter.galley(position, galley, visuals.strong_text_color());
                    }
                },
            );
        });
        events
    }

    fn header(&mut self, ui: &mut Ui, total_width: f32) {
        let (rect, _) = ui.allocate_exact_size(vec2(total_width.max(ui.available_width()), ROW_HEIGHT), Sense::hover());
        let visuals = ui.visuals();
        ui.painter().rect_filled(rect, 0.0, visuals.widgets.noninteractive.bg_fill);
        let font = TextStyle::Button.resolve(ui.style());
        let mut left = rect.left();
        let color = visuals.strong_text_color();
        let stroke = Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color);
        for column in 0..self.columns.len() {
            let width = self.widths[column];
            let clip = Rect::from_min_max(pos2(left, rect.top()), pos2(left + width - 4.0, rect.bottom()));
            let mut job = egui::text::LayoutJob::simple_singleline(self.columns[column].0.clone(), font.clone(), color);
            job.wrap = egui::text::TextWrapping::truncate_at_width((width - 10.0).max(0.0));
            let painter = ui.painter().with_clip_rect(clip);
            let galley = painter.layout_job(job);
            painter.galley(pos2(left + 6.0, rect.center().y - galley.size().y / 2.0), galley, color);
            left += width;
            // A handle on the right border to drag the column wider or narrower
            let handle = Rect::from_center_size(pos2(left, rect.center().y), vec2(8.0, ROW_HEIGHT));
            let response: Response = ui.interact(handle, self.id.with(("column", column)), Sense::drag());
            if response.hovered() || response.dragged() {
                ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
            }
            if response.dragged() {
                self.columns[column].1 = (width + response.drag_delta().x).max(MIN_COLUMN_WIDTH);
                // The widths of the other columns stay as they are shown
                for (other, shown) in self.widths.iter().enumerate() {
                    if other != column {
                        self.columns[other].1 = *shown;
                    }
                }
            }
            ui.painter().vline(left, rect.y_range(), stroke);
        }
    }

    fn visible_rows(&self, rows: &[TreeRow]) -> Vec<usize> {
        let mut visible = Vec::with_capacity(rows.len());
        let mut hidden_below: Option<usize> = None;
        for (index, row) in rows.iter().enumerate() {
            if let Some(depth) = hidden_below {
                if row.depth > depth {
                    continue;
                }
                hidden_below = None;
            }
            visible.push(index);
            if self.closed.contains(&row.key) {
                hidden_below = Some(row.depth);
            }
        }
        visible
    }
}

fn full_text(row: &TreeRow) -> String {
    let mut cells: Vec<String> = row.cells.iter().filter(|cell| !cell.is_empty()).cloned().collect();
    if let (Some(secondary), Some(first)) = (&row.secondary, cells.first_mut()) {
        first.push_str(" · ");
        first.push_str(secondary);
    }
    cells.join("\n")
}

/// Shortens text that is wider than `width` in the middle, keeping the start and the end.
fn shorten_middle(painter: &egui::Painter, text: &str, font: &FontId, width: f32) -> String {
    let measure = |s: &str| painter.layout_no_wrap(s.to_string(), font.clone(), Color32::WHITE).size().x;
    if measure(text) <= width {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let mut keep = chars.len().saturating_sub(1);
    while keep > 2 {
        let head: String = chars[..keep.div_ceil(2)].iter().collect();
        let tail: String = chars[chars.len() - keep / 2..].iter().collect();
        let candidate = format!("{head}…{tail}");
        if measure(&candidate) <= width {
            return candidate;
        }
        keep -= 1;
    }
    "…".to_string()
}

/// Fits the column widths into the available width: the last column takes what is left (at least
/// `MIN_LAST_WIDTH`); when that is not enough, the other columns shrink in proportion, each to at least
/// `MIN_COLUMN_WIDTH`.
fn fit_widths(widths: &[f32], available: f32) -> Vec<f32> {
    const MIN_LAST_WIDTH: f32 = 160.0;
    let Some((_, others)) = widths.split_last() else { return Vec::new() };
    let mut fitted: Vec<f32> = others.to_vec();
    let sum: f32 = fitted.iter().sum();
    if sum + MIN_LAST_WIDTH > available && sum > 0.0 {
        let scale = ((available - MIN_LAST_WIDTH) / sum).clamp(0.0, 1.0);
        for width in &mut fitted {
            *width = (*width * scale).max(MIN_COLUMN_WIDTH);
        }
    }
    let sum: f32 = fitted.iter().sum();
    fitted.push((available - sum).max(MIN_LAST_WIDTH));
    fitted
}

fn column_at(widths: &[f32], left: f32, pointer: Option<egui::Pos2>) -> usize {
    let Some(pointer) = pointer else { return 0 };
    let mut right = left;
    for (column, width) in widths.iter().enumerate() {
        right += width;
        if pointer.x < right {
            return column;
        }
    }
    widths.len().saturating_sub(1)
}

/// The keys of the ancestors of a row.
pub fn ancestors(rows: &[TreeRow], key: RowKey) -> Vec<RowKey> {
    let Some(position) = rows.iter().position(|row| row.key == key) else { return Vec::new() };
    let mut depth = rows[position].depth;
    let mut result = Vec::new();
    for row in rows[..position].iter().rev() {
        if row.depth < depth {
            result.push(row.key);
            depth = row.depth;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<TreeRow> {
        vec![
            TreeRow::new(1, 0, vec!["a".into()]),
            TreeRow::new(2, 1, vec!["b".into()]),
            TreeRow::new(3, 2, vec!["c".into()]),
            TreeRow::new(4, 1, vec!["d".into()]),
            TreeRow::new(5, 0, vec!["e".into()]),
        ]
    }

    #[test]
    fn closed_row_hides_its_descendants_only() {
        let mut tree = TreeTable::new("t", &[("Element", 100.0)]);
        tree.closed.insert(2);
        assert_eq!(tree.visible_rows(&rows()), vec![0, 1, 3, 4]);
        tree.closed.insert(1);
        assert_eq!(tree.visible_rows(&rows()), vec![0, 4]);
    }

    #[test]
    fn widths_fit_the_available_width() {
        // Enough room: the last column takes the rest
        assert_eq!(fit_widths(&[100.0, 100.0, 50.0], 500.0), vec![100.0, 100.0, 300.0]);
        // Too little room: the others shrink so the last column keeps its minimum
        let fitted = fit_widths(&[200.0, 200.0, 50.0], 400.0);
        assert_eq!(fitted.iter().sum::<f32>(), 400.0);
        assert_eq!(fitted[2], 160.0);
    }

    #[test]
    fn select_opens_ancestors() {
        let mut tree = TreeTable::new("t", &[("Element", 100.0)]);
        tree.closed.extend([1, 2]);
        tree.select(Some(3), &rows());
        assert!(tree.closed.is_empty());
        assert_eq!(ancestors(&rows(), 3), vec![2, 1]);
    }
}

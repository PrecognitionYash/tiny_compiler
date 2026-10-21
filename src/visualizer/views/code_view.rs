use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Vec2};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CodeKind {
    Source,
    Assembly,
}

pub struct CodePane<'a> {
    pub title: &'a str,
    pub text: &'a str,
    pub kind: CodeKind,
}

const FONT_SIZE: f32 = 13.0;
const CHAR_W: f32 = 7.85;
const LINE_H: f32 = 20.0;
const GUTTER: f32 = 48.0;

fn palette(kind: CodeKind, text: &str, in_comment: bool) -> Color32 {
    if in_comment {
        return Color32::from_rgb(117, 143, 132);
    }
    if matches!(text, "def" | "return" | "if" | "else" | "for" | "in") {
        return Color32::from_rgb(199, 176, 255);
    }
    if text == "print" {
        return Color32::from_rgb(102, 220, 224);
    }
    if text.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return Color32::from_rgb(250, 204, 110);
    }
    if matches!(
        text,
        "mov"
            | "push"
            | "pop"
            | "call"
            | "ret"
            | "sub"
            | "add"
            | "cmp"
            | "jmp"
            | "je"
            | "jne"
            | "lea"
            | "imul"
            | "idiv"
            | "and"
            | "or"
            | "xor"
            | "sar"
            | "shl"
            | "setl"
            | "setle"
            | "setg"
            | "setge"
            | "sete"
            | "setne"
    ) {
        return Color32::from_rgb(130, 190, 255);
    }
    if matches!(
        text,
        "+" | "-"
            | "*"
            | "/"
            | "&"
            | "|"
            | "^"
            | "<<"
            | ">>"
            | "="
            | "=="
            | "!="
            | "<"
            | ">"
            | "<="
            | ">="
            | ":"
            | "("
            | ")"
            | ","
    ) {
        return Color32::from_rgb(244, 145, 190);
    }
    match kind {
        CodeKind::Source => Color32::from_rgb(222, 229, 240),
        CodeKind::Assembly => Color32::from_rgb(211, 224, 240),
    }
}

fn segments(line: &str, kind: CodeKind) -> Vec<(String, Color32)> {
    let line = line.replace('\t', "    ");
    if kind == CodeKind::Source {
        let hash = line.find('#');
        let slash = line.find("//");
        let comment_start = match (hash, slash) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) | (None, Some(a)) => Some(a),
            (None, None) => None,
        };
        if let Some(comment_start) = comment_start {
            let mut out = tokenize(&line[..comment_start], kind);
            out.push((
                line[comment_start..].to_string(),
                Color32::from_rgb(117, 143, 132),
            ));
            return out;
        }
    } else if line.trim_start().starts_with('.') || line.trim_start().ends_with(':') {
        return vec![(line, Color32::from_rgb(120, 210, 190))];
    }
    tokenize(&line, kind)
}

fn tokenize(line: &str, kind: CodeKind) -> Vec<(String, Color32)> {
    let chars: Vec<char> = line.chars().collect();
    let mut result = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        let start = i;
        if chars[i].is_whitespace() {
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
        } else if chars[i].is_ascii_alphanumeric()
            || chars[i] == '_'
            || chars[i] == '.'
            || chars[i] == '$'
        {
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric()
                    || chars[i] == '_'
                    || chars[i] == '.'
                    || chars[i] == '$')
            {
                i += 1;
            }
        } else if i + 1 < chars.len()
            && matches!(
                (chars[i], chars[i + 1]),
                ('<', '<') | ('>', '>') | ('=', '=') | ('!', '=')
            )
        {
            i += 2;
        } else {
            i += 1;
        }
        let text: String = chars[start..i].iter().collect();
        let color = if text.chars().all(char::is_whitespace) {
            Color32::from_rgb(222, 229, 240)
        } else {
            palette(kind, text.trim(), false)
        };
        result.push((text, color));
    }
    result
}

fn paint_line(ui: &mut egui::Ui, pane: Rect, line_no: usize, line: &str, kind: CodeKind) {
    let painter = ui.painter_at(pane);
    let bg = if line_no.is_multiple_of(2) {
        Color32::from_rgb(23, 29, 40)
    } else {
        Color32::from_rgb(20, 26, 36)
    };
    painter.rect_filled(pane, egui::CornerRadius::same(0), bg);
    painter.text(
        Pos2::new(pane.left() + 8.0, pane.top() + 2.0),
        Align2::LEFT_TOP,
        format!("{:>4}", line_no + 1),
        FontId::monospace(FONT_SIZE - 1.0),
        Color32::from_rgb(107, 122, 143),
    );
    let mut x = pane.left() + GUTTER;
    let y = pane.top() + 2.0;
    for (text, color) in segments(line, kind) {
        if !text.is_empty() {
            painter.text(
                Pos2::new(x, y),
                Align2::LEFT_TOP,
                &text,
                FontId::monospace(FONT_SIZE),
                color,
            );
            x += text.chars().count() as f32 * CHAR_W;
        }
    }
}

pub fn show_comparison(
    ui: &mut egui::Ui,
    id: &'static str,
    left: CodePane<'_>,
    right: CodePane<'_>,
) {
    let left_lines: Vec<&str> = left.text.lines().collect();
    let right_lines: Vec<&str> = right.text.lines().collect();
    let max_chars = left_lines
        .iter()
        .chain(right_lines.iter())
        .map(|line| line.replace('\t', "    ").chars().count())
        .max()
        .unwrap_or(32);
    let min_pane = (ui.available_width() * 0.5).max(360.0);
    let pane_width = min_pane.max(GUTTER + max_chars as f32 * CHAR_W + 24.0);
    let canvas_width = pane_width * 2.0;
    ui.horizontal(|ui| {
        ui.add_sized(
            Vec2::new(pane_width, 28.0),
            egui::Label::new(
                egui::RichText::new(left.title)
                    .strong()
                    .color(Color32::from_rgb(143, 190, 255)),
            ),
        );
        ui.add_sized(
            Vec2::new(pane_width, 28.0),
            egui::Label::new(
                egui::RichText::new(right.title)
                    .strong()
                    .color(Color32::from_rgb(104, 224, 198)),
            ),
        );
    });
    egui::Frame::new()
        .fill(Color32::from_rgb(17, 22, 31))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(51, 62, 80)))
        .corner_radius(egui::CornerRadius::same(10))
        .show(ui, |ui| {
            egui::ScrollArea::both()
                .id_salt((id, "shared-code-scroll"))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let rows = left_lines.len().max(right_lines.len()).max(1);
                    let (whole, _) = ui.allocate_exact_size(
                        Vec2::new(canvas_width, rows as f32 * LINE_H),
                        egui::Sense::hover(),
                    );
                    for index in 0..rows {
                        let y = whole.top() + index as f32 * LINE_H;
                        let left_rect = Rect::from_min_size(
                            Pos2::new(whole.left(), y),
                            Vec2::new(pane_width, LINE_H),
                        );
                        let right_rect = Rect::from_min_size(
                            Pos2::new(whole.left() + pane_width, y),
                            Vec2::new(pane_width, LINE_H),
                        );
                        paint_line(
                            ui,
                            left_rect,
                            index,
                            left_lines.get(index).copied().unwrap_or(""),
                            left.kind,
                        );
                        paint_line(
                            ui,
                            right_rect,
                            index,
                            right_lines.get(index).copied().unwrap_or(""),
                            right.kind,
                        );
                    }
                    ui.painter().line_segment(
                        [
                            Pos2::new(whole.left() + pane_width, whole.top()),
                            Pos2::new(whole.left() + pane_width, whole.bottom()),
                        ],
                        egui::Stroke::new(1.0, Color32::from_rgb(65, 78, 101)),
                    );
                });
        });
}

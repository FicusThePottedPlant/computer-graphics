mod logic;

use eframe::egui;
use eframe::egui::{Color32, Pos2};
use egui_extras::{Column, TableBuilder};
use epaint::{pos2, Stroke, vec2};
use logic::utils::cut;
use logic::windows::ErrorWindow;
use std::{
    sync::{Arc, Mutex},
    thread,
};


#[derive(Debug)]
struct MyApp {
    background: egui::Color32,
    line_color: egui::Color32,
    cutter_color: egui::Color32,
    res_color: egui::Color32,
    res_width: u32,
    error: ErrorWindow,

    buf_x_l: String,
    buf_y_l: String,
    buf_x_r: String,
    buf_y_r: String,

    buf_x1: String,
    buf_y1: String,
    buf_x2: String,
    buf_y2: String,

    cutter: (Option<Pos2>, Option<Pos2>),
    buf_line: (Option<Pos2>, Option<Pos2>),
    cut_lines: Vec<(Pos2, Pos2)>,
    lines: Vec<(Pos2, Pos2)>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            background: Color32::WHITE,
            line_color: Color32::DARK_BLUE,
            cutter_color: Color32::DARK_GRAY,
            res_color: Color32::RED,
            res_width: 2,
            error: Default::default(),
            buf_x_l: "".to_string(),
            buf_y_l: "".to_string(),
            buf_x_r: "".to_string(),
            buf_y_r: "".to_string(),

            buf_x1: "".to_string(),
            buf_y1: "".to_string(),
            buf_x2: "".to_string(),
            buf_y2: "".to_string(),

            buf_line: (None, None),
            cut_lines: vec![],
            cutter: (None, None),
            lines: vec![],
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.error.enabled() {
            self.error.update(ctx);
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(!self.error.enabled());
            self.ui(ui);
        });
    }
}

impl MyApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(950.0);
                ui.set_height(650.0);
                self.painter(ui);
            });
            self.control(ui);
        });
    }

    fn control(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Цвет фона");
                use egui::color_picker::{color_edit_button_srgba, Alpha};
                color_edit_button_srgba(ui, &mut self.background, Alpha::Opaque);
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Цвет отсекателя");
                use egui::color_picker::{color_edit_button_srgba, Alpha};

                color_edit_button_srgba(ui, &mut self.cutter_color, Alpha::Opaque);
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Цвет отрезка");
                use egui::color_picker::{color_edit_button_srgba, Alpha};
                color_edit_button_srgba(ui, &mut self.line_color, Alpha::Opaque);
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Цвет отсеченного отрезка");
                use egui::color_picker::{color_edit_button_srgba, Alpha};
                color_edit_button_srgba(ui, &mut self.res_color, Alpha::Opaque);
            });

            ui.separator();
            ui.vertical(|ui| {
                ui.label("Толщина отсеченного отрезка");
                ui.add(egui::Slider::new(&mut self.res_width, 1..=5).suffix(" пикс."));
            });

            ui.collapsing("Установка отсекателя", |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.buf_x_l).hint_text("X левый: "));
                    ui.add(egui::TextEdit::singleline(&mut self.buf_y_l).hint_text("Y верхний: "));
                });
                ui.vertical_centered_justified(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.buf_x_r).hint_text("X правый: "));
                    ui.add(egui::TextEdit::singleline(&mut self.buf_y_r).hint_text("Y нижний: "));
                });
                if ui.button("Установить").clicked() {
                    self.set_cutter();
                }
            });

            ui.collapsing("Добавление отрезка", |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.buf_x1).hint_text("X1: "));
                    ui.add(egui::TextEdit::singleline(&mut self.buf_y1).hint_text("Y1: "));
                });
                ui.vertical_centered_justified(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.buf_x2).hint_text("X2: "));
                    ui.add(egui::TextEdit::singleline(&mut self.buf_y2).hint_text("Y2: "));
                });
                if ui.button("Добавить").clicked() {
                    self.add_line();
                }
            });

            ui.vertical_centered_justified(|ui| {
                ui.separator();
                if ui.button("Отсечь").clicked() {
                    self.cut();
                }
                if ui.button("Очистка").clicked() {
                    self.clear();
                }
            });
            ui.separator();
            ui.vertical(|ui| {
                ui.label("Управление:");
                ui.label("Отсекатель - ЛКМ");
                ui.label("Прямая - ПКМ");
                ui.label("Прямая - SHIFT+ПКМ");
                if let (Some(Pos2 { x: x1, y: y1 }), Some(Pos2 { x: x2, y: y2 })) = self.cutter {
                    ui.label(format!("Отсекатель ({:} {:}), ({:} {:})", x1 as i32, y1 as i32, x2 as i32, y2 as i32));
                }
            });
        });
    }
}

impl MyApp {
    fn painter(&mut self, ui: &mut egui::Ui) {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        painter.rect(
            painter.clip_rect().shrink(0.0),
            0.0,
            self.background,
            egui::Stroke::new(0.5, egui::Color32::BLACK),
        );
        let p_rect = response.rect;

        let to_screen = egui::emath::RectTransform::from_to(
            egui::Rect::from_min_size(egui::Pos2::ZERO, p_rect.size()),
            response.rect.translate(egui::Pos2::ZERO.to_vec2()),
        );
        let ppp = ui.ctx().pixels_per_point();
        let unit = ppp.recip();

        if response.clicked_by(egui::PointerButton::Primary) {
            let mouse_pos = response.hover_pos().unwrap_or_default();
            let pos1 = mouse_pos + [-6.0, -5.0].into();
            let pos1 = to_screen.transform_pos((pos1 / unit).round());
            if let (Some(mut left), None) = self.cutter {
                let mut pos = pos1 * unit;
                if left.x > pos.x {
                    std::mem::swap(&mut left, &mut pos);
                }
                if left.y > pos.y {
                    std::mem::swap(&mut left.y, &mut pos.y);
                }
                self.cutter = (Some(left), Some(pos));
            } else if let (Some(_), Some(_)) = self.cutter {
                if let (Some(_), None) = self.buf_line {} else {
                    self.cutter = (Some(pos1 * unit), None);
                }
            } else if let (None, None) = self.cutter {
                self.cutter = (Some(pos1 * unit), None);
            }
        }

        if response.clicked_by(egui::PointerButton::Secondary) {
            let mouse_pos = response.hover_pos().unwrap_or_default();
            let pos1 = mouse_pos + [-6.0, -5.0].into();
            let pos1 = to_screen.transform_pos((pos1 / unit).round());
            if let (Some(left), None) = self.buf_line {
                let mut pos = pos1 * unit;
                if ui.input(|ui| ui.modifiers.matches_logically(egui::Modifiers::SHIFT)) {
                    let dx = (left.x - pos.x).abs();
                    let dy = (left.y - pos.y).abs();
                    if dy > dx {
                        pos.x = left.x;
                    } else {
                        pos.y = left.y;
                    }
                }
                self.buf_line = (None, None);
                self.lines.push((left, pos));
            } else if let (Some(_), Some(_)) = self.cutter {
                self.buf_line = (Some(pos1 * unit), None);
            } else if let (None, None) = self.cutter {
                self.buf_line = (Some(pos1 * unit), None);
            }
        }

        if response.hovered {
            let mouse_pos = response.hover_pos().unwrap_or_default();
            let pos1 = mouse_pos + [-6.0, -5.0].into();
            let pos1 = to_screen.transform_pos((pos1 / unit).round());
            if let (Some(mut left), None) = self.cutter {
                let mut pos = pos1 * unit;
                if left.x > pos.x {
                    std::mem::swap(&mut left, &mut pos);
                }

                if left.y > pos.y {
                    std::mem::swap(&mut left.y, &mut pos.y);
                }
                painter.rect_stroke(
                    [left, pos].into(),
                    1.0, egui::Stroke::new(unit, self.cutter_color));
            } else if let (Some(mut left), None) = self.buf_line {
                let mut pos = pos1 * unit;
                if ui.input(|ui| ui.modifiers.matches_logically(egui::Modifiers::SHIFT)) {
                    let dx = (left.x - pos.x).abs();
                    let dy = (left.y - pos.y).abs();
                    if dy > dx {
                        pos.x = left.x;
                    } else {
                        pos.y = left.y;
                    }
                }
                painter.line_segment(
                    [left, pos].into(),
                    egui::Stroke::new(unit, self.line_color));
            }
        }

        if let (Some(mut left), Some(mut right)) = self.cutter {
            painter.rect_stroke(
                [left, right].into(),
                1.0, egui::Stroke::new(unit, self.cutter_color));
        }

        painter.extend(self.lines.iter().map(|&(a, b)| {
            egui::Shape::line_segment(
                [a, b],
                egui::Stroke::new(unit, self.line_color))
        })
        );
        painter.extend(self.cut_lines.iter().map(|&(a, b)| {
            egui::Shape::line_segment(
                [a, b],
                egui::Stroke::new(self.res_width as f32 * unit, self.res_color))
        })
        );
    }
}

impl MyApp {
    fn cut(&mut self) {
        if let (Some(a), Some(b)) = self.cutter {
            self.cut_lines = cut((a, b), &self.lines);
        }
    }

    fn clear(&mut self) {
        self.lines.clear();
        self.cut_lines.clear();
        self.cutter = (None, None);
    }
    fn set_cutter(&mut self) {
        if let (Ok(mut x1), Ok(mut y1), Ok(mut x2), Ok(mut y2)) = (
            self.parse_field::<u32>(self.buf_x_l.clone()),
            self.parse_field::<u32>(self.buf_y_l.clone()),
            self.parse_field::<u32>(self.buf_x_r.clone()),
            self.parse_field::<u32>(self.buf_y_r.clone()),
        ) {
            if x1 > x2 {
                std::mem::swap(&mut x1, &mut x2);
            }
            if y1 > y1 {
                std::mem::swap(&mut y1, &mut y2);
            }
            self.cutter = (Some(pos2(x1 as f32, y1 as f32)), Some(pos2(x2 as f32, y2 as f32)));
        } else {
            self.error.enable();
        }
    }

    fn add_line(&mut self) {
        if let (Ok(x1), Ok(y1), Ok(x2), Ok(y2)) = (
            self.parse_field::<u32>(self.buf_x1.clone()),
            self.parse_field::<u32>(self.buf_y1.clone()),
            self.parse_field::<u32>(self.buf_x2.clone()),
            self.parse_field::<u32>(self.buf_y2.clone()),
        ) {
            self.lines.push((pos2(x1 as f32, y1 as f32), pos2(x2 as f32, y2 as f32)));
        } else {
            self.error.enable();
        }
    }
}

// parsing
impl MyApp {
    fn parse_field<T>(&mut self, field: String) -> Result<T, ()>
        where
            T: std::str::FromStr,
    {
        field.parse::<T>().map_err(|_| {
            self.error
                .set_error(
                    "Ошибка".to_string(),
                    format!("Ошибочное значение {}", field),
                )
                .enable()
        })
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((1280.0, 1024.0)),
        default_theme: eframe::Theme::Light,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "Лабораторная работа 7",
        native_options,
        Box::new(|_| Box::<MyApp>::default()),
    )
}

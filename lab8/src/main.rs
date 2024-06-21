mod logic;

use eframe::egui;
use eframe::egui::{Color32, Pos2};
use epaint::{pos2};
use logic::utils::{cut, Polygon};
use logic::windows::ErrorWindow;

fn angle(a: Pos2, b: Pos2, c: Pos2, d: Pos2) -> f32 {
    let (x1, y1) = a.into();
    let (x2, y2) = b.into();
    let (x3, y3) = c.into();
    let (x4, y4) = d.into();
    let dx1 = x2 - x1;
    let dy1 = y2 - y1;
    let dx2 = x4 - x3;
    let dy2 = y4 - y3;

    let dot_product = dx1 * dx2 + dy1 * dy2;
    let magnitude1 = (dx1.powi(2) + dy1.powi(2)).sqrt();
    let magnitude2 = (dx2.powi(2) + dy2.powi(2)).sqrt();

    let cos_angle = dot_product / (magnitude1 * magnitude2);
    let angle_rad = cos_angle.acos();
    let angle_deg = angle_rad.to_degrees();

    if angle_deg > 90.0 {
        180.0 - angle_deg
    } else {
        angle_deg
    }
}


#[derive(Debug, PartialEq)]
enum State {
    POLY,
    LINE,
}

#[derive(Debug)]
struct MyApp {
    background: egui::Color32,
    line_color: egui::Color32,
    cutter_color: egui::Color32,
    res_color: egui::Color32,
    res_width: u32,
    error: ErrorWindow,
    state: State,

    buf_x_l: String,
    buf_y_l: String,

    buf_x1: String,
    buf_y1: String,
    buf_x2: String,
    buf_y2: String,

    cutter: Polygon,
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
            state: State::POLY,
            buf_x_l: "".to_string(),
            buf_y_l: "".to_string(),

            buf_x1: "".to_string(),
            buf_y1: "".to_string(),
            buf_x2: "".to_string(),
            buf_y2: "".to_string(),

            buf_line: (None, None),
            cut_lines: vec![],
            cutter: Default::default(),
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
            ui.radio_value(&mut self.state, State::POLY, "Отсекатель");
            ui.radio_value(&mut self.state, State::LINE, "Отрезок");
            ui.collapsing("Добавить вершину отсекателя", |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.buf_x_l).hint_text("X: "));
                    ui.add(egui::TextEdit::singleline(&mut self.buf_y_l).hint_text("Y: "));
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
                if ui.button("Замкнуть").clicked() {
                    self.cutter.close();
                }
                if ui.button("Очистить отрезки").clicked() {
                    self.cut_lines.clear();
                    self.lines.clear();
                }
            });
            ui.separator();
            ui.vertical(|ui| {
                ui.label("Управление:");
                ui.label("Ввод - нажатие левой кнопки");
                ui.label("Вертикальная/Горизонтальная Прямая - ЛЕВАЯ + SHIFT");
                ui.label("Параллельная Прямая - ЛЕВАЯ + ПРАВЫЙ CTRL");
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

        if response.clicked_by(egui::PointerButton::Primary) && self.state == State::POLY {
            if self.cutter.closed() {
                self.cutter.open();
            }
            let mouse_pos = response.hover_pos().unwrap_or_default();
            let pos1 = mouse_pos + [-6.0, -5.0].into();
            let pos1 = to_screen.transform_pos((pos1 / unit).round());
            let mut pos1 = pos1 * unit;
            if let Some(left) = self.cutter.last() {
                if ui.input(|ui| ui.modifiers.matches_logically(egui::Modifiers::SHIFT)) {
                    let dx = (left.x - pos1.x).abs();
                    let dy = (left.y - pos1.y).abs();
                    if dy > dx {
                        pos1.x = left.x;
                    } else {
                        pos1.y = left.y;
                    }
                }
            }

            self.cutter.push(pos1);
        }

        if response.clicked_by(egui::PointerButton::Secondary) {
            self.cutter.close();
        }

        if response.clicked_by(egui::PointerButton::Primary) && self.state == State::LINE {
            let mouse_pos = response.hover_pos().unwrap_or_default();
            let pos1 = mouse_pos + [-6.0, -5.0].into();
            let pos1 = to_screen.transform_pos((pos1 / unit).round());
            let mut pos1 = pos1 * unit;
            if let (Some(left), None) = self.buf_line {

                if ui.input(|ui| ui.modifiers.matches_logically(egui::Modifiers::SHIFT)) {
                    let dx = (left.x - pos1.x).abs();
                    let dy = (left.y - pos1.y).abs();
                    if dy > dx {
                        pos1.x = left.x;
                    } else {
                        pos1.y = left.y;
                    }
                }
                if ui.input(|ui| ui.modifiers.matches_logically(egui::Modifiers::CTRL)) {
                    if let Some([e0, e1]) = self.cutter.vertices().windows(2).min_by_key(|edge| {
                        angle(edge[0], edge[1], left, pos1).to_bits()
                    }) {
                        let edge_dx = e1.x - e0.x;
                        let edge_dy = e1.y - e0.y;

                        let line_dx = pos1.x - left.x;
                        let line_dy = pos1.y - left.y;

                        let edge_length = (edge_dx * edge_dx + edge_dy * edge_dy).sqrt();
                        let projection_length = (line_dx * edge_dx + line_dy * edge_dy) / edge_length;

                        pos1.x = left.x + (projection_length * edge_dx) / edge_length;
                        pos1.y = left.y + (projection_length * edge_dy) / edge_length;
                    }
                }
                self.buf_line = (None, None);
                self.lines.push((left, pos1));
            } else {
                self.buf_line = (Some(pos1), None);
            }
        }

        if response.hovered {
            let mouse_pos = response.hover_pos().unwrap_or_default();
            let pos1 = mouse_pos + [-6.0, -5.0].into();
            let pos1 = to_screen.transform_pos((pos1 / unit).round());
            let mut pos1 = pos1 * unit;
            match self.state {
                State::POLY => {
                    if !self.cutter.closed() {
                        if let Some(&last) = self.cutter.last() {
                            if ui.input(|ui| ui.modifiers.matches_logically(egui::Modifiers::SHIFT)) {
                                let dx = (last.x - pos1.x).abs();
                                let dy = (last.y - pos1.y).abs();
                                if dy > dx {
                                    pos1.x = last.x ;
                                } else {
                                    pos1.y = last.y;
                                }
                            }
                            painter.line_segment(
                                [last, pos1].into(),
                                egui::Stroke::new(unit, self.cutter_color),
                            );
                        }
                    }
                }
                State::LINE => {
                    match self.buf_line {
                        (Some(left), None) => {
                            if ui.input(|ui| ui.modifiers.matches_logically(egui::Modifiers::SHIFT)) {
                                let dx = (left.x - pos1.x).abs();
                                let dy = (left.y - pos1.y).abs();
                                if dy > dx {
                                    pos1.x = left.x;
                                } else {
                                    pos1.y = left.y;
                                }
                            }
                            if ui.input(|ui| ui.modifiers.matches_logically(egui::Modifiers::CTRL)) {
                                if let Some([e0, e1]) = self.cutter.vertices().windows(2).min_by_key(|edge| {
                                    angle(edge[0], edge[1], left, pos1).to_bits()
                                }) {
                                    let edge_dx = e1.x - e0.x;
                                    let edge_dy = e1.y - e0.y;

                                    let line_dx = pos1.x - left.x;
                                    let line_dy = pos1.y - left.y;

                                    let edge_length = (edge_dx * edge_dx + edge_dy * edge_dy).sqrt();
                                    let projection_length = (line_dx * edge_dx + line_dy * edge_dy) / edge_length;

                                    pos1.x = left.x + (projection_length * edge_dx) / edge_length;
                                    pos1.y = left.y + (projection_length * edge_dy) / edge_length;
                                }
                            }
                            painter.line_segment(
                                [left, pos1].into(),
                                egui::Stroke::new(unit, self.line_color),
                            );
                        }
                        _ => {}
                    }
                }
            }

        }

        painter.extend(self.lines.iter().map(|&(a, b)| {
            egui::Shape::line_segment([a, b], egui::Stroke::new(unit, self.line_color))
        }));
        painter.extend(self.cut_lines.iter().map(|&(a, b)| {
            egui::Shape::line_segment(
                [a, b],
                egui::Stroke::new(self.res_width as f32 * unit, self.res_color),
            )
        }));

        painter.extend(
            self.cutter
                .vertices()
                .windows(2)
                .map(|a| {
                    egui::Shape::line_segment(
                        [a[0], a[1]],
                        egui::Stroke::new(1., self.cutter_color),
                    )
                })
        );

        if self.cutter.closed() {
            painter.line_segment(
                [
                    self.cutter.vertices().last().unwrap().clone(),
                    self.cutter.vertices().first().unwrap().clone(),
                ],
                egui::Stroke::new(unit, self.cutter_color));
        }
    }
}

impl MyApp {
    fn cut(&mut self) {
        if !self.cutter.closed() {
            self.error
                .set_error(
                    "Ошибка".to_string(),
                    format!("Многоугольник не замкнут"),
                )
                .enable();
            return;
        }
        if let Some(x) = cut(self.cutter.vertices(), &self.lines) {
            self.cut_lines = x;
        } else {
            self.error
                .set_error(
                    "Ошибка".to_string(),
                    format!("Многоугольник не выпуклый"),
                )
                .enable()
        }


    }

    fn clear(&mut self) {
        self.lines.clear();
        self.cut_lines.clear();
        self.cutter.clear();
    }
    fn set_cutter(&mut self) {
        if let (Ok(x), Ok(y)) = (
            self.parse_field::<u32>(self.buf_x_l.clone()),
            self.parse_field::<u32>(self.buf_y_l.clone()),
        ) {
            if self.cutter.closed() {
                self.cutter.open();
            }
            self.cutter.push(Pos2 {x: x as f32, y: y as f32});

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
            self.lines
                .push((pos2(x1 as f32, y1 as f32), pos2(x2 as f32, y2 as f32)));
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
        "Лабораторная работа 8",
        native_options,
        Box::new(|_| Box::<MyApp>::default()),
    )
}

mod logic;

use eframe::egui;
use eframe::egui::Pos2;
use egui_extras::{Column, TableBuilder};
use logic::utils::Canvas;
use logic::windows::ErrorWindow;
use std::{
    sync::{Arc, Mutex},
    thread,
};
// use eframe::egui::CursorIcon::Default;

pub fn are_collinear(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    x4: f32,
    y4: f32,
) -> bool {
    let slope1 = (y2 - y1) / (x2 - x1);
    let slope2 = (y4 - y3) / (x4 - x3);
    slope1 == slope2 || (slope1.is_infinite() && slope2.is_infinite())
}

#[derive(Debug)]
struct MyApp {
    background: egui::Color32,
    stroke: egui::Color32,
    error: ErrorWindow,
    canvas: Arc<Mutex<Canvas>>,

    buf_x: String,
    buf_y: String,

    buf_dur: String,
    dur_res: Arc<Mutex<std::time::Duration>>,
    timeout: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            error: Default::default(),
            background: egui::Color32::WHITE,
            stroke: egui::Color32::RED,
            canvas: Arc::new(Mutex::new(Canvas::new())),
            buf_x: "".to_string(),
            buf_y: "".to_string(),
            buf_dur: "".to_string(),
            dur_res: Arc::new(Mutex::new(std::time::Duration::new(0, 0))),
            timeout: false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
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
                ui.label("Цвет фигуры");
                use egui::color_picker::{color_edit_button_srgba, Alpha};
                let mut canvas = self.canvas.lock().unwrap();
                color_edit_button_srgba(ui, &mut self.stroke, Alpha::Opaque);
                canvas.set_color(self.stroke);
            });

            ui.vertical_centered_justified(|ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.buf_x).hint_text("X: "));
                    ui.add(egui::TextEdit::singleline(&mut self.buf_y).hint_text("Y: "));
                });
                if ui.button("Добавить точку").clicked() {
                    self.add_point();
                }
            });
            self.update_table(ui);
            ui.vertical_centered_justified(|ui| {
                if ui.button("Замкнуть фигуру").clicked() {
                    self.close_figure();
                }
                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    ui.checkbox(&mut self.timeout, "Задержка");
                    if self.timeout {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.buf_dur)
                                .hint_text("Задержка (мс): "),
                        );
                    }
                });
                if ui.button("Залить фигуру").clicked() {
                    self.clean_figure();
                    self.fill_figure();
                }
                if ui.button("Очистить холст").clicked() {
                    self.clear_figure();
                }
                if ui.button("Очистить заливку").clicked() {
                    self.clean_figure();
                }

                ui.label(format!(
                    "Время заливки {:.5} сек.",
                    self.dur_res.lock().unwrap().as_secs_f64()
                ));
            });
        });
    }

    fn update_table(&self, ui: &mut egui::Ui) {
        let canvas = self.canvas.lock().unwrap();
        let table = TableBuilder::new(ui)
            .column(Column::remainder())
            .column(Column::remainder())
            .column(Column::remainder())
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .striped(true)
            .max_scroll_height(200.0)
            .stick_to_bottom(true)
            .drag_to_scroll(true)
            .resizable(false);
        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("№");
                });
                header.col(|ui| {
                    ui.strong("X");
                });
                header.col(|ui| {
                    ui.strong("Y");
                });
            })
            .body(|mut body| {
                let data = canvas.points();
                for (c, i) in data.iter().enumerate() {
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            if canvas.all_closed().contains(&c) {
                                ui.label("Фигура");
                            }
                            ui.label(format!("{}", c));
                        });
                        row.col(|ui| {
                            ui.label(format!("{}", i.x));
                        });
                        row.col(|ui| {
                            ui.label(format!("{}", i.y));
                        });
                    });
                }
            });
    }

    fn painter(&mut self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click());
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

        if response.clicked_by(egui::PointerButton::Secondary) {
            self.close_figure();
        }
        let mut canvas = self.canvas.lock().unwrap();

        if response.hovered() && canvas.last_closed() != canvas.points().len() {
            let mouse_pos = response.hover_pos().unwrap_or_default();
            let pos2 = mouse_pos + [-12.0, -15.0].into();
            let mut pos2 = to_screen.transform_pos((pos2 / unit).round());
            if let Some(pos1) = canvas.points().last() {
                if ui.input(|ui| ui.modifiers.matches_logically(egui::Modifiers::SHIFT)) {
                    let dx = (pos2.x - pos1.x).abs();
                    let dy = (pos2.y - pos1.y).abs();
                    if dy > dx {
                        pos2.x = pos1.x;
                    } else {
                        pos2.y = pos1.y;
                    }
                }
                if let Some(pos3) = canvas.last_closed_point() {
                    if (pos3.x - pos2.x).powi(2) + (pos3.y - pos2.y).powi(2) <= 81. {
                        painter.add(egui::Shape::circle_filled(
                            to_screen.transform_pos(*pos3 * unit),
                            4.,
                            egui::Color32::GRAY,
                        ));
                    }
                }

                for pos3 in canvas.points() {
                    if (pos3.x - pos2.x).abs() <= 2. {
                        painter.add(egui::Shape::dashed_line(
                            &[
                                to_screen.transform_pos(Pos2::new(pos3.x, 0.0) * unit),
                                to_screen.transform_pos(Pos2::new(pos2.x, 2000.0) * unit),
                            ],
                            egui::Stroke::new(unit / 2., egui::Color32::GRAY),
                            5.0,
                            2.0,
                        ));
                        painter.add(egui::Shape::circle_filled(
                            to_screen.transform_pos(*pos3 * unit),
                            4.,
                            egui::Color32::GRAY,
                        ));
                    }
                    if (pos3.y - pos2.y).abs() <= 2. {
                        painter.add(egui::Shape::dashed_line(
                            &[
                                to_screen.transform_pos(Pos2::new(0.0, pos2.y) * unit),
                                to_screen.transform_pos(Pos2::new(2000.0, pos3.y) * unit),
                            ],
                            egui::Stroke::new(unit / 2., egui::Color32::GRAY),
                            5.0,
                            2.0,
                        ));
                        painter.add(egui::Shape::circle_filled(
                            to_screen.transform_pos(*pos3 * unit),
                            4.,
                            egui::Color32::GRAY,
                        ));
                    }
                }
                if (pos2.x - pos1.x).abs() <= 2. {
                    painter.add(egui::Shape::dashed_line(
                        &[
                            to_screen.transform_pos(Pos2::new(pos1.x, 0.0) * unit),
                            to_screen.transform_pos(Pos2::new(pos2.x, 2000.0) * unit),
                        ],
                        egui::Stroke::new(unit / 2., egui::Color32::GRAY),
                        5.0,
                        2.0,
                    ));
                }
                if (pos2.y - pos1.y).abs() <= 2. {
                    painter.add(egui::Shape::dashed_line(
                        &[
                            to_screen.transform_pos(Pos2::new(0.0, pos1.y) * unit),
                            to_screen.transform_pos(Pos2::new(2000.0, pos2.y) * unit),
                        ],
                        egui::Stroke::new(unit / 2., egui::Color32::GRAY),
                        5.0,
                        2.0,
                    ));
                }
                painter.add(egui::Shape::line_segment(
                    [
                        to_screen.transform_pos(pos1.to_owned() * unit),
                        to_screen.transform_pos(pos2 * unit),
                    ],
                    egui::Stroke::new(unit, self.stroke.clone()),
                ));
            }
        }

        if response.clicked() {
            let mouse_pos = response.interact_pointer_pos().unwrap();
            let pos1 = mouse_pos + [-12.0, -15.0].into();
            let mut pos1 = to_screen.transform_pos((pos1 / unit).round());
            if let Some(pos2) = canvas.points().last() {
                if let Some(pos3) = canvas.last_closed_point() {
                    if (pos2.x - pos1.x).abs() <= 2. {
                        pos1.x = pos2.x;
                    }
                    if (pos2.y - pos1.y).abs() <= 2. {
                        pos1.y = pos2.y;
                    }
                    if (pos3.x - pos1.x).powi(2) + (pos3.y - pos1.y).powi(2) <= 100. {
                        pos1 = pos3.to_owned();
                    }
                }
                if ui.input(|ui| ui.modifiers.matches_logically(egui::Modifiers::SHIFT)) {
                    let dx = (pos2.x - pos1.x).abs();
                    let dy = (pos2.y - pos1.y).abs();
                    if dy > dx {
                        pos1.x = pos2.x;
                    } else {
                        pos1.y = pos2.y;
                    }
                }
            }
            if canvas.points().len() - canvas.last_closed() > 2 {
                let (x1, y1) = canvas.points()[canvas.points().len() - 1].into();
                let (x2, y2) = canvas.points()[canvas.points().len() - 2].into();
                let (x3, y3) = canvas.points()[canvas.points().len() - 3].into();
                let (x4, y4) = pos1.round().into();
                if !are_collinear(x1, y1, x2, y2, x3, y3, x4, y4) {
                    canvas.add_point(pos1.round());
                } else {
                    self.error
                        .set_error(
                            "Ошибка".to_string(),
                            "Вырожденный многоугольник".to_string(),
                        )
                        .enable();
                }
            } else {
                canvas.add_point(pos1.round());
            }
        }

        let filler = canvas.filler().iter().map(|(pos1, pos2)| {
            let pos1 = pos1.clone();
            let pos2 = pos2.clone();
            egui::Shape::line_segment(
                [
                    to_screen.transform_pos(pos1 * unit),
                    to_screen.transform_pos(pos2 * unit),
                ],
                egui::Stroke::new(unit, self.stroke.clone()),
            )
        });

        let edges = canvas.edges().iter().map(|(p1, p2)| {
            let p1 = canvas.points()[p1.to_owned()];
            let p2 = canvas.points()[p2.to_owned()];
            let p1 = Pos2::new(p1.x, p1.y);
            let p2 = Pos2::new(p2.x, p2.y);
            egui::Shape::line_segment(
                [
                    to_screen.transform_pos(p1 * unit),
                    to_screen.transform_pos(p2 * unit),
                ],
                egui::Stroke::new(unit, self.stroke.clone()),
            )
        });
        painter.extend(filler);
        painter.extend(edges);
    }
}

impl MyApp {
    fn add_point(&mut self) {
        if let (Ok(x), Ok(y)) = (
            self.parse_field::<u32>(self.buf_x.clone()),
            self.parse_field::<u32>(self.buf_y.clone()),
        ) {
            let mut canvas = self.canvas.lock().unwrap();
            if canvas.points().len() - canvas.last_closed() > 2 {
                let (x1, y1) = canvas.points()[canvas.points().len() - 1].into();
                let (x2, y2) = canvas.points()[canvas.points().len() - 2].into();
                let (x3, y3) = canvas.points()[canvas.points().len() - 2].into();
                let (x4, y4) = (x as f32, y as f32);
                if !are_collinear(x1, y1, x2, y2, x3, y3, x4, y4) {
                    canvas.add_point([x as f32, y as f32].into());
                } else {
                    self.error
                        .set_error(
                            "Ошибка".to_string(),
                            "Вырожденный многоугольник".to_string(),
                        )
                        .enable();
                }
            } else {
                canvas.add_point([x as f32, y as f32].into());
            }
        } else {
            self.error.enable();
        }
    }

    fn close_figure(&mut self) {
        let mut canvas = self.canvas.lock().unwrap();
        if canvas.points().len() - canvas.last_closed() > 2 {
            let (x1, y1) = canvas.points()[canvas.points().len() - 1].into();
            let (x2, y2) = canvas.points()[canvas.points().len() - 2].into();
            let (x3, y3) = canvas.points()[canvas.points().len() - 3].into();
            if !are_collinear(x1, y1, x2, y2, x3, y3, x1, y1) {
                if canvas.close().is_none() {
                    self.error
                        .set_error("Ошибка".to_string(), "Нечего замкнуть".to_string())
                        .enable();
                }
            } else {
                self.error
                    .set_error(
                        "Ошибка".to_string(),
                        "Вырожденный многоугольник".to_string(),
                    )
                    .enable();
            }
        } else {
            if canvas.close().is_none() {
                self.error
                    .set_error("Ошибка".to_string(), "Нечего замкнуть".to_string())
                    .enable();
            }
        }
    }

    fn clear_figure(&mut self) {
        let mut canvas = self.canvas.lock().unwrap();
        *self.dur_res.lock().unwrap() = Default::default();
        canvas.clear();
    }

    fn clean_figure(&mut self) {
        let mut canvas = self.canvas.lock().unwrap();
        *self.dur_res.lock().unwrap() = Default::default();
        canvas.clean();
    }

    fn start_filling(&mut self, d: u64) {
        let canvas = self.canvas.clone();
        let mut dur = self.dur_res.clone();
        thread::spawn(move || {
            Canvas::filling(&canvas, &mut dur, d);
        });
    }

    fn fill_figure_run(&mut self) {
        if self.buf_dur.is_empty() || !self.timeout {
            self.start_filling(0);
        } else {
            if let Ok(d) = self.parse_field::<u64>(self.buf_dur.clone()) {
                self.start_filling(d);
            }
        }
    }

    fn fill_figure(&mut self) {
        if !self.canvas.lock().unwrap().is_closed() {
            self.fill_figure_run();
        } else {
            self.error
                .set_error("Ошибка".to_string(), "Фигура не замкнута!".to_string())
                .enable();
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
        "Лабораторная работа 5",
        native_options,
        Box::new(|_| Box::<MyApp>::default()),
    )
}

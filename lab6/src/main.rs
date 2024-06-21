mod logic;

use eframe::egui;
use eframe::egui::{Color32, Pos2};
use egui_extras::{Column, TableBuilder};
use epaint::{Stroke,
             vec2};
use logic::utils::Canvas;
use logic::windows::ErrorWindow;
use std::{
    sync::{Arc, Mutex},
    thread,
};

#[derive(Debug, PartialEq)]
enum DrawMode {
    Line,
    Ellipse,
    Circle,
}

#[derive(Debug)]
struct MyApp {
    background: egui::Color32,
    stroke: egui::Color32,
    border_color: egui::Color32,
    error: ErrorWindow,
    canvas: Arc<Mutex<Canvas>>,

    buf_x: String,
    buf_y: String,

    buf_seed_x: String,
    buf_seed_y: String,

    buf_dur: String,
    dur_res: Arc<Mutex<std::time::Duration>>,
    timeout: bool,
    recursive: bool,
    mode: DrawMode,
    seed: Option<Pos2>,

    buf_rad1: String,
    buf_rad2: String,

    buf_ellipse: (Option<Pos2>, Option<Pos2>),
    buf_circle: (Option<Pos2>, Option<Pos2>),
    bibl: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            error: Default::default(),
            background: egui::Color32::WHITE,
            stroke: egui::Color32::RED,
            border_color: egui::Color32::BLACK,
            canvas: Arc::new(Mutex::new(Canvas::new())),
            buf_x: "".to_string(),
            buf_y: "".to_string(),
            buf_seed_x: "".to_string(),
            buf_seed_y: "".to_string(),
            buf_dur: "".to_string(),
            dur_res: Arc::new(Mutex::new(std::time::Duration::new(0, 0))),
            timeout: false,
            recursive: false,
            seed: None,
            mode: DrawMode::Line,

            buf_rad1: Default::default(),
            buf_rad2: Default::default(),
            buf_ellipse: (None, None),
            buf_circle: (None, None),
            bibl: false
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
            // ui.checkbox(&mut self.bibl, "Отрисовка прямых библиотечным алгоритмом");
            ui.horizontal(|ui| {
                ui.label("Цвет фона");
                use egui::color_picker::{color_edit_button_srgba, Alpha};
                color_edit_button_srgba(ui, &mut self.background, Alpha::Opaque);
                let mut canvas = self.canvas.lock().unwrap();
                canvas.background = self.background;
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Цвет фигуры");
                use egui::color_picker::{color_edit_button_srgba, Alpha};

                color_edit_button_srgba(ui, &mut self.stroke, Alpha::Opaque);
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Цвет границы");
                use egui::color_picker::{color_edit_button_srgba, Alpha};
                color_edit_button_srgba(ui, &mut self.border_color, Alpha::Opaque);
            });

            ui.horizontal_wrapped(|ui| {
                ui.radio_value(&mut self.mode, DrawMode::Line, "Ломанная");
                ui.radio_value(&mut self.mode, DrawMode::Circle, "Окружность");
                ui.radio_value(&mut self.mode, DrawMode::Ellipse, "Эллипс");
            });

            ui.vertical_centered_justified(|ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.buf_x).hint_text("X: "));
                    ui.add(egui::TextEdit::singleline(&mut self.buf_y).hint_text("Y: "));
                    if self.mode == DrawMode::Circle {
                        ui.add(egui::TextEdit::singleline(&mut self.buf_rad1).hint_text("Радиус: "));
                    }

                    if self.mode == DrawMode::Ellipse {
                        ui.add(egui::TextEdit::singleline(&mut self.buf_rad1).hint_text("Полуось x: "));
                        ui.add(egui::TextEdit::singleline(&mut self.buf_rad2).hint_text("Полуось y: "));
                    }
                });
                match self.mode {
                    DrawMode::Line => {
                        if ui.button("Добавить точку").clicked() {
                            self.add_point();
                        }
                    }
                    DrawMode::Circle => {
                        if ui.button("Добавить Окружность").clicked() {
                            self.add_circle();
                        }
                    }
                    DrawMode::Ellipse => {
                        if ui.button("Добавить Эллипс").clicked() {
                            self.add_ellipse();
                        }
                    }
                }
            });
            self.update_table(ui);
            if let Some(seed) = self.seed {
                ui.label(format!("Затравочный пиксель: {} {}", seed.x as u32, seed.y as u32));
            } else {
                ui.label("Затравочный пиксель: не установлен");
            }
            ui.vertical_centered_justified(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.buf_seed_x).hint_text("Затравка X: "));
                ui.add(egui::TextEdit::singleline(&mut self.buf_seed_y).hint_text("Затравка Y: "));
                if ui.button("Установить затравку").clicked() {
                    self.set_seed();
                }
            });

            ui.vertical_centered_justified(|ui| {
                if self.mode == DrawMode::Line && ui.button("Замкнуть фигуру").clicked() {
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
                            // if canvas.all_closed().contains(&c) {
                            //     ui.label("Фигура");
                            // }
                            ui.label(format!("{}", c));
                        });
                        row.col(|ui| {
                            ui.label(format!("{}", i.0.x));
                        });
                        row.col(|ui| {
                            ui.label(format!("{}", i.0.y));
                        });
                    });
                }
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


        if response.clicked_by(egui::PointerButton::Middle) {
            let mouse_pos = response.hover_pos().unwrap_or_default();
            let pos1 = mouse_pos + [-6.0, -5.0].into();
            let pos1 = to_screen.transform_pos((pos1 / unit).round());
            self.set_seed_pos(pos1.x as u32, pos1.y as u32);
        }
        let mut canvas = self.canvas.lock().unwrap();
        if response.clicked_by(egui::PointerButton::Secondary) {
            canvas.close();
        }

        if response.hovered() {
            let mouse_pos = response.hover_pos().unwrap_or_default();
            let pos1 = mouse_pos + [-6.0, -5.0].into();
            let mut pos1 = to_screen.transform_pos((pos1 / unit).round());
            match self.mode {
                DrawMode::Line => {
                    if let Some(&pos2) = canvas.last_closed_point() {
                        if ui.input(|ui| ui.modifiers.matches_logically(egui::Modifiers::SHIFT)) {
                            let dx = (pos2.x - pos1.x).abs();
                            let dy = (pos2.y - pos1.y).abs();
                            if dy > dx {
                                pos1.x = pos2.x;
                            } else {
                                pos1.y = pos2.y;
                            }
                        }
                        painter.line_segment(
                            [pos1 * unit, pos2 * unit],
                            Stroke::new(unit, self.border_color),
                        );
                    }
                }
                DrawMode::Ellipse => {
                    if let Some(c) = self.buf_ellipse.0 {
                        let dx = (c.x - pos1.x).abs();
                        let dy = (c.y - pos1.y).abs();
                        painter.add(
                            egui::Shape::ellipse_stroke(c * unit, vec2(dx, dy) * unit, Stroke::new(unit, self.border_color))
                        );
                    }
                }
                DrawMode::Circle => {
                    if let Some(x) = self.buf_circle.0 {
                        let dx = std::cmp::max((x.x - pos1.x).abs() as u32, (x.y - pos1.y).abs() as u32);
                        painter.add(
                            egui::Shape::circle_stroke(x * unit, dx as f32 * unit, Stroke::new(unit, self.border_color))
                        );
                    }
                }
            }
        }
        if response.dragged() || response.clicked() {
            let mouse_pos = response.hover_pos().unwrap_or_default();
            let pos1 = mouse_pos + [-6.0, -5.0].into();
            let mut pos1 = to_screen.transform_pos((pos1 / unit).round()).round();
            match self.mode {
                DrawMode::Line => {
                    let pos2 = canvas.last_closed_point();
                    if Some(&pos1) != pos2 {
                        if let Some(&pos2) = pos2 {
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
                        canvas.add_point(pos1.round(), self.border_color);
                    }
                }
                DrawMode::Ellipse => {
                    if let Some(x) = self.buf_ellipse.0 {
                        self.buf_ellipse = (None, None);
                        let dx = (x.x - pos1.x).abs();
                        let dy = (x.y - pos1.y).abs();
                        canvas.add_ellipse(x, [dx, dy].into(), self.border_color);
                    } else {
                        self.buf_ellipse.0 = Some(pos1);
                    }
                }
                DrawMode::Circle => {
                    if let Some(x) = self.buf_circle.0 {
                        self.buf_circle = (None, None);
                        let dx = std::cmp::max((x.x - pos1.x).abs() as u32, (x.y - pos1.y).abs() as u32);
                        canvas.add_circle(x, dx as f32, self.border_color);
                    } else {
                        self.buf_circle.0 = Some(pos1);
                    }
                }
            }
        }
        let shapes = canvas.strings.iter().map(|&((a, b), c)| {
            egui::Shape::line_segment([a * unit, b * unit], Stroke::new(unit, c))
        });
        painter.extend(shapes);



        if self.bibl {
            let shapes: Vec<_> = canvas
                .closes()
                .windows(2)
                .flat_map(|pair| {
                    canvas.points()[pair[0]..pair[1]].windows(2).map(|window| {
                        let ((pos1, _), (pos2, color2)) = (window[0], window[1]);
                        egui::Shape::line_segment([pos1 * unit, pos2 * unit], Stroke::new(unit, color2))
                    })
                })
                .chain(
                    canvas
                        .points()
                        .get(canvas.closes().last().unwrap().to_owned()..)
                        .map(|points| {
                            points.windows(2).map(|window| {
                                let ((pos1, _), (pos2, color2)) = (window[0], window[1]);
                                egui::Shape::line_segment(
                                    [pos1 * unit, pos2 * unit],
                                    Stroke::new(unit, color2),
                                )
                            })
                        })
                        .unwrap(),
                )
                .collect();

            painter.extend(shapes);

            let shapes = canvas.circles.iter().map(|&(a, b, c)| {
                egui::Shape::circle_stroke(a * unit, b * unit, Stroke::new(unit, c))
            });
            painter.extend(shapes);

            let shapes = canvas.ellipse.iter().map(|&(a, b, c)| {
                egui::Shape::ellipse_stroke(a * unit, b.to_vec2() * unit, Stroke::new(unit, c))
            });
            painter.extend(shapes);
        } else {
            let shapes = canvas.pixels_edges.iter().map(|(&(x, y), &(r, g, b))| {
                let p = Pos2::new(x as f32 * unit, y as f32 * unit);
                egui::Shape::rect_filled(
                    [
                        p,
                        p + [unit, unit].into(),
                    ]
                        .into(),
                    0.0,
                    Color32::from_rgb(r, g, b),
                )
            });
            painter.extend(shapes);
        }

        if let Some(seed) = self.seed {
            painter.circle_filled(seed * unit, unit * 2., self.stroke);
        }
    }
}

impl MyApp {
    fn add_point(&mut self) {
        if let (Ok(x), Ok(y)) = (
            self.parse_field::<u32>(self.buf_x.clone()),
            self.parse_field::<u32>(self.buf_y.clone()),
        ) {
            let mut canvas = self.canvas.lock().unwrap();
            canvas.add_point([x as f32, y as f32].into(), self.border_color);
        } else {
            self.error.enable();
        }
    }

    fn add_circle(&mut self) {
        if let (Ok(x), Ok(y), Ok(r)) = (
            self.parse_field::<u32>(self.buf_x.clone()),
            self.parse_field::<u32>(self.buf_y.clone()),
            self.parse_field::<u32>(self.buf_rad1.clone()),
        ) {
            let mut canvas = self.canvas.lock().unwrap();
            canvas.add_circle([x as f32, y as f32].into(), r as f32, self.border_color);
        } else {
            self.error.enable();
        }
    }

    fn add_ellipse(&mut self) {
        if let (Ok(x), Ok(y), Ok(r1), Ok(r2)) = (
            self.parse_field::<u32>(self.buf_x.clone()),
            self.parse_field::<u32>(self.buf_y.clone()),
            self.parse_field::<u32>(self.buf_rad1.clone()),
            self.parse_field::<u32>(self.buf_rad2.clone()),
        ) {
            let mut canvas = self.canvas.lock().unwrap();
            canvas.add_ellipse([x as f32, y as f32].into(), [r1 as f32, r2 as f32].into(), self.border_color);
        } else {
            self.error.enable();
        }
    }

    fn close_figure(&mut self) {
        // self.set_seed_pos(501, 600);
        let mut canvas = self.canvas.lock().unwrap();
        // canvas.add_point([500., 500.].into(),  self.border_color);
        // canvas.add_point([500., 900.].into(),  self.border_color);
        // canvas.add_point([502., 900.].into(),  self.border_color);
        // canvas.add_point([502., 500.].into(), self.border_color);

        canvas.close();
    }

    fn set_seed(&mut self) {
        if let (Ok(x), Ok(y)) = (
            self.parse_field::<u32>(self.buf_seed_x.clone()),
            self.parse_field::<u32>(self.buf_seed_y.clone()),
        ) {
            self.set_seed_pos(x, y);
        } else {
            self.error.enable();
        }
    }

    fn set_seed_pos(&mut self, x: u32, y: u32) {
        let canvas = self.canvas.lock().unwrap();
        if canvas.at(x, y) != self.border_color {
            self.seed = Some(Pos2::new(x as f32, y as f32));
        } else {
            self.error.set_error("Ошибка".into(), "Затравочный пиксель должен быть внутри области".into()).enable();
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
        if let Some(seed) = self.seed {
            let canvas = self.canvas.clone();
            let mut dur = self.dur_res.clone();
            let fill = self.stroke.clone();
            let border = self.border_color.clone();
            let rec = self.recursive.clone();

            thread::spawn(move || {
                Canvas::filling(seed, &canvas, fill, border, &mut dur, d, rec);
            });
        } else {
            self.error.set_error("Ошибка".into(), "Не указана затравка".into()).enable();
        }
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
        self.fill_figure_run();
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
        "Лабораторная работа 6",
        native_options,
        Box::new(|_| Box::<MyApp>::default()),
    )
}

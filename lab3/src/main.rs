mod logic;

use logic::algo::*;
use logic::utils::*;
use logic::windows::*;

use eframe::egui::color_picker::color_edit_button_srgba;
use eframe::egui::widgets::color_picker::Alpha::Opaque;
use eframe::egui::{pos2, Color32, Pos2, Stroke};
use eframe::{egui, emath};

#[derive(Debug, Clone)]
enum Line {
    Path([Pos2; 2], Color32),
    Line(Vec<Pos2>, Color32),
    LinePix(Vec<(Pos2, f32)>, Color32),
}

#[derive(Default, Debug, Clone, PartialEq)]
enum DrawType {
    #[default]
    Segment,
    Spectre,
}

#[derive(Debug)]
struct MyApp {
    error: ErrorWindow,
    histo: HistoWindow,
    graph: GraphWindow,
    draw_type: DrawType,
    lines: Vec<Line>,
    algo: Algo,
    background: Color32,
    buf_x1: String,
    buf_y1: String,
    buf_x2: String,
    buf_y2: String,
    buf_angle: String,
    buf_linecolor: Color32,
    ppp: i32,
    buf_len: String,
    buf_len2: String,
    painter_pos: Pos2,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            error: Default::default(),
            histo: Default::default(),
            graph: Default::default(),
            draw_type: Default::default(),
            lines: vec![],
            algo: Default::default(),
            background: Color32::WHITE,
            buf_x1: "712".into(),
            buf_y1: "484".into(),
            buf_x2: "0".into(),
            buf_y2: "0".into(),
            buf_angle: Default::default(),
            buf_linecolor: Color32::RED,
            buf_len: Default::default(),
            ppp: 1,
            painter_pos: Pos2::ZERO,
            buf_len2: "1000".to_string(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.error().enabled() {
            self.error().update(ctx);
        }
        if self.histo().enabled() {
            self.histo().update(ctx);
        }
        if self.graph().enabled() {
            self.graph().update(ctx);
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(!self.error().enabled());
            ui.set_enabled(!self.histo().enabled());
            ui.set_enabled(!self.graph().enabled());
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(950.0);
                    ui.set_height(650.0);
                    self.painter(ui);
                });
                self.control(ui);
            });
        });
    }
}

impl MyApp {
    fn error(&mut self) -> &mut ErrorWindow {
        &mut self.error
    }

    fn histo(&mut self) -> &mut HistoWindow {
        &mut self.histo
    }

    fn graph(&mut self) -> &mut GraphWindow {
        &mut self.graph
    }

    pub fn painter(&mut self, ui: &mut egui::Ui) {
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
            painter.rect(
                painter.clip_rect().shrink(0.0),
                0.0,
                self.background,
                Stroke::new(0.5, Color32::BLACK),
            );
            let p_rect = response.rect;

            let to_screen = emath::RectTransform::from_to(
                egui::Rect::from_min_size(
                    pos2(
                        (self.ppp - 1) as f32 * p_rect.width() / 2.0,
                        (self.ppp - 1) as f32 * p_rect.height() / 2.0,
                    ),
                    p_rect.size(),
                ),
                response.rect.translate(self.painter_pos.to_vec2()),
            );
            let ppp = ui.ctx().pixels_per_point();
            let unit = 1.0 / ppp;
            let shapes = self
                .lines
                .iter()
                .map(|line| self.convert_line_to_shape(line, &to_screen, unit));
            painter.extend(shapes);
        });
    }

    fn convert_line_to_shape(
        &self,
        line: &Line,
        to_screen: &emath::RectTransform,
        unit: f32,
    ) -> egui::Shape {
        match line {
            Line::Path(v, c) => egui::Shape::line_segment(
                [
                    to_screen.transform_pos(v[0] * unit * self.ppp as f32),
                    to_screen.transform_pos(v[1] * unit * self.ppp as f32),
                ],
                Stroke::new(unit * self.ppp as f32, c.clone()),
            ),

            Line::Line(v, c) => egui::Shape::Vec(
                v.iter()
                    .map(|p| {
                        let unit = unit * self.ppp as f32;
                        let p = pos2(p.x * unit, p.y * unit);
                        egui::Shape::rect_filled(
                            [
                                to_screen.transform_pos(p),
                                to_screen.transform_pos(p + [unit, unit].into()),
                            ]
                            .into(),
                            0.0,
                            c.clone(),
                        )
                    })
                    .collect(),
            ),

            Line::LinePix(v, c) => egui::Shape::Vec(
                v.iter()
                    .map(|(p, a)| {
                        let unit = unit * self.ppp as f32;
                        let p = pos2(p.x * unit, p.y * unit);
                        egui::Shape::rect_filled(
                            [
                                to_screen.transform_pos(p),
                                to_screen.transform_pos(p + [unit, unit].into()),
                            ]
                            .into(),
                            0.0,
                            abate_color(c.to_owned(), self.background, a.to_owned()),
                        )
                    })
                    .collect(),
            ),
        }
    }

    fn control(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Цвет фона");
                    color_edit_button_srgba(ui, &mut self.background, Opaque);
                });
                ui.horizontal(|ui| {
                    ui.label("Пикселей в точке");
                    ui.add(egui::Slider::new(&mut self.ppp, 1..=20).suffix("^2"));
                });
                ui.radio_value(&mut self.algo, Algo::DDA, "ЦДА");
                ui.radio_value(
                    &mut self.algo,
                    Algo::BresenhamFloat,
                    "Алгоритм Брезенхема вещественный",
                );
                ui.radio_value(
                    &mut self.algo,
                    Algo::BresenhamReal,
                    "Алгоритм Брезенхема целочисленный",
                );
                ui.radio_value(
                    &mut self.algo,
                    Algo::BresenhamJaggiesLess,
                    "Алгоритм Брезенхема с устранением ступенчатости",
                );
                ui.radio_value(&mut self.algo, Algo::WU, "Алгоритм Ву");
                ui.radio_value(&mut self.algo, Algo::BuiltIn, "Встроенная функция");

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Цвет отрезка");
                    color_edit_button_srgba(ui, &mut self.buf_linecolor, Opaque);
                });
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.draw_type, DrawType::Segment, "Отрезок");
                    ui.radio_value(&mut self.draw_type, DrawType::Spectre, "Спектр");
                });
                ui.vertical_centered_justified(|ui| {
                    ui.label("Первая точка");
                    ui.vertical_centered_justified(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.buf_x1).hint_text("X1"));
                        ui.add(egui::TextEdit::singleline(&mut self.buf_y1).hint_text("Y1"));
                    });
                });
                match self.draw_type {
                    DrawType::Spectre => {
                        ui.vertical_centered_justified(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.buf_len).hint_text("Длина"),
                            );
                            ui.add(
                                egui::TextEdit::singleline(&mut self.buf_angle).hint_text("Угол"),
                            );
                            if ui.button("Построить спектр").clicked() {
                                self.parse_to_draw_spectre()
                            };
                        });
                    }
                    DrawType::Segment => {
                        ui.vertical_centered_justified(|ui| {
                            ui.label("Вторая точка");
                            ui.vertical_centered_justified(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.buf_x2).hint_text("X2"),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.buf_y2).hint_text("Y2"),
                                );
                            });
                        });
                        ui.vertical_centered_justified(|ui| {
                            if ui.button("Построить прямую").clicked() {
                                self.parse_to_draw_line();
                            };
                        });
                    }
                }

                ui.separator();
                ui.vertical_centered_justified(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.buf_len2).hint_text("Длина"));
                    if ui.button("Сравнение времени").clicked() {
                        self.parse_to_measure();
                    };
                    if ui.button("Сравнение ступенчатости").clicked() {
                        self.parse_jaggies();
                    };
                    if ui.button("Очистка экрана").clicked() {
                        self.clear();
                    };
                });
                ui.separator();
            });
        });
    }

    fn parse_field_x1(&mut self) -> Result<f32, ()> {
        self.buf_x1.parse::<f32>().map_err(|_| {
            self.error().set_error(
                "Ошибка".to_string(),
                "Некорректное значение в поле x1:".to_string(),
            )
        })
    }

    fn parse_field_y1(&mut self) -> Result<f32, ()> {
        self.buf_y1.parse::<f32>().map_err(|_| {
            self.error().set_error(
                "Ошибка".to_string(),
                "Некорректное значение в поле Y1:".to_string(),
            )
        })
    }
    fn parse_field_x2(&mut self) -> Result<f32, ()> {
        self.buf_x2.parse::<f32>().map_err(|_| {
            self.error().set_error(
                "Ошибка".to_string(),
                "Некорректное значение в поле x2:".to_string(),
            )
        })
    }
    fn parse_field_y2(&mut self) -> Result<f32, ()> {
        self.buf_y2.parse::<f32>().map_err(|_| {
            self.error().set_error(
                "Ошибка".to_string(),
                "Некорректное значение в поле Y2:".to_string(),
            )
        })
    }
    fn parse_field_angle(&mut self) -> Result<f32, ()> {
        let res = self.buf_angle.parse::<f32>().map_err(|_| {
            self.error().set_error(
                "Ошибка".to_string(),
                "Некорректное значение в поле угла:".to_string(),
            )
        });
        res
    }

    fn parse_field_len(&mut self) -> Result<f32, ()> {
        let res = self.buf_len.parse::<f32>().map_err(|_| {
            self.error().set_error(
                "Ошибка".to_string(),
                "Некорректное значение в поле длины:".to_string(),
            )
        });
        res
    }

    fn parse_field_len2(&mut self) -> Result<f32, ()> {
        let res = self.buf_len2.parse::<f32>().map_err(|_| {
            self.error().set_error(
                "Ошибка".to_string(),
                "Некорректное значение в поле длины:".to_string(),
            )
        });
        res
    }

    fn parse_to_draw_line(&mut self) {
        let x1 = self.parse_field_x1();
        let y1 = self.parse_field_y1();
        let x2 = self.parse_field_x2();
        let y2 = self.parse_field_y2();
        if let (Ok(x1), Ok(y1), Ok(x2), Ok(y2)) = (x1, y1, x2, y2) {
            self.draw_line([pos2(x1 as f32, y1 as f32), pos2(x2 as f32, y2 as f32)]);
        } else {
            self.error.enable();
        }
    }

    fn parse_to_draw_spectre(&mut self) {
        let x1 = self.parse_field_x1();
        let y1 = self.parse_field_y1();
        let len = self.parse_field_len();
        let angle = self.parse_field_angle();
        if let (Ok(x1), Ok(y1), Ok(angle), Ok(len)) = (x1, y1, angle, len) {
            if angle == 0.0 {
                self.error().set_error(
                    "Ошибка".to_string(),
                    "Некорректное значение в поле угла: 0".to_string(),
                )
            } else {
                self.draw_spectre(
                    [
                        pos2(x1 as f32, y1 as f32),
                        pos2(x1 as f32 + len as f32, y1 as f32),
                    ],
                    angle,
                );
            }
        } else {
            self.error.enable();
        }
    }

    fn parse_to_measure(&mut self) {
        let len = self.parse_field_len2();
        if let Ok(len) = len {
            self.histo.enable();
            let dda = measure_time(dda, len);
            let bf = measure_time(bresenham_float, len);
            let bi = measure_time(bresenham_int, len);
            let bj = measure_time(bresenham_jaggiesless, len);
            let wu = measure_time(wu, len);
            let eg = measure_time(
                |points| {
                    egui::Painter::line_segment(
                        &mut egui::Painter::new(
                            egui::Context::default(),
                            egui::LayerId::debug(),
                            egui::Rect::NAN,
                        ),
                        points.clone(),
                        Stroke::new(1.0, egui::Color32::RED),
                    )
                },
                len,
            );
            let mut dm = vec![dda, bf, bi, bj, wu, eg];
            dm.sort();
            let values = vec![
                ("Брезенхем целочисленный".to_owned(), dm[1]),
                ("Брезенхем вещественный".to_owned(), dm[2]),
                ("Брезенхем с устранением ступенчатости".to_owned(), dm[3]),
                ("ЦДА".to_owned(), dm[4]),
                ("ВУ".to_owned(), dm[5]),
                ("Встроенная функция".to_owned(), dm[0]),
            ];
            self.histo.set_values(&values);
        } else {
            self.error.enable();
        }
    }

    fn parse_jaggies(&mut self) {
        let len = self.parse_field_len2();
        if let Ok(len) = len {
            self.graph().enable();
            let dda = measure_jaggies(dda, len);
            let bf = measure_jaggies(bresenham_float, len);
            let bi = measure_jaggies(bresenham_int, len);
            let bj = measure_jaggies_bj(bresenham_jaggiesless, len);
            let wu = measure_jaggies_wu(wu, len);
            let values = vec![
                ("ЦДА".to_owned(), dda),
                ("Брезенхем целочисленный".to_owned(), bf),
                ("Брезенхем вещественный".to_owned(), bi),
                ("Брезенхем с устранением ступенчатости".to_owned(), bj),
                ("ВУ".to_owned(), wu),
            ];
            self.graph().set_values(&values, len);
        } else {
            self.error.enable();
        }
    }

    fn draw_line(&mut self, points: [Pos2; 2]) {
        match self.algo {
            Algo::DDA => {
                let points = dda(&points);
                self.lines.push(Line::Line(points, self.buf_linecolor));
            }
            Algo::BuiltIn => {
                self.lines.push(Line::Path(points, self.buf_linecolor));
            }
            Algo::BresenhamFloat => {
                let points = bresenham_float(&points);
                self.lines.push(Line::Line(points, self.buf_linecolor));
            }
            Algo::BresenhamReal => {
                let points = bresenham_int(&points);
                self.lines.push(Line::Line(points, self.buf_linecolor));
            }
            Algo::BresenhamJaggiesLess => {
                let points = bresenham_jaggiesless(&points);
                self.lines.push(Line::LinePix(points, self.buf_linecolor));
            }
            Algo::WU => {
                let points = wu(&points);
                self.lines.push(Line::LinePix(points, self.buf_linecolor));
            }
        };
    }

    fn draw_spectre(&mut self, mut points: [Pos2; 2], angle: f32) {
        let mut i = 0.0;
        self.draw_line(points);
        while i + angle <= 360.0 {
            let center = points[0];
            rotate(&mut points[1], angle, &center);
            self.draw_line(points);
            i += angle;
        }
    }

    fn clear(&mut self) {
        self.lines.clear();
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((1280.0, 1024.0)),
        default_theme: eframe::Theme::Light,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "Лабораторная работа 3",
        native_options,
        Box::new(|_| Box::<MyApp>::default()),
    )
}

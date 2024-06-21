mod logic;

use std::fs::OpenOptions;
use eframe::egui;
use egui::{Pos2, Vec2};
use logic::algorithms::*;
use logic::windows::*;
use logic::utils::*;

#[derive(Default, Debug, PartialEq)]
enum ShapeType {
    #[default]
    CIRCLE,
    ELLIPSE,
}

#[derive(Default, Debug, PartialEq)]
enum ShowType {
    #[default]
    ONE,
    SPECTRE,
}

#[derive(Debug)]
enum CanonicalShapes {
    Circle(Pos2, f32, egui::Color32),
    Ellipse(Pos2, Vec2, egui::Color32),
    Path(Vec<egui::Pos2>, egui::Color32),
}

#[derive(Debug)]
struct MyApp {
    error: ErrorWindow,
    graph: GraphWindow,
    background: egui::Color32,
    stroke: egui::Color32,
    shape_type: ShapeType,
    draw_type: DrawType,
    show_type: ShowType,
    shapes: Vec<CanonicalShapes>,
    buf_x: String,
    buf_y: String,
    buf_radius: String,
    buf_axe1: String,
    buf_axe2: String,
    buf_count: String,
    buf_step: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            error: Default::default(),
            graph: Default::default(),
            background: egui::Color32::WHITE,
            stroke: egui::Color32::RED,
            shape_type: Default::default(),
            draw_type: Default::default(),
            show_type: Default::default(),
            shapes: vec![],
            buf_x: "700".to_string(),
            buf_y: "400".to_string(),
            buf_radius: "".to_string(),
            buf_axe1: "".to_string(),
            buf_axe2: "".to_string(),
            buf_count: "".to_string(),
            buf_step: "".to_string(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.error.enabled() {
            self.error.update(ctx);
        }
        if self.graph.enabled() {
            self.graph.update(ctx);
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
            ui.radio_value(&mut self.draw_type, DrawType::CANONICAL, "Каноническое");
            ui.radio_value(&mut self.draw_type, DrawType::PARAMETRIC, "Параметрическое");
            ui.radio_value(&mut self.draw_type, DrawType::MIDPOINT, "Средняя точка");
            ui.radio_value(&mut self.draw_type, DrawType::BRESENHAM, "Брезензем");
            ui.radio_value(&mut self.draw_type, DrawType::BuiltIn, "Библиотечная");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Цвет фигуры");
                use egui::color_picker::{color_edit_button_srgba, Alpha};
                color_edit_button_srgba(ui, &mut self.stroke, Alpha::Opaque);
            });
            ui.add(egui::TextEdit::singleline(&mut self.buf_x).hint_text("Центр x"));
            ui.add(egui::TextEdit::singleline(&mut self.buf_y).hint_text("Центр y"));
            ui.horizontal_wrapped(|ui| {
                ui.radio_value(&mut self.shape_type, ShapeType::CIRCLE, "Окружность");
                ui.radio_value(&mut self.shape_type, ShapeType::ELLIPSE, "Эллипс");
            });
            match self.shape_type {
                ShapeType::CIRCLE => {
                    ui.add(egui::TextEdit::singleline(&mut self.buf_radius).hint_text("Радиус"));
                }
                ShapeType::ELLIPSE => {
                    ui.add(egui::TextEdit::singleline(&mut self.buf_axe1).hint_text("Полуось x"));
                    ui.add(egui::TextEdit::singleline(&mut self.buf_axe2).hint_text("Полуось y"));
                }
            }
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.radio_value(&mut self.show_type, ShowType::ONE, "Фигура");
                ui.radio_value(&mut self.show_type, ShowType::SPECTRE, "Спектр");
            });
            if self.show_type == ShowType::SPECTRE {
                ui.add(egui::TextEdit::singleline(&mut self.buf_step).hint_text("Шаг"));
                ui.add(egui::TextEdit::singleline(&mut self.buf_count).hint_text("Количество"));
            }
            ui.vertical_centered_justified(|ui| {
                if ui.button("Построить").clicked() {
                    self.parse_draw_request();
                };
            });

            ui.separator();
            ui.vertical_centered_justified(|ui| {
                if ui.button("Очистить холст").clicked() {
                    self.clear();
                };
            });

            ui.separator();
            ui.label("Замеры времени");
            ui.vertical_centered_justified(|ui| {
                if ui.button("Измерить окружности").clicked() {
                    self.measure_circle()
                }
                if ui.button("Измерить эллипсы").clicked() {
                    self.measure_ellipse()
                }
            });
            ui.separator();

        });
    }

    fn painter(&self, ui: &mut egui::Ui) {
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
        let shapes = self.shapes.iter().map(|shape| match shape {
            CanonicalShapes::Circle(center, radius, stroke) => {
                let center = *center * unit;
                let radius = *radius * unit;
                egui::Shape::circle_stroke(
                    to_screen.transform_pos(center),
                    radius.to_owned(),
                    egui::Stroke::new(unit, stroke.to_owned()),
                )
            }
            CanonicalShapes::Ellipse(center, axes, stroke) => {
                let center = *center * unit;
                let axes = *axes * unit;
                egui::Shape::ellipse_stroke(
                    to_screen.transform_pos(center.to_owned()),
                    axes.to_owned(),
                    egui::Stroke::new(unit, stroke.to_owned()),
                )
            }
            CanonicalShapes::Path(pix, stroke) => egui::Shape::Vec(
                pix.iter()
                    .map(|p| {
                        let p = Pos2::new(p.x * unit, p.y * unit);
                        egui::Shape::rect_filled(
                            [
                                to_screen.transform_pos(p),
                                to_screen.transform_pos(p + [unit, unit].into()),
                            ]
                            .into(),
                            0.0,
                            stroke.clone(),
                        )
                    })
                    .collect(),
            ),
        });
        painter.extend(shapes);
    }



    fn measure_circle(&mut self) {
            self.graph.enable();

            let mut b = OpenOptions::new()
                .write(true)
                .append(true)
                .read(true)
                .open("./src/data/circle/bresenham.txt").unwrap();

            let mut c =  OpenOptions::new()
                .read(true)
                .append(true)
                .read(true)
                .open("./src/data/circle/canonical.txt").unwrap();

            let mut m =  OpenOptions::new()
                .append(true)
                .write(true)
                .read(true)
                .open("./src/data/circle/midpoint.txt").unwrap();

            let mut p = OpenOptions::new()
                .append(true)
                .write(true)
                .read(true)
                .open("./src/data/circle/parametric.txt").unwrap();

            let mut i = OpenOptions::new()
                .append(true)
                .write(true)
                .read(true)
                .open("./src/data/circle/builtin.txt").unwrap();

            let mut circle = Circle::new([700.0, 500.0].into(), 500.0);
            run_profile(&mut circle, DrawType::BRESENHAM, &mut b);
            run_profile(&mut circle, DrawType::CANONICAL, &mut c);
            run_profile(&mut circle, DrawType::MIDPOINT, &mut m);
            run_profile(&mut circle, DrawType::PARAMETRIC, &mut p);
            run_profile(&mut circle, DrawType::BuiltIn, &mut i);

            let values = vec![
                ("Каноническое уравнение".to_owned(), prof(&mut b)),
                ("Параметрическое уравнение".to_owned(), prof(&mut p)),
                ("Алгоритм средней точки".to_owned(), prof(&mut c)),
                ("Алгоритм Брезенхема".to_owned(), prof(&mut m)),
                ("Встроенный".to_owned(), prof(&mut i)),
            ];
            self.graph.set_values(&values);
    }

    fn measure_ellipse(&mut self) {
        self.graph.enable();
        let mut b = OpenOptions::new()
            .write(true)
            .append(true)
            .read(true)
            .open("./src/data/ellipse/bresenham.txt").unwrap();
            // .open("/dev/null").unwrap();

        let mut c =  OpenOptions::new()
            .read(true)
            .append(true)
            .read(true)
            .open("./src/data/ellipse/canonical.txt").unwrap();
            // .open("/dev/null").unwrap();

        let mut m =  OpenOptions::new()
            .append(true)
            .write(true)
            .read(true)
            .open("./src/data/ellipse/midpoint.txt").unwrap();
            // .open("/dev/null").unwrap();

        let mut p = OpenOptions::new()
            .append(true)
            .write(true)
            .read(true)
            .open("./src/data/ellipse/parametric.txt").unwrap();
            // .open("/dev/null").unwrap();

        let mut i = OpenOptions::new()
            .append(true)
            .write(true)
            .read(true)
            .open("./src/data/ellipse/builtin.txt").unwrap();
            // .open("/dev/null").unwrap();

        let mut ellipse = Ellipse::new([700.0, 500.0].into(), [500.0, 500.0].into());
        run_profile(&mut ellipse, DrawType::BRESENHAM, &mut b);
        run_profile(&mut ellipse, DrawType::CANONICAL, &mut c);
        run_profile(&mut ellipse, DrawType::MIDPOINT, &mut m);
        run_profile(&mut ellipse, DrawType::PARAMETRIC, &mut p);
        run_profile(&mut ellipse, DrawType::BuiltIn, &mut i);
        let values = vec![
            ("Каноническое уравнение".to_owned(), prof(&mut b)),
            ("Параметрическое уравнение".to_owned(), prof(&mut c)),
            ("Алгоритм Брезенхема".to_owned(), prof(&mut p)),
            ("Алгоритм средней точки".to_owned(), prof(&mut m)),
            ("Встроенный".to_owned(), prof(&mut i)),
        ];
        self.graph.set_values(&values);
    }
}

// parsing
impl MyApp {
    fn parse_field<T>(&mut self, field: String) -> Result<T, ()>
    where
        T: std::str::FromStr,
    {
        field.parse::<T>().map_err(|_| {
            self.error.set_error(
                "Ошибка".to_string(),
                format!("Ошибочное значение {}", field),
            )
        })
    }

    fn parse_to_draw_circle(&mut self) {
        if let (Ok(x), Ok(y), Ok(r)) = (
            self.parse_field::<u32>(self.buf_x.clone()),
            self.parse_field::<u32>(self.buf_y.clone()),
            self.parse_field::<u32>(self.buf_radius.clone()),
        ) {
            self.draw_circle([x as f32, y as f32].into(), r as f32);
        } else {
            self.error.enable();
        }
    }

    fn parse_to_draw_ellipse(&mut self) {
        if self.buf_axe1 == "0" || self.buf_axe2 == "0" {
            self.error.set_error("Ошибка".to_owned(), "0 полуось нельзя".to_owned());
            self.error.enable();
            return;
        }
        else if let (Ok(x), Ok(y), Ok(a1), Ok(a2)) = (
            self.parse_field::<u32>(self.buf_x.clone()),
            self.parse_field::<u32>(self.buf_y.clone()),
            self.parse_field::<u32>(self.buf_axe1.clone()),
            self.parse_field::<u32>(self.buf_axe2.clone()),
        ) {
            self.draw_ellipse([x as f32 , y as f32 ].into(), [a1 as f32, a2 as f32].into());
        } else {
            self.error.enable()
        }
    }

    fn draw_circle(&mut self, center: Pos2, r: f32) {
        match self.draw_type {
            DrawType::CANONICAL => {
                self.shapes.push(CanonicalShapes::Path(
                    Circle::new(center, r).draw_canonic(),
                    self.stroke,
                ));
            }
            DrawType::PARAMETRIC => {
                self.shapes.push(CanonicalShapes::Path(
                    Circle::new(center, r).draw_parametric(),
                    self.stroke,
                ));
            }
            DrawType::MIDPOINT => {
                self.shapes.push(CanonicalShapes::Path(
                    Circle::new(center, r).draw_midpoint(),
                    self.stroke,
                ));
            }
            DrawType::BRESENHAM => {
                self.shapes.push(CanonicalShapes::Path(
                    Circle::new(center, r).draw_bresenham(),
                    self.stroke,
                ));
            }
            DrawType::BuiltIn => self
                .shapes
                .push(CanonicalShapes::Circle(center, r, self.stroke)),
        }
    }

    fn draw_ellipse(&mut self, center: Pos2, radius: Vec2) {
        match self.draw_type {
            DrawType::CANONICAL => {
                self.shapes.push(CanonicalShapes::Path(
                    Ellipse::new(center, radius).draw_canonic(),
                    self.stroke,
                ));
            }
            DrawType::PARAMETRIC => {
                self.shapes.push(CanonicalShapes::Path(
                    Ellipse::new(center, radius).draw_parametric(),
                    self.stroke,
                ));
            }
            DrawType::MIDPOINT => {
                self.shapes.push(CanonicalShapes::Path(
                    Ellipse::new(center, radius).draw_midpoint(),
                    self.stroke,
                ));
            }
            DrawType::BRESENHAM => {
                self.shapes.push(CanonicalShapes::Path(
                    Ellipse::new(center, radius).draw_bresenham(),
                    self.stroke,
                ));
            }
            DrawType::BuiltIn => {
                self.shapes
                    .push(CanonicalShapes::Ellipse(center, radius, self.stroke))
            }
        }
    }

    fn parse_draw_request(&mut self) {
        match self.show_type {
            ShowType::ONE => match self.shape_type {
                ShapeType::CIRCLE => self.parse_to_draw_circle(),
                ShapeType::ELLIPSE => self.parse_to_draw_ellipse(),
            },
            ShowType::SPECTRE => match self.shape_type {
                ShapeType::CIRCLE => self.parse_to_draw_circle_spectre(),
                ShapeType::ELLIPSE => self.parse_to_draw_ellipse_spectre(),
            },
        }
    }

    fn parse_to_draw_circle_spectre(&mut self) {
        if let (Ok(x), Ok(y), Ok(r), Ok(step), Ok(count)) = (
            self.parse_field::<u32>(self.buf_x.clone()),
            self.parse_field::<u32>(self.buf_y.clone()),
            self.parse_field::<u32>(self.buf_radius.clone()),
            self.parse_field::<u32>(self.buf_step.clone()),
            self.parse_field::<u32>(self.buf_count.clone()),
        ) {
            for i in 0..count {
                self.draw_circle([x as f32, y as f32].into(), (r + (i * step)) as f32);
            }
        } else {
            self.error.enable();
        }
    }

    fn parse_to_draw_ellipse_spectre(&mut self) {
        if self.buf_axe1 == "0" || self.buf_axe2 == "0" {
            self.error.enable();
            return;
        }
        if let (Ok(x), Ok(y), Ok(a1), Ok(a2), Ok(step), Ok(count)) = (
            self.parse_field::<u32>(self.buf_x.clone()),
            self.parse_field::<u32>(self.buf_y.clone()),
            self.parse_field::<u32>(self.buf_axe1.clone()),
            self.parse_field::<u32>(self.buf_axe2.clone()),
            self.parse_field::<u32>(self.buf_step.clone()),
            self.parse_field::<u32>(self.buf_count.clone()),
        ) {
                let y_step = step as f32 * a2 as f32 / a1 as f32;
                for i in 0..count {
                    self.draw_ellipse(
                        [x as f32, y as f32].into(),
                        [(a1 + (i * step)) as f32, (a2 as f32 + (i as f32 * y_step.round()) as f32)].into(),
                    );
                }
        } else {
            self.error.enable();
        }
    }

    fn clear(&mut self) {
        self.shapes.clear();
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((1280.0, 1024.0)),
        default_theme: eframe::Theme::Light,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "Лабораторная работа 4",
        native_options,
        Box::new(|_| Box::<MyApp>::default()),
    )
}

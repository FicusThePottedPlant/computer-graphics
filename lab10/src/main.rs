mod logic;


use std::fmt::Display;
use eframe::egui;
use eframe::egui::{Color32, Pos2};
use epaint::{pos2, Stroke};
use logic::utils::*;
use logic::windows::ErrorWindow;


#[derive(Debug, Default, PartialEq)]
enum FUNCS {
    #[default]
    A,
    B,
    C,
    D,
    E,
    F,
}

impl std::fmt::Display for FUNCS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let func_name = match self {
            FUNCS::A => "(1/5) * sin(x) * cos(z) - (3/2) * cos(7a/4) * exp(-c)",
            FUNCS::B => "z.exp() - x.cos()",
            FUNCS::C => "cos(x) * sin(z)",
            FUNCS::D => "sin(x)",
            FUNCS::E => "cos(z)",
            FUNCS::F => "8 * cos(1.2 * R) / (R + 1), R = sqrt(x^2+z^2)",
        };
        write!(f, "{}", func_name)
    }
}

impl FUNCS {
    fn f(&self) -> fn(f32, f32) -> f32 {
        match self {
            FUNCS::A => |x: f32, z: f32| {
                let pi = std::f32::consts::PI;
                let c = (x - pi) * (x - pi) + (z - pi) * (z - pi);
                (1. / 5.) * x.sin() * z.cos() - (3. / 2.) * (7. * c / 4.).cos() * (-c).exp()
            },
            FUNCS::B => |x: f32, z: f32| {
                z.exp() - x.cos()
            },
            FUNCS::C => |x: f32, z: f32| {
                x.cos() * z.sin()
            },
            FUNCS::D => |x: f32, z: f32| {
                x.sin()
            },
            FUNCS::E => |x: f32, z: f32| {
                z.cos()
            },
            FUNCS::F => |x: f32, z: f32| {
                let r  = (x * x +z * z).sqrt();
                8. * (1.2 * r).cos() / (r + 1.)
            },
            _ => |x: f32, z: f32| {
                    x
                }
        }
    }
}

#[derive(Debug)]
struct MyApp {
    error: ErrorWindow,
    background: Color32,
    graph: Color32,
    func: FUNCS,

    x_r: String,
    y_r: String,
    z_r: String,

    x_start: String,
    x_end: String,
    x_step: String,

    z_start: String,
    z_end: String,
    z_step: String,

    lines: Vec<(Pos2, Pos2)>,

    scale: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            error: Default::default(),
            background: Color32::WHITE,
            graph: Color32::BLACK,
            func: FUNCS::default(),
            x_r: "25".to_string(),
            y_r: "15".to_string(),
            z_r: "0".to_string(),
            x_start: "-6".to_string(),
            x_end: "6".to_string(),
            x_step: "0.1".to_string(),
            z_start: "-6".to_string(),
            z_end: "6".to_string(),
            z_step: "0.1".to_string(),
            lines: vec![],
            scale: "35".to_string()
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
                ui.label("Цвет графика");
                use egui::color_picker::{color_edit_button_srgba, Alpha};

                color_edit_button_srgba(ui, &mut self.graph, Alpha::Opaque);
            });
            ui.label("Функция");
            egui::ComboBox::from_label("")
                .selected_text(self.func.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.func, FUNCS::A, FUNCS::A.to_string());
                    ui.selectable_value(&mut self.func, FUNCS::B, FUNCS::B.to_string());
                    ui.selectable_value(&mut self.func, FUNCS::C, FUNCS::C.to_string());
                    ui.selectable_value(&mut self.func, FUNCS::D, FUNCS::D.to_string());
                    ui.selectable_value(&mut self.func, FUNCS::E, FUNCS::E.to_string());
                    ui.selectable_value(&mut self.func, FUNCS::F, FUNCS::F.to_string());
                });
            ui.separator();
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.x_r).hint_text("Поворот по x").desired_width(100.0));
                // if ui.button("Повернуть").clicked() {}
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.y_r).hint_text("Поворот по y").desired_width(100.0));
                // if ui.button("Повернуть").clicked() {}
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.z_r).hint_text("Поворот по z").desired_width(100.0));
                // if ui.button("Повернуть").clicked() {}
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.vertical(|ui| {
                    ui.label("x");
                    ui.add(egui::TextEdit::singleline(&mut self.x_start).hint_text("X начальное").desired_width(100.0));
                    ui.add(egui::TextEdit::singleline(&mut self.x_end).hint_text("X конечное").desired_width(100.0));
                    ui.add(egui::TextEdit::singleline(&mut self.x_step).hint_text("X шаг").desired_width(100.0));
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.label("z");
                    ui.add(egui::TextEdit::singleline(&mut self.z_start).hint_text("Z начальное").desired_width(100.0));
                    ui.add(egui::TextEdit::singleline(&mut self.z_end).hint_text("Z конечное").desired_width(100.0));
                    ui.add(egui::TextEdit::singleline(&mut self.z_step).hint_text("Z шаг").desired_width(100.0));
                });
            });
            ui.add(egui::TextEdit::singleline(&mut self.scale).hint_text("Масштабирование").desired_width(100.0));
            if ui.button("Построить").clicked() {
                self.run();
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
        painter.extend(self.lines.iter().map(|&(a, b)| {
            egui::Shape::line_segment([a, b], egui::Stroke::new(unit, self.graph))
        }));
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

    fn run(&mut self) {
        if let (Ok(xs), Ok(xe), Ok(xh), Ok(zs), Ok(ze), Ok(zh)) = (
            self.parse_field::<f32>(self.x_start.clone()),
            self.parse_field::<f32>(self.x_end.clone()),
            self.parse_field::<f32>(self.x_step.clone()),
            self.parse_field::<f32>(self.z_start.clone()),
            self.parse_field::<f32>(self.z_end.clone()),
            self.parse_field::<f32>(self.z_step.clone()),
        ) {
            let ax = self.parse_field::<f32>(self.x_r.clone()).unwrap_or(0.0);
            let ay = self.parse_field::<f32>(self.y_r.clone()).unwrap_or(0.0);
            let az = self.parse_field::<f32>(self.z_r.clone()).unwrap_or(0.0);
            let s = self.parse_field::<f32>(self.scale.clone()).unwrap_or(50.0);
            let f = self.func.f();
            let mut hor = Horizont::new((950, 650), f, s);
            let mut res = vec![];
            self.lines.clear();
            hor.horizon_algo(Spacing::new(xs, xe, xh), Spacing::new(zs, ze, zh),&mut res,  ax, ay, az, s);
            for i in res {
                self.lines.push(((i.0 as f32, i.1 as f32).into(), (i.2 as f32, i.3 as f32).into()))
            }
        } else {
            self.error.enable();
        }
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((1280.0, 1024.0)),
        default_theme: eframe::Theme::Light,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "Лабораторная работа 10",
        native_options,
        Box::new(|_| Box::<MyApp>::default()),
    )
}


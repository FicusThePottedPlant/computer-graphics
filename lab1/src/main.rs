use eframe::emath::{Pos2, Vec2b};
use eframe::epaint::Color32;
use eframe::{egui::Ui, Theme};
use egui::Align2;
use egui_extras::{Column, TableBuilder};
use egui_plot::{Plot, Points};

pub mod geo;
use geo::{Point};

#[derive(Default, PartialEq)]
enum Set {
    #[default]
    First,
    Second,
}

#[derive(Default)]
struct MyApp {
    buf_x: String,
    buf_y: String,
    set: Set,
    dots1: Vec<Point>,
    dots2: Vec<Point>,
    show_error: bool,
    edit_buf_x: String,
    edit_buf_y: String,
    show_edit: bool,
    to_edit: usize,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.show_error {
            self.update_error(ctx);
        }
        if self.show_edit {
            self.update_edit(ctx);
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(!self.show_error);
            ui.set_enabled(!self.show_edit);
            ui.horizontal(|ui| {
                self.update_plot(ui);
                ui.separator();
                ui.vertical(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.radio_value(&mut self.set, Set::First, "Множество 1");
                        ui.radio_value(&mut self.set, Set::Second, "Множество 2");
                        if self.set == Set::First && ui.button("Очистка множества 1").clicked() {
                            self.dots2.clear();
                        }
                        if self.set == Set::Second && ui.button("Очистка множества 2").clicked() {
                            self.dots2.clear();
                        }
                    });
                    self.input_coords(ui);
                    self.update_table(ui);
                    self.calculate_triangle(ui);
                });
            });
        });
    }
}

impl MyApp {
    fn update_error(&mut self, ctx: &egui::Context) {
        egui::Window::new("Ошибка при конвертации")
            .default_pos(Pos2 {
                x: 1280.0 / 2.0,
                y: 1024.0 / 2.0 - 200.0,
            })
            .pivot(Align2::CENTER_CENTER)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.label("Недопустимые символы в полях x или y.\nИли поля пустые");
                    if ui.button("Ok").clicked() {
                        self.show_error = false;
                    }
                });
            });
    }

    fn update_edit(&mut self, ctx: &egui::Context) {
        egui::Window::new("Редактирование")
            .default_pos(Pos2 {
                x: 1280.0 / 2.0,
                y: 1024.0 / 2.0 - 200.0,
            })
            .pivot(Align2::CENTER_CENTER)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.label("Редактирование");
                    ui.text_edit_singleline(&mut self.edit_buf_x);
                    ui.text_edit_singleline(&mut self.edit_buf_y);
                });
                ui.horizontal(|ui| {
                    if ui.button("Сохранить").clicked() {
                        if let (Ok(x), Ok(y)) = (
                            self.edit_buf_x.parse::<i32>(),
                            self.edit_buf_y.parse::<i32>(),
                        ) {
                            match self.set {
                                Set::First => self.dots1[self.to_edit] = Point::new(x, y),
                                Set::Second => self.dots2[self.to_edit] = Point::new(x, y),
                            }
                        } else {
                            self.show_error = true;
                        }
                        self.show_edit = false;
                        self.show_error = false;
                        self.edit_buf_x.clear();
                        self.edit_buf_y.clear();
                    }
                });
            });
    }

    fn calculate_triangle(&mut self, ui: &mut Ui) {
        if ui.button("Рассчитать").clicked() {
            println!("{:?}", geo::calc(&self.dots1, &self.dots2));
        }
    }

    fn input_coords(&mut self, ui: &mut Ui) {
        ui.horizontal_top(|ui| {
            ui.text_edit_singleline(&mut self.buf_x);
            ui.text_edit_singleline(&mut self.buf_y);
        });
        if ui.button("Добавить").clicked() {
            if let (Ok(x), Ok(y)) = (self.buf_x.parse::<i32>(), self.buf_y.parse::<i32>()) {
                match self.set {
                    Set::First => self.dots1.push(Point::new(x, y)),
                    Set::Second => self.dots2.push(Point::new(x, y)),
                }
                self.buf_x.clear();
                self.buf_y.clear();
            } else {
                self.show_error = true;
            }
        }
    }

    fn update_table(&mut self, ui: &mut Ui) {
        let table = TableBuilder::new(ui)
            .column(Column::remainder())
            .column(Column::remainder())
            .column(Column::remainder())
            .column(Column::remainder())
            .column(Column::remainder())
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .striped(true)
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
                header.col(|ui| {
                    ui.strong("");
                });
                header.col(|ui| {
                    ui.strong("");
                });
            })
            .body(|mut body| {
                let data;
                match self.set {
                    Set::First => data = self.dots1.clone(),
                    Set::Second => data = self.dots2.clone(),
                }
                for (c, i) in data.iter().enumerate() {
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label(format!("{}", c));
                        });
                        row.col(|ui| {
                            ui.label(format!("{}", i.x));
                        });
                        row.col(|ui| {
                            ui.label(format!("{}", i.y));
                        });
                        row.col(|ui| {
                            if ui.button("Удалить").clicked() {
                                match self.set {
                                    Set::First => self.dots1.remove(c),
                                    Set::Second => self.dots2.remove(c),
                                };
                            };
                        });
                        row.col(|ui| {
                            if ui.button("Изменить").clicked() {
                                self.to_edit = c;
                                self.edit_buf_x = data[self.to_edit].x.to_string();
                                self.edit_buf_y = data[self.to_edit].y.to_string();
                                self.show_edit = true;
                            };
                        });
                    });
                }
            });
    }

    fn update_plot(&mut self, ui: &mut Ui) {
        let plot = Plot::new("plot")
            .data_aspect(1.0)
            .view_aspect(1.0)
            .include_x(-10)
            .include_x(10)
            .include_y(-10)
            .include_y(10)
            .allow_scroll(false)
            .allow_drag(false)
            .allow_zoom(false)
            .allow_boxed_zoom(false)
            .show_axes(true)
            .allow_double_click_reset(false)
            .width(650.0)
            .height(650.0)
            .allow_double_click_reset(false)
            .show_x(false)
            .show_y(false)
            .auto_bounds(Vec2b { x: false, y: false });

        plot.show(ui, |plot_ui| {
            let points1 = Points::new(
                self.dots1
                    .iter()
                    .map(|coord| [coord.x, coord.y])
                    .collect::<Vec<[f64; 2]>>(),
            )
            .color(Color32::RED);
            let points2 = Points::new(
                self.dots2
                    .iter()
                    .map(|coord| [coord.x, coord.y])
                    .collect::<Vec<[f64; 2]>>(),
            )
            .color(Color32::GREEN);
            plot_ui.points(points1.radius(5.0));
            plot_ui.points(points2.radius(5.0));
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((1280.0, 1024.0)),
        default_theme: Theme::Light,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "Lab 1",
        native_options,
        Box::new(|_| Box::<MyApp>::default()),
    )
}

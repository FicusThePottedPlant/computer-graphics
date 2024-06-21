use crate::logic::*;
use eframe::egui;
use eframe::egui::Color32;
use eframe::emath::Align2;
use eframe::{egui::Ui, Theme};
use egui_plot::{Corner, Legend, Line, Plot, Polygon};

pub mod logic;

#[derive(Clone, Debug)]
enum Action {
    Translation(f64, f64),
    Scaling(f64, f64, f64, f64),
    Rotation(f64, f64, f64),
}

impl Action {
    fn reverse(self) -> Self {
        match self {
            Action::Translation(dx, dy) => Action::Translation(-dx, -dy),
            Action::Scaling(kx, ky, cx, cy) => Action::Scaling(1.0 / kx, 1.0 / ky, cx, cy),
            Action::Rotation(a, cx, cy) => Action::Rotation(-a, cx, cy),
        }
    }
}

#[derive(Default, Debug)]
struct ErrorWindow {
    error_title: String,
    error_description: String,
    error_show: bool,
}

impl ErrorWindow {
    pub fn set_error(&mut self, title: String, description: String) {
        self.error_show = true;
        self.error_title = title;
        self.error_description = description;
    }
}

#[derive(Default, Debug)]
struct Figure {
    top: Vec<[f64; 2]>,
    bottom: Vec<[f64; 2]>,
}

#[derive(Default, Debug)]
struct MyApp {
    buf_transfer_x: String,
    buf_transfer_y: String,
    parabola: Vec<[f64; 2]>,
    exp_pos: Vec<[f64; 2]>,
    exp_neg: Vec<[f64; 2]>,
    figure: Figure,
    is_default: bool,
    buf_scale_x: String,
    buf_scale_y: String,
    buf_center_x: String,
    buf_center_y: String,
    buf_angle: String,
    actions_history: Vec<Action>,
    action_state: usize,
    error: ErrorWindow,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.is_default {
            self.set_default(-30.0, 30.0);
            self.is_default = true;
        }
        if self.error.error_show {
            self.update_error(ctx);
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(!self.error.error_show);
            ui.horizontal(|ui| {
                self.update_plot(ui);
                ui.vertical(|ui| {
                    self.transfer(ui);
                    ui.label("Центр операций масштабирования/поворота");
                    ui.add(egui::TextEdit::singleline(&mut self.buf_center_x).hint_text("CX: "));
                    ui.add(egui::TextEdit::singleline(&mut self.buf_center_y).hint_text("CY: "));
                    self.scale(ui);
                    self.rotate(ui);
                    ui.columns(2, |ui| {
                        ui[0].vertical_centered_justified(|ui| {
                            ui.set_enabled(self.action_state != 0);
                            if ui
                                .button("<-")
                                .on_disabled_hover_text("Нельзя продвинуться раньше")
                                .clicked()
                            {
                                let action = self.actions_history[self.action_state - 1].clone();
                                self.fix_convexing(action.clone());
                                self.apply_action(action.clone().reverse());
                                self.action_state -= 1
                            }
                        });
                        ui[1].vertical_centered_justified(|ui| {
                            ui.set_enabled(self.action_state != self.actions_history.len());
                            if ui
                                .button("->")
                                .on_disabled_hover_text("Нельзя продвинуться дальше")
                                .clicked()
                            {
                                let action = self.actions_history[self.action_state].clone();
                                self.fix_convexing(action.clone());
                                self.apply_action(action.clone());
                                self.action_state += 1;
                            }
                        });
                    });
                });
            });
        });
    }
}

impl MyApp {
    fn fix_convexing(&mut self, action: Action) {
        if let Action::Scaling(scale_x, scale_y, _, _) = action {
            if (scale_x < 0.0 || scale_y < 0.0) && scale_x * scale_y <= 0.0 {
                self.figure.top.reverse();
                let last = self.figure.top.remove(0);
                self.figure.top.push(last);
            }
        }
    }

    fn update_error(&mut self, ctx: &egui::Context) {
        egui::Window::new(self.error.error_title.clone())
            .default_pos([1280.0 / 2.0, 1024.0 / 2.0 - 200.0])
            .resizable(false)
            .pivot(Align2::CENTER_CENTER)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(self.error.error_description.clone());
                    if ui.button("Ок").clicked() {
                        self.error.error_show = false;
                    }
                });
            });
    }

    fn push_action_history(&mut self, action: Action) {
        self.apply_action(action.clone());
        if self.actions_history.len() == self.action_state {
            self.actions_history.push(action);
        } else {
            self.actions_history.drain(self.action_state + 1..);
            self.actions_history[self.action_state] = action;
        }
        self.action_state += 1
    }


    fn transfer(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label("Перемещение");
            ui.vertical_centered_justified(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.buf_transfer_x).hint_text("X: "));
                ui.add(egui::TextEdit::singleline(&mut self.buf_transfer_y).hint_text("Y: "));
                ui.set_enabled(
                    !self.buf_transfer_x.is_empty()
                        && !self.buf_transfer_y.is_empty()
                );
                if ui
                    .button("Переместить")
                    .on_disabled_hover_text(format!("Дополните необходимые поля: {:} {:}",
                    if self.buf_transfer_x.is_empty() {"\nПоле перемещения dx;"} else {""},
                    if self.buf_transfer_y.is_empty() {"\nПоле перемещения dy;"} else {""}
                    ))
                    .clicked()
                {
                    let dx = self.buf_transfer_x.parse::<f64>().map_err(|_| {
                        self.error.set_error(
                            "Ошибка перемещения".to_string(),
                            "Некорректное значение в поле X:".to_string(),
                        )
                    });
                    let dy = self.buf_transfer_y.parse::<f64>().map_err(|_| {
                        self.error.set_error(
                            "Ошибка перемещения".to_string(),
                            "Некорректное значение в поле Y".to_string(),
                        )
                    });
                    if let (Ok(dx), Ok(dy)) = (dx, dy) {
                        let action = Action::Translation(dx, dy);
                        self.push_action_history(action);
                    }
                    self.update_plot(ui);
                }
            });
        });
        ui.spacing();
    }

    fn scale(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label("Масштабирование");
            ui.vertical_centered_justified(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.buf_scale_x).hint_text("KX: "));
                ui.add(egui::TextEdit::singleline(&mut self.buf_scale_y).hint_text("KY: "));
                ui.set_enabled(
                    !self.buf_scale_x.is_empty()
                        && !self.buf_scale_y.is_empty()
                        && !self.buf_center_x.is_empty()
                        && !self.buf_center_y.is_empty(),
                );
                if ui
                    .button("Масштабировать")
                    .on_disabled_hover_text(format!("Дополните необходимые поля: {:} {:} {:} {:}",
                                                    if self.buf_scale_x.is_empty() {"\nПоле масштабирования kx;"} else {""},
                                                    if self.buf_scale_y.is_empty() {"\nПоле масштабирования ky;"} else {""},
                                                    if self.buf_center_x.is_empty() {"\nЦентр масштабирования x;"} else {""},
                                                    if self.buf_center_y.is_empty() {"\nЦентр масштабирования y;"} else {""}
                    ))
                    .clicked()
                {
                    let scale_x = self.buf_scale_x.parse::<f64>().map_err(|_| {
                        self.error.set_error(
                            "Ошибка масштабирования".to_string(),
                            "Некорректное значение в поле KX".to_string(),
                        )
                    });
                    let scale_y = self.buf_scale_y.parse::<f64>().map_err(|_| {
                        self.error.set_error(
                            "Ошибка масштабирования".to_string(),
                            "Некорректное значение в поле KY".to_string(),
                        )
                    });
                    let center_x = self.buf_center_x.parse::<f64>().map_err(|_| {
                        self.error.set_error(
                            "Ошибка масштабирования".to_string(),
                            "Некорректное значение в поле CX".to_string(),
                        )
                    });
                    let center_y = self.buf_center_y.parse::<f64>().map_err(|_| {
                        self.error.set_error(
                            "Ошибка масштабирования".to_string(),
                            "Некорректное значение в поле CY".to_string(),
                        )
                    });

                    if let (Ok(scale_x), Ok(scale_y), Ok(center_x), Ok(center_y)) =
                        (scale_x, scale_y, center_x, center_y)
                    {
                        if scale_x == 0.0 || scale_y == 0.0 {
                            self.error.error_description =
                                "Недопустимый параметр масштабирования 0".to_string();
                            self.error.error_title = "Ошибка масштабирования".to_string();
                            self.error.error_show = true;
                        } else {
                            let action = Action::Scaling(scale_x, scale_y, center_x, center_y);
                            self.fix_convexing(action.clone());
                            self.push_action_history(action);
                        }
                    }
                    self.update_plot(ui);
                }
            });
        });
        ui.spacing();
    }

    fn rotate(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label("Поворот");
            ui.vertical_centered_justified(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.buf_angle).hint_text("Угол (градусы): "),
                );
                ui.set_enabled(
                    !self.buf_angle.is_empty()
                        && !self.buf_center_x.is_empty()
                        && !self.buf_center_y.is_empty(),
                );
                if ui
                    .button("Поворот")
                    .on_disabled_hover_text(format!("Дополните необходимые поля: {:} {:} {:}",
                                                    if self.buf_angle.is_empty() {"\nПоле угла поворота;"} else {""},
                                                    if self.buf_center_x.is_empty() {"\nЦентр масштабирования x;"} else {""},
                                                    if self.buf_center_y.is_empty() {"\nЦентр масштабирования y;"} else {""}
                    ))
                    .clicked()
                {
                    let angle = self.buf_angle.parse::<f64>().map_err(|_| {
                        self.error.set_error(
                            "Ошибка поворота".to_string(),
                            "Некорректное значение в поле Угол".to_string(),
                        )
                    });
                    let center_x = self.buf_center_x.parse::<f64>().map_err(|_| {
                        self.error.set_error(
                            "Ошибка поворота".to_string(),
                            "Некорректное значение в поле CX".to_string(),
                        )
                    });
                    let center_y = self.buf_center_y.parse::<f64>().map_err(|_| {
                        self.error.set_error(
                            "Ошибка поворота".to_string(),
                            "Некорректное значение в поле CY".to_string(),
                        )
                    });

                    if let (Ok(angle), Ok(center_x), Ok(center_y)) = (angle, center_x, center_y) {
                        let action = Action::Rotation(angle, center_x, center_y);
                        self.push_action_history(action);
                    }
                    self.update_plot(ui);
                }
            });
        });
        ui.spacing();
    }

    fn apply_action(&mut self, action: Action) {
        for i in itertools::chain!(
            self.parabola.iter_mut(),
            self.exp_pos.iter_mut(),
            self.exp_neg.iter_mut(),
            self.figure.top.iter_mut(),
            self.figure.bottom.iter_mut(),
        ) {
            match action {
                Action::Translation(dx, dy) => {
                    transform(i, dx, dy);
                }
                Action::Scaling(scale_x, scale_y, center_x, center_y) => {
                    scale(i, scale_x, scale_y, center_x, center_y);
                }
                Action::Rotation(angle, center_x, center_y) => {
                    rotate(i, angle, center_x, center_y);
                }
            }
        }
    }

    fn update_plot(&mut self, ui: &mut Ui) {
        let plot = Plot::new("plot")
            .legend(
                Legend::default()
                    .background_alpha(0.0)
                    .position(Corner::LeftBottom),
            )
            .allow_scroll(false)
            .allow_drag(false)
            .allow_zoom(false)
            .allow_boxed_zoom(false)
            .show_axes(true)
            .allow_double_click_reset(false)
            .width(950.0)
            .height(650.0)
            .show_x(false)
            .show_y(false)
            .view_aspect(1.0)
            .data_aspect(1.0)
            .include_x(20)
            .include_x(-20)
            .auto_bounds([false, false].into());

        plot.show(ui, |plot_ui| {
            plot_ui.line(Line::new(self.exp_pos.clone())
                .name("y=x^2"));
            plot_ui.line(Line::new(self.parabola.clone())
                .name("y=exp(x)"));
            plot_ui.line(Line::new(self.exp_neg.clone())
                .name("y=exp(-x)"));
            plot_ui.polygon(
                Polygon::new(self.figure.top.clone())
                    .fill_color(Color32::from_rgba_premultiplied(255, 255, 0, 70))
                    .width(0.0)
                    .name("Фигура"),
            );
            plot_ui.polygon(
                Polygon::new(self.figure.bottom.clone())
                    .fill_color(Color32::from_rgba_premultiplied(255, 255, 0, 70))
                    .width(0.0)
                    .name("Фигура"),
            );
        });
    }

    fn set_default(&mut self, from: f64, to: f64) {
        let from = from as i32;
        let to = to as i32;
        let root = find_root(|x: f64| x * x - x.exp(), -1.0, 0.0, 0.001).unwrap();
        self.parabola = (from * 100..to * 100)
            .map(|i| {
                let x = i as f64 * 0.01;
                [x, x.exp()]
            })
            .collect();
        self.exp_pos = (from * 100..to * 100)
            .map(|i| {
                let x = i as f64 * 0.01;
                [x, x * x]
            })
            .collect();
        self.exp_neg = (from * 100..to * 100)
            .map(|i| {
                let x = i as f64 * 0.01;
                [x, (-x).exp()]
            })
            .collect();
        let left = root * 1000.0;
        let right = -left;
        self.figure.top = (left as i32..=right as i32)
            .rev()
            .map(|i| {
                let x = i as f64 * 0.001;
                [x, if x < 0.0 { x.exp() } else { (-x).exp() }]
            })
            .collect();
        self.figure.top.extend([[0.0, root.exp()]].iter());

        self.figure.bottom = (left as i32..=right as i32)
            .map(|i| {
                let x = i as f64 * 0.001;
                [x, x * x]
            })
            .collect();
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((1280.0, 1024.0)),
        default_theme: Theme::Light,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "Лабораторная работа 2",
        native_options,
        Box::new(|_| Box::<MyApp>::default()),
    )
}

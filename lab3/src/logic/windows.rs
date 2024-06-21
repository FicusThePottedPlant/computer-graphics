use eframe::egui;
use eframe::egui::{Align2, Color32};

#[derive(Default, Debug)]
pub struct ErrorWindow {
    title: String,
    description: String,
    enabled: bool,
}
impl ErrorWindow {
    pub fn set_error(&mut self, title: String, description: String) {
        self.enabled = true;
        self.title = title;
        self.description = description;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn title(&self) -> &String {
        &self.title
    }
    pub fn description(&self) -> &String {
        &self.description
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new(self.title().clone())
            .anchor(Align2::CENTER_CENTER, [0.0; 2])
            .movable(false)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(self.description().clone());
                    if ui.button("Ок").clicked() {
                        self.disable();
                    }
                });
            });
    }
}

#[derive(Default, Debug)]
pub struct HistoWindow {
    enabled: bool,
    data: Vec<(String, u128)>,
}
impl HistoWindow {
    pub fn set_values(&mut self, data: &[(String, u128)]) {
        self.enabled = true;
        self.data = data.into();
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.data.clear();
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Гистограмма".to_owned())
            .anchor(Align2::CENTER_CENTER, [0.0; 2])
            .movable(false)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    egui_plot::Plot::new("Замеры времени")
                        .legend(egui_plot::Legend::default().position(egui_plot::Corner::LeftTop))
                        .width(1000.0)
                        .height(500.0)
                        .allow_scroll(false)
                        .allow_drag(false)
                        .allow_zoom(false)
                        .allow_boxed_zoom(false)
                        .allow_double_click_reset(false)
                        .y_axis_label("Время выполнения (нс)")
                        .show_x(false)
                        .show(ui, |plot_ui| {
                            let colors = [
                                Color32::RED,
                                Color32::LIGHT_GREEN,
                                Color32::BLUE,
                                Color32::KHAKI,
                                Color32::DARK_GREEN,
                                Color32::GRAY,
                            ];
                            for (i, x) in self.data.iter().enumerate() {
                                plot_ui.bar_chart({
                                    egui_plot::BarChart::new(
                                        [egui_plot::Bar::new(i as f64 * 20.0, x.1 as f64)
                                            .width(10.0)
                                            .fill(colors[i])]
                                        .to_vec(),
                                    )
                                    .name(x.0.clone())
                                    .color(colors[i])
                                })
                            }
                        });
                    ui.separator();
                    ui.add_space(20.0);
                    if ui.button("Ок").clicked() {
                        self.disable();
                    }
                });
            });
    }
}

#[derive(Default, Debug)]
pub struct GraphWindow {
    enabled: bool,
    data: Vec<(String, Vec<(i32, i32)>)>,
    len: f32,
}
impl GraphWindow {
    pub fn set_values(&mut self, data: &[(String, Vec<(i32, i32)>)], len: f32) {
        self.enabled = true;
        self.data = data.into();
        self.len = len;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn graph(points: &Vec<(i32, i32)>, color: Color32) -> egui_plot::Line {
        egui_plot::Line::new(egui_plot::PlotPoints::new({
            points
                .into_iter()
                .map(|(x, y)| [x.to_owned() as f64, y.to_owned() as f64])
                .collect::<Vec<[f64; 2]>>()
        }))
        .color(color)
    }

    pub fn plot_graph(
        &self,
        ui: &mut egui::Ui,
        graph: &(String, Vec<(i32, i32)>),
        color32: Color32,
    ) {
        egui_plot::Plot::new(graph.clone())
            .legend(egui_plot::Legend::default().position(egui_plot::Corner::LeftTop))
            .width(300.0)
            .height(250.0)
            .allow_scroll(false)
            .allow_boxed_zoom(false)
            .set_margin_fraction([0.2, 0.2].into())
            .allow_double_click_reset(false)
            .x_axis_label(graph.0.clone())
            .show(ui, |plot_ui| plot_ui.line(Self::graph(&graph.1, color32)));
    }
    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Графики".to_owned())
            .anchor(Align2::CENTER_CENTER, [0.0; 2])
            .movable(false)
            .collapsible(false)
            .resizable(false)
            .max_width(100.0)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.horizontal(|ui| {
                        self.plot_graph(ui, &self.data[0], Color32::RED);
                        self.plot_graph(ui, &self.data[1], Color32::GREEN);
                        // self.plot_graph(ui, &self.data[1], Color32::GREEN);
                    });
                    ui.end_row();
                    ui.horizontal(|ui| {
                        self.plot_graph(ui, &self.data[2], Color32::BLUE);
                        self.plot_graph(ui, &self.data[3], Color32::KHAKI);
                        self.plot_graph(ui, &self.data[4], Color32::BROWN);
                    });
                    ui.end_row();
                    ui.label("Y - количество ступенек; X - Угол наклона (от 0 до 90)");
                    ui.label(format!("Длина {:?}", self.len));
                    if ui.button("Ок").clicked() {
                        self.disable();
                    }
                });
            });
    }
}

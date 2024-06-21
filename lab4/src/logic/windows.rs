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
pub struct GraphWindow {
    enabled: bool,
    data: Vec<(String, Vec<(i32, f64)>)>,
}
impl GraphWindow {
    pub fn set_values(&mut self, data: &[(String, Vec<(i32, f64)>)]) {
        self.enabled = true;
        self.data = data.into();
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

    pub fn graph(points: &(String, Vec<(i32, f64)>), color: Color32) -> egui_plot::Line {
        egui_plot::Line::new(egui_plot::PlotPoints::new({
            points.1.clone()
                .into_iter()
                .map(|(x, y)| [x.to_owned() as f64, y.to_owned() as f64])
                .collect::<Vec<[f64; 2]>>()
        }))
        .color(color).name(points.0.clone())
    }

    pub fn plot_graph(
        &self,
        ui: &mut egui::Ui,
    ) {
        egui_plot::Plot::new("График")
            .legend(egui_plot::Legend::default().position(egui_plot::Corner::LeftTop))
            .width(1000.0)
            .height(500.0)
            .allow_scroll(false)
            .allow_boxed_zoom(false)
            .set_margin_fraction([0.2, 0.2].into())
            .allow_double_click_reset(false)
            .allow_drag(false)
            .x_axis_label("Радиус")
            .y_axis_label("Время (мс)")
            .show(ui, |plot_ui| {
                plot_ui.line(Self::graph(&self.data[0], Color32::RED));
                plot_ui.line(Self::graph(&self.data[1], Color32::GREEN));
                plot_ui.line(Self::graph(&self.data[2], Color32::BLUE));
                plot_ui.line(Self::graph(&self.data[3], Color32::BLACK));
                plot_ui.line(Self::graph(&self.data[4], Color32::GRAY));
            });
    }
    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Графики".to_owned())
            .anchor(Align2::CENTER_CENTER, [0.0; 2])
            .resizable(false)
            .movable(true)
            .max_width(100.0)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    self.plot_graph(ui);
                    if ui.button("Ок").clicked() {
                        self.disable();
                    }
                });
            });
    }
}

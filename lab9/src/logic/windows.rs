use eframe::egui;
use eframe::egui::Align2;

#[derive(Default, Debug)]
pub struct ErrorWindow {
    title: String,
    description: String,
    enabled: bool,
}

impl ErrorWindow {
    pub fn set_error(&mut self, title: String, description: String) -> &mut Self {
        self.enabled = true;
        self.title = title;
        self.description = description;
        self
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

pub struct Interface {
    pub iterations: i32,
    pub value: f32,
    pub exponent:f32,
}

impl Interface {
    pub fn new() -> Self {
        Self {
            iterations: 500,
            value: 2.0,
            exponent: 2.0,
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |_| {
                egui::Window::new("Fractal Playground")
                    .default_open(true)
                    .show(ctx, |ui: &mut egui::Ui| {
                        ui.collapsing("Parameters", |ui| {
                            ui.label("Iterations");
                            ui.add(egui::Slider::new(&mut self.iterations, 0..=3000).text("Iterations"));
                            ui.label("Value");
                            ui.add(egui::Slider::new(&mut self.value, -10.0..=10.0).text("Value"));
                            ui.label("Exponent");
                            ui.add(egui::Slider::new(&mut self.exponent,0.0..=10.0).text("Exponent"));
                        });
                    });
            });
    }
}
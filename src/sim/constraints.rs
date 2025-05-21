#[derive(Clone)]
pub struct CircleConstraint {
    radius: f32,
    elasticity: f32,
}

impl CircleConstraint {
    pub fn new(radius: f32, elasticity: f32) -> Self {
        Self { radius, elasticity }
    }
}

impl super::Constraint for CircleConstraint {
    fn constrain(&self, particle: &mut super::Particle) {
        let dist = particle.position.length() + particle.radius;

        let dist_over = (dist - self.radius).max(0.0);

        let mut pv = particle.position - particle.last_position;

        particle.position -= particle.position * dist_over;

        if dist > self.radius {
            pv = pv.reflect(particle.position.normalize()) * self.elasticity;
        
            particle.last_position = particle.position - pv;
        }
    }

    fn draw_sim(&self, renderer: &dyn super::rendering::SimRenderer, ui: &mut egui::Ui, render_state: &super::rendering::RenderState) {
        const THICKNESS: f32 = 0.025;
        renderer.circle(glam::Vec2::ZERO, self.radius, THICKNESS, egui::Color32::WHITE, ui, render_state);
    }
}

impl super::rendering::RenderableTool for CircleConstraint {
    fn draw(&mut self, ui: &mut egui::Ui) -> egui::InnerResponse<bool> {
        let mut changed = false;
        egui::Frame::group(ui.style())
            .corner_radius(5.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
            
            ui.heading("Circle constraint");
            egui::Grid::new("circle-settings")
                .show(ui, |ui| {
                
                ui.label("Radius:");
                changed |= ui.add(egui::DragValue::new(&mut self.radius).speed(0.01).range(0.0..=f32::INFINITY)).changed();

                ui.end_row();

                ui.label("Elasticity:");
                changed |= ui.add(egui::DragValue::new(&mut self.elasticity).speed(0.01).range(0.0..=1.0)).changed();
            });
            changed
        })
    }
}

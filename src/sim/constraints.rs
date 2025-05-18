#[derive(Clone)]
pub struct CircleConstraint {
    radius: f32
}

impl CircleConstraint {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl super::Constraint for CircleConstraint {
    fn constrain(&self, particle: &mut super::Particle) {
        let dist = particle.position.length() + particle.radius;

        let dist_over = (dist - self.radius).max(0.0);

        let mut pv = particle.position - particle.last_position;

        particle.position -= particle.position * dist_over;

        if dist > self.radius {
            pv = pv.reflect(particle.position.normalize());
        
            particle.last_position = particle.position - pv;
        }
    }

    fn draw(&self, renderer: &dyn super::rendering::SimRenderer, ui: &mut egui::Ui, render_state: &super::rendering::RenderState) {
        const THICKNESS: f32 = 0.025;
        renderer.circle(glam::Vec2::ZERO, self.radius, THICKNESS, egui::Color32::WHITE, ui, render_state);
    }
}

#[derive(Clone)]
pub struct CircleConstraint {
    radius: f32,
    elasticity: f32,
}

#[derive(Clone)]
pub struct HoleCircleConstraint {
    radius: f32,
    open_angle_start: f32,
    open_angle_end: f32,
    //anim_timescale: f32,
    elasticity: f32,
}

impl CircleConstraint {
    pub fn new(radius: f32, elasticity: f32) -> Self {
        Self { radius, elasticity }
    }
    pub fn default() -> Self {
        Self { radius: 1.0, elasticity: 1.0 }
    }
}

impl HoleCircleConstraint {
    const THICKNESS: f32 = 0.025;
    pub fn new(radius: f32, open_angle_start: f32, open_angle_end: f32, /*anim_timescale: f32,*/ elasticity: f32) -> Self {
        Self {
            radius, open_angle_start, open_angle_end, /*anim_timescale,*/ elasticity
        }
    }
    pub fn default() -> Self {
        Self {
            radius: 1.0,
            open_angle_start: 0.2,
            open_angle_end: 0.4,
            elasticity: 1.0
        }
    }
}

impl super::Constraint for CircleConstraint {
    fn constrain(&self, particle: &mut super::Particle) {
        let dist = particle.position.length() + particle.radius;

        let dist_over = (dist - self.radius).max(0.0);

        particle.position -= particle.position * dist_over;

        if dist > self.radius {
            particle.velocity = particle.velocity.reflect(particle.position.normalize()) * self.elasticity;
        
        }
    }

    fn draw_sim(&self, renderer: &dyn super::rendering::SimRenderer, ui: &mut egui::Ui, render_state: &super::rendering::RenderState) {
        const THICKNESS: f32 = 0.025;
        renderer.circle(glam::Vec2::ZERO, self.radius, THICKNESS, egui::Color32::WHITE, ui, render_state);
    }
}

impl super::Constraint for HoleCircleConstraint {
    fn constrain(&self, particle: &mut super::Particle) {
        let pos_len_sq = particle.position.length_squared();
        let last_pos_len_sq = particle.last_position.length_squared();
        let radius_sq_inside = (self.radius - particle.radius - 0.5*Self::THICKNESS) * (self.radius - particle.radius - 0.5*Self::THICKNESS);
        let radius_sq_outside = (self.radius + particle.radius + 0.5*Self::THICKNESS) * (self.radius + particle.radius + 0.5*Self::THICKNESS);

        let hit_inside = pos_len_sq >= radius_sq_inside && last_pos_len_sq < radius_sq_inside;
        let hit_outside = pos_len_sq <= radius_sq_outside && last_pos_len_sq > radius_sq_outside;

        if !(hit_outside || hit_inside) {
            return;
        }

        let pos_dir = particle.position.normalize();

        let start_dir = glam::vec2(self.open_angle_start.cos(), self.open_angle_start.sin());
        let end_dir = glam::vec2(self.open_angle_end.cos(), self.open_angle_end.sin());

        let in_open_arc = if self.open_angle_start < self.open_angle_end {

            let cross_start = start_dir.perp_dot(pos_dir);
            let cross_end = pos_dir.perp_dot(end_dir);
            cross_start >= 0.0 && cross_end >= 0.0
        } else {
            let cross_start = start_dir.perp_dot(pos_dir);
            let cross_end = pos_dir.perp_dot(end_dir);
            cross_start >= 0.0 || cross_end >= 0.0
        };

        if in_open_arc {
            return;
        }

        particle.velocity = particle.velocity.reflect(pos_dir) * self.elasticity;

        particle.position = pos_dir * if hit_inside { radius_sq_inside.sqrt() } else { radius_sq_outside.sqrt() };
    }

    fn draw_sim(&self, renderer: &dyn super::rendering::SimRenderer, ui: &mut egui::Ui, render_state: &super::rendering::RenderState) {
        const SEGMENTS: u32 = 32;

        // Normalize angles to [0, TAU)
        use std::f32::consts::TAU;
        let start_angle = self.open_angle_end.rem_euclid(TAU);
        let mut end_angle = self.open_angle_start.rem_euclid(TAU);

        // Handle arc wrapping: if end < start, add TAU to end for smooth loop
        if end_angle < start_angle {
            end_angle += TAU;
        }

        let step = (end_angle - start_angle) / SEGMENTS as f32;
        let mut theta = start_angle;

        for _ in 0..SEGMENTS {
            let last_pos = glam::vec2(theta.cos(), theta.sin()) * (self.radius);
            theta += step;
            let this_pos = glam::vec2((theta + 0.01).cos(), (theta + 0.01).sin()) * (self.radius);

            renderer.line_segment(last_pos, this_pos, 0.025, egui::Color32::WHITE, ui, render_state);
        }
    }
}

impl super::rendering::RenderableTool for CircleConstraint {
    fn draw(&mut self, ui: &mut egui::Ui, id_salt: &mut u32) -> egui::InnerResponse<(bool, bool)> {
        let mut changed = false;
        let mut remove = false;
        egui::Frame::group(ui.style())
            .corner_radius(5.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
            
            ui.horizontal(|ui| {
                ui.heading("Circle");

                remove = ui.button("X").on_hover_text("Remove").clicked();
            });

            egui::Grid::new(format!("circle-settings{}", id_salt))
                .show(ui, |ui| {
                
                ui.label("Radius:");
                changed |= ui.add(egui::DragValue::new(&mut self.radius).speed(0.01).range(0.0..=f32::INFINITY)).changed();

                ui.end_row();

                ui.label("Elasticity:");
                changed |= ui.add(egui::DragValue::new(&mut self.elasticity).speed(0.01).range(0.0..=1.0)).changed();
            });
            *id_salt += 1;
            (changed, remove)
        })
    }
}

impl super::rendering::RenderableTool for HoleCircleConstraint {
    fn draw(&mut self, ui: &mut egui::Ui, id_salt: &mut u32) -> egui::InnerResponse<(bool, bool)> {
        let mut changed = false;
        let mut remove = false;
        egui::Frame::group(ui.style())
            .corner_radius(5.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
            
            ui.horizontal(|ui| {
                ui.heading("Circle With Hole");

                remove = ui.button("X").on_hover_text("Remove").clicked();
            });

            egui::Grid::new(format!("circle-hole-settings{}", id_salt))
                .show(ui, |ui| {
                
                ui.label("Radius:");
                changed |= ui.add(egui::DragValue::new(&mut self.radius).speed(0.01).range(0.0..=f32::INFINITY)).changed();

                ui.label("Open angle:");
                changed |= ui.add(egui::DragValue::new(&mut self.open_angle_start).speed(0.01).range(0.0..=2.0*std::f32::consts::PI).prefix("Start:")).changed();
                changed |= ui.add(egui::DragValue::new(&mut self.open_angle_end).speed(0.01).range(0.0..=2.0*std::f32::consts::PI).prefix("End:")).changed();

                ui.end_row();

                ui.label("Elasticity:");
                changed |= ui.add(egui::DragValue::new(&mut self.elasticity).speed(0.01).range(0.0..=1.0)).changed();
            });
            *id_salt += 1;
            (changed, remove)
        }) 
    }
}

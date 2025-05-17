pub struct Viewport {
    pub sim_units_per_vw: f32
}

impl Viewport {
    pub fn sim_units_to_logical_points(&self, sim_units: f32, vw_points: f32) -> f32 {
        let logical_points_per_sim_unit = vw_points / self.sim_units_per_vw;
        sim_units * logical_points_per_sim_unit
    }
}

pub trait SimRenderer {
    fn render(&self, sim: &super::SimulationRenderState, ui: &mut egui::Ui);
}

pub struct CpuSimRenderer {
    pub viewport: Viewport
}

impl CpuSimRenderer {
    pub fn new() -> Self {
        Self {
            viewport: Viewport { sim_units_per_vw: 1.0 }
        }
    }
}

impl SimRenderer for CpuSimRenderer {
    fn render(&self, sim: &super::SimulationRenderState, ui: &mut egui::Ui) {
        let vw = ui.available_width();
        let vh = ui.available_height();

        let (rect, _) = ui.allocate_exact_size(egui::vec2(vw, vh), egui::Sense::empty());

        for particle in &sim.particles {
            let mut pos = egui::pos2(particle.position.x, particle.position.y);
            pos.x = self.viewport.sim_units_to_logical_points(pos.x, vw) + rect.center().x;
            pos.y = self.viewport.sim_units_to_logical_points(pos.y, vw) + rect.center().y;

            let radius = self.viewport.sim_units_to_logical_points(particle.radius, vw);

            ui.painter().circle_filled(pos, radius, particle.color);
        }
    }
}

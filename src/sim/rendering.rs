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
    fn render(&self, sim: &super::SimulationState, ui: &mut egui::Ui);

    fn line_segment(&self, a: glam::Vec2, b: glam::Vec2, thickness: f32, color: egui::Color32, ui: &mut egui::Ui, render_state: &RenderState);
    fn circle(&self, center: glam::Vec2, radius: f32, thickness: f32, color: egui::Color32, ui: &mut egui::Ui, render_state: &RenderState);
    fn circle_filled(&self, center: glam::Vec2, radius: f32, color: egui::Color32, ui: &mut egui::Ui, render_state: &RenderState);
}

pub trait RenderableTool {
    fn draw(&mut self, ui: &mut egui::Ui, id_salt: &mut u32) -> egui::InnerResponse<(bool, bool)>;
}

pub struct RenderState {
    center: egui::Pos2,
    vw: f32
}

pub struct CpuSimRenderer {
    pub viewport: Viewport,
}

impl CpuSimRenderer {
    pub fn new() -> Self {
        Self {
            viewport: Viewport { sim_units_per_vw: 2.0 }
        }
    }
}

impl SimRenderer for CpuSimRenderer {
    fn render(&self, sim: &super::SimulationState, ui: &mut egui::Ui) {
        let vw = ui.available_width();
        let vh = ui.available_height();

        let (rect, _) = ui.allocate_exact_size(egui::vec2(vw, vh), egui::Sense::empty());

        let render_state = RenderState { center: rect.center(), vw };

        for particle in &sim.particles {
            let mut pos = egui::pos2(particle.position.x, particle.position.y);
            pos.x = self.viewport.sim_units_to_logical_points(pos.x, vw) + rect.center().x;
            pos.y = self.viewport.sim_units_to_logical_points(pos.y, vw) + rect.center().y;

            let radius = self.viewport.sim_units_to_logical_points(particle.radius, vw);

            ui.painter().circle_filled(pos, radius, particle.color);
        }

        for constraint in &sim.constraints {
            constraint.draw_sim(self, ui, &render_state);
        }
    }

    fn line_segment(&self, a: glam::Vec2, b: glam::Vec2, thickness: f32, color: egui::Color32, ui: &mut egui::Ui, render_state: &RenderState) {
        let vw = render_state.vw;
        let c = render_state.center;

        let a_x = self.viewport.sim_units_to_logical_points(a.x, vw) + c.x;
        let a_y = self.viewport.sim_units_to_logical_points(a.y, vw) + c.y;
        
        let b_x = self.viewport.sim_units_to_logical_points(b.x, vw) + c.x;
        let b_y = self.viewport.sim_units_to_logical_points(b.y, vw) + c.y;

        let thickness = self.viewport.sim_units_to_logical_points(thickness, vw);

        ui.painter().line_segment([egui::pos2(a_x, a_y), egui::pos2(b_x, b_y)], egui::Stroke::new(thickness, color));
    }

    fn circle(&self, center: glam::Vec2, radius: f32, thickness: f32, color: egui::Color32, ui: &mut egui::Ui, render_state: &RenderState) {
        let vw = render_state.vw;
        let c = render_state.center;

        let c_x = self.viewport.sim_units_to_logical_points(center.x, vw) + c.x;
        let c_y = self.viewport.sim_units_to_logical_points(center.y, vw) + c.y;
        
        let radius = self.viewport.sim_units_to_logical_points(radius, vw);

        let thickness = self.viewport.sim_units_to_logical_points(thickness, vw);

        ui.painter().circle_stroke(egui::pos2(c_x, c_y), radius, egui::Stroke::new(thickness, color)); 
    }

    fn circle_filled(&self, center: glam::Vec2, radius: f32, color: egui::Color32, ui: &mut egui::Ui, render_state: &RenderState) {
        let vw = render_state.vw;
        let c = render_state.center;
        
        let c_x = self.viewport.sim_units_to_logical_points(center.x, vw) + c.x;
        let c_y = self.viewport.sim_units_to_logical_points(center.y, vw) + c.y;
        
        let radius = self.viewport.sim_units_to_logical_points(radius, vw);

        ui.painter().circle_filled(egui::pos2(c_x, c_y), radius, color);  
    }
}

use super::rendering;

pub trait SimEvent: Send + dyn_clone::DynClone + rendering::RenderableTool {
    fn trigger(&self, sim: &mut super::SimulationState);
}
dyn_clone::clone_trait_object!(SimEvent);

pub trait SimTrigger: Send + dyn_clone::DynClone + rendering::RenderableTool {
    fn is_triggered(&self, sim: &super::SimulationState) -> bool;
}
dyn_clone::clone_trait_object!(SimTrigger);

#[derive(Clone)]
pub struct TriggerManager {
    trigger: Box<dyn SimTrigger>,
    events: Vec<Box<dyn SimEvent>>
}

impl TriggerManager {
    pub fn new(trigger: Box<dyn SimTrigger>, events: Vec<Box<dyn SimEvent>>) -> Self {
        Self {
            trigger, events
        }
    }

    pub fn process(&self, sim: &mut super::SimulationState) {
        if self.trigger.is_triggered(sim) {
            for event in &self.events {
                event.trigger(sim);
            }
        }
    }
}

impl rendering::RenderableTool for TriggerManager {
    fn draw(&mut self, ui: &mut egui::Ui) -> egui::InnerResponse<bool> {
        egui::Frame::group(ui.style())
            .corner_radius(5.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
            let mut changed = false;

            ui.vertical(|ui| {
                ui.label("Trigger");
                changed |= self.trigger.draw(ui).inner;
            
                ui.label("Events");
                for event in &mut self.events {
                    changed |= event.draw(ui).inner;
                }

            });
            changed
        })
    }
}

#[derive(Clone)]
pub struct SpawnEvent {
    pub particle: super::Particle
}

impl SimEvent for SpawnEvent {
    fn trigger(&self, sim: &mut super::SimulationState) {
        sim.add_particle(self.particle.clone());
    }
}


impl rendering::RenderableTool for SpawnEvent {
    fn draw(&mut self, ui: &mut egui::Ui) -> egui::InnerResponse<bool> {
        let mut changed = false;

        egui::Frame::group(ui.style())
            .fill(egui::Color32::from_gray(30))
            .corner_radius(5.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
            
            ui.heading("Spawn Particle");
            egui::Grid::new("particle-settings")
                .show(ui, |ui| {

                let mut vx = (self.particle.position.x - self.particle.last_position.x) * 60.0;
                let mut vy = (self.particle.position.y - self.particle.last_position.y) * 60.0;

                ui.label("Position");
                changed |= ui.add(egui::DragValue::new(&mut self.particle.position.x).prefix("X:").speed(0.01)).changed();
                changed |= ui.add(egui::DragValue::new(&mut self.particle.position.y).prefix("Y:").speed(0.01)).changed();
                ui.end_row();

                ui.label("Velocity");

                changed |= ui.add(egui::DragValue::new(&mut vx).prefix("X:").speed(0.01)).changed();
                changed |= ui.add(egui::DragValue::new(&mut vy).prefix("Y:").speed(0.01)).changed();

                self.particle.last_position.x = self.particle.position.x - vx/60.0;
                self.particle.last_position.y = self.particle.position.y - vy/60.0;
                ui.end_row();

                ui.label("Radius");
                changed |= ui.add(egui::DragValue::new(&mut self.particle.radius).speed(0.01)).changed();
                ui.end_row();
                
                ui.label("Color");

                let mut hsva: egui::epaint::Hsva = crate::util::color32_to_hsva(self.particle.color);

                changed |= ui.color_edit_button_hsva(&mut hsva).changed();

                self.particle.color = crate::util::hsva_to_color32(hsva);
            });
            changed
        })

    }
}

#[derive(Clone)]
pub struct AnyLeftCircleTrigger {
    radius: f32
}

impl AnyLeftCircleTrigger {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl SimTrigger for AnyLeftCircleTrigger {
    fn is_triggered(&self, sim: &super::SimulationState) -> bool {
        for particle in &sim.particles {
            if particle.position.length() > self.radius && particle.last_position.length() <= self.radius {
                return true;
            }
        }

        false
    }
}

impl rendering::RenderableTool for AnyLeftCircleTrigger {
    fn draw(&mut self, ui: &mut egui::Ui) -> egui::InnerResponse<bool> {
        let mut changed = false;

        egui::Frame::group(ui.style())
            .corner_radius(5.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
            
            ui.heading("Any particle left circular bound");
            
            egui::Grid::new("circle-settings")
                .show(ui, |ui| {
                ui.label("Radius:");
                changed |= ui.add(egui::DragValue::new(&mut self.radius).speed(0.01)).changed();
            });
            changed
        })

    }
}

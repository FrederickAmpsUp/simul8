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
    events: Vec<Box<dyn SimEvent>>,

    selected_event: String,
    new_event: Option<Box<dyn SimEvent>>
}

impl TriggerManager {
    pub fn new(trigger: Box<dyn SimTrigger>, events: Vec<Box<dyn SimEvent>>) -> Self {
        Self {
            trigger, events,
            selected_event: String::new(),
            new_event: None
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
    fn draw(&mut self, ui: &mut egui::Ui, id_salt: &mut u32) -> egui::InnerResponse<(bool, bool)> {
        egui::Frame::group(ui.style())
            .corner_radius(5.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
            let mut changed = false;
            let mut remove = false;

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Trigger");

                    ui.with_layout(egui::Layout::left_to_right(egui::Align::RIGHT), |ui| {
                        remove = ui.button("X").on_hover_text("Remove").clicked();
                    });
                });
                changed |= self.trigger.draw(ui, id_salt).inner.0;
           
                ui.horizontal(|ui| {
                    ui.label("Events");

                    ui.separator();

                    egui::ComboBox::new("event-selector", "")
                        .selected_text(self.selected_event.clone())
                        .show_ui(ui, |ui| {
                        
                        if ui.selectable_value(&mut self.selected_event, "Spawn Particle".into(), "Spawn Particle").clicked() {
                            self.new_event = Some(Box::new(
                                crate::sim::event::SpawnEvent { particle: crate::sim::Particle::new(glam::Vec2::ZERO, 0.05, egui::Color32::RED) }
                            ));
                        }
                    });

                    if ui.button("+ Add").clicked() {
                        if let Some(e) = &self.new_event {
                            self.events.push(e.clone());
                        }
                    }
                });
                let mut i = 0usize;
                let mut remove = None; 
                for event in &mut self.events {
                    let res = event.draw(ui, id_salt).inner;
                    changed |= res.0;

                    if res.1 {
                        remove = Some(i);
                    }
                    i += 1;
                }
                if let Some(r) = remove {
                    self.events.remove(r);
                    changed = true;
                }
            });
            (changed, remove)
        })
    }
}

#[derive(Clone)]
pub struct SpawnEvent {
    pub particle: super::Particle,
}

impl SimEvent for SpawnEvent {
    fn trigger(&self, sim: &mut super::SimulationState) {
        sim.add_particle(self.particle.clone());
    }
}


impl rendering::RenderableTool for SpawnEvent {
    fn draw(&mut self, ui: &mut egui::Ui, id_salt: &mut u32) -> egui::InnerResponse<(bool, bool)> {
        let mut changed = false;
        let mut remove = false;

        egui::Frame::group(ui.style())
            .fill(egui::Color32::from_gray(30))
            .corner_radius(5.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
            
            ui.horizontal(|ui| {
                ui.heading("Spawn Particle");

                remove = ui.button("X").on_hover_text("Remove").clicked();
            });

            egui::Grid::new(format!("particle-settings{}", id_salt))
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
            *id_salt += 1;
            (changed, remove)
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
    fn draw(&mut self, ui: &mut egui::Ui, id_salt: &mut u32) -> egui::InnerResponse<(bool, bool)> {
        let mut changed = false;

        egui::Frame::group(ui.style())
            .corner_radius(5.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
            
            ui.heading("Any particle left circular bound");
           
            egui::Grid::new(format!("circle-settings{}", id_salt))
                .show(ui, |ui| {
                ui.label("Radius:");
                changed |= ui.add(egui::DragValue::new(&mut self.radius).speed(0.01)).changed();
            });
            *id_salt += 1;
            (changed, false) // TODO: remove
        })

    }
}

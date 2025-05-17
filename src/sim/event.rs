pub trait SimEvent: Send {
    fn trigger(&self, sim: &mut super::SimulationState);
}

pub trait SimTrigger: Send {
    fn is_triggered(&self, sim: &super::SimulationState) -> bool;
}

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

// will make this configurable in the future
pub struct SpawnEvent;

impl SimEvent for SpawnEvent {
    fn trigger(&self, sim: &mut super::SimulationState) {
        sim.add_particle(super::Particle::new(
            glam::Vec2::ZERO,
            0.05,
            egui::Color32::CYAN
        ));
    }
}

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
            if particle.position.length() > self.radius {
                return true;
            }
        }

        false
    }
}

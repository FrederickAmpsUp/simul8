pub mod rendering;
pub mod event;
pub mod constraints;

#[derive(Clone)]
pub struct Particle {
    pub position: glam::Vec2,
    pub last_position: glam::Vec2,
    pub radius: f32,
    pub color: egui::Color32
}

impl Particle {
    pub fn new(position: glam::Vec2, radius: f32, color: egui::Color32) -> Self {
        Self {
            position,
            last_position: position,
            radius, color
        }
    }
}

pub trait ConstraintClone {
    fn clone_box(& self) -> Box<dyn Constraint>;
}

pub trait Constraint: Send + ConstraintClone {
    fn constrain(&self, particle: &mut Particle);
    fn draw(&self, _renderer: &dyn rendering::SimRenderer, _ui: &mut egui::Ui, _r: &rendering::RenderState) {}
}

impl<T> ConstraintClone for T
where T: Constraint + Clone + 'static
{
    fn clone_box(& self) -> Box<dyn Constraint> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Constraint> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub struct SimulationState {
    particles: Vec<Particle>,
    constraints: Vec<Box<dyn Constraint>>,
    
    trigger_managers: Vec<event::TriggerManager>,

    pub gravity_accel: glam::Vec2,

    sim_render_tx: Option<crate::util::OverwriteSlot<SimulationRenderState>>
}

pub struct SimulationRenderState {
    particles: Vec<Particle>,
    constraints: Vec<Box<dyn Constraint>>,
}

impl SimulationState {
    pub fn new(sim_render_tx: Option<crate::util::OverwriteSlot<SimulationRenderState>>) -> Self {
        Self {
            particles: vec![],
            constraints: vec![],
            trigger_managers: vec![],
            gravity_accel: glam::Vec2::ZERO,
            sim_render_tx
        }
    }

    pub fn add_particle(&mut self, particle: Particle) {
        self.particles.push(particle);
    }
    pub fn add_constraint(&mut self, constraint: impl Constraint + 'static) {
        self.constraints.push(Box::new(constraint));
    }

    pub fn add_trigger_manager(&mut self, manager: event::TriggerManager) {
        self.trigger_managers.push(manager);
    }

    fn solve_constraints(&mut self, steps: u32) {
        for _ in 0..steps {
            for constraint in &self.constraints {
                for particle in &mut self.particles {
                    constraint.constrain(particle);
                }
            }
        }
    }

    fn solve_pbd(&mut self, dt: f32) {
        for particle in &mut self.particles {
            let v = (particle.position - particle.last_position) / dt + dt * self.gravity_accel;

            particle.last_position = particle.position;
            particle.position += dt * v;
        }

        self.solve_constraints(1);
    }

    fn update_triggers(&mut self) {
        let tms: Vec<event::TriggerManager> = self.trigger_managers.drain(..).collect();
        for tm in &tms {
            tm.process(self);
        }
        self.trigger_managers = tms;
    }

    fn step(&mut self, dt: f32) {
        self.update_triggers();
        self.solve_pbd(dt);
    }

    fn update_render_state(&mut self) {
        if let Some(tx) = &self.sim_render_tx {
            let state = self.get_render_state();
            tx.write(state);
        }
    }
    
    pub fn single_step(&mut self, dt: f32) {
        self.step(dt);
        self.update_render_state();
    }

    pub fn multi_step(&mut self, steps: u32, dt: f32) {
        for _ in 0..steps {
            self.step(dt / steps as f32);
        }
        self.update_render_state();
    }

    pub fn get_render_state(&self) -> SimulationRenderState {
        SimulationRenderState { particles: self.particles.clone(), constraints: self.constraints.clone() }
    }
}

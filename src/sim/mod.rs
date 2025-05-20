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

pub trait Constraint: Send + dyn_clone::DynClone {
    fn constrain(&self, particle: &mut Particle);
    fn draw(&self, _renderer: &dyn rendering::SimRenderer, _ui: &mut egui::Ui, _r: &rendering::RenderState) {}
}
dyn_clone::clone_trait_object!(Constraint);

#[derive(Clone)]
pub struct SimulationState {
    particles: Vec<Particle>,
    constraints: Vec<Box<dyn Constraint>>,
    
    trigger_managers: Vec<event::TriggerManager>,

    pub gravity_accel: glam::Vec2,
}

pub enum SimulationCommand {

}

pub struct SimulationInterface {
    manager_tx: futures::channel::mpsc::Sender<SimulationCommand>,
    manager_rx: futures::channel::mpsc::Receiver<SimulationCommand>
}
pub struct SimulationManager {
    frame_cache: Vec<SimulationState>,

    fps: f32,

    interface_tx: futures::channel::mpsc::Sender<SimulationCommand>,
    interface_rx: futures::channel::mpsc::Receiver<SimulationCommand>
}

impl SimulationManager {
    fn new(interface_tx: futures::channel::mpsc::Sender<SimulationCommand>, interface_rx: futures::channel::mpsc::Receiver<SimulationCommand>) -> Self {
        Self {
            frame_cache: vec![],
            fps: 60.0,
            interface_tx, interface_rx
        }
    }

    fn run_frame(&mut self) {
        let mut last_frame = (self.frame_cache.last()).unwrap_or(&SimulationState::new()).clone();

        last_frame.single_step(1.0 / self.fps);

        self.frame_cache.push(last_frame);
    }

    fn get_frame(&mut self, frame: usize) -> &SimulationState {
        while self.frame_cache.len() <= frame {
            self.run_frame();
        }

        &self.frame_cache[frame]
    }

    fn get_frame_mut(&mut self, frame: usize) -> &mut SimulationState {
        self.frame_cache.truncate(frame+1);

        while self.frame_cache.len() <= frame {
            self.run_frame();
        }

        &mut self.frame_cache[frame]
    }
}

impl SimulationState {
    pub fn new() -> Self {
        Self {
            particles: vec![],
            constraints: vec![],
            trigger_managers: vec![],
            gravity_accel: glam::Vec2::ZERO,
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

    pub fn single_step(&mut self, dt: f32) {
        self.step(dt);
    }

    pub fn multi_step(&mut self, steps: u32, dt: f32) {
        for _ in 0..steps {
            self.step(dt / steps as f32);
        }
    }
}

pub mod rendering;

#[derive(Clone)]
pub struct Particle {
    pub position: glam::Vec2,
    pub last_position: glam::Vec2,
    pub radius: f32,
    pub color: egui::Color32
}

pub trait Constraint: Send {
    fn constrain(&self, particle: &mut Particle);
}

pub struct SimulationState {
    particles: Vec<Particle>,
    constraints: Vec<Box<dyn Constraint>>,
    
    pub gravity_accel: glam::Vec2,

    sim_render_tx: Option<crate::util::OverwriteSlot<SimulationRenderState>>
}

pub struct SimulationRenderState {
    particles: Vec<Particle>
}

impl SimulationState {
    pub fn new(sim_render_tx: Option<crate::util::OverwriteSlot<SimulationRenderState>>) -> Self {
        Self {
            particles: vec![],
            constraints: vec![],
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

    fn step(&mut self, dt: f32) {
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
        SimulationRenderState { particles: self.particles.clone() }
    }
}

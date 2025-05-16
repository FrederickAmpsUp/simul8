pub struct Particle {
    pub position: glam::Vec2,
    pub last_position: glam::Vec2,
    pub radius: f32,
    pub color: egui::Color32
}

pub trait Constraint {
    fn constrain(&self, particle: &mut Particle);
}

pub mod rendering;

pub struct SimulationState<'a> {
    particles: Vec<Particle>,
    constraints: Vec<Box<dyn Constraint + 'a>>,
    
    gravity_accel: glam::Vec2
}

impl<'a> SimulationState<'a> {
    pub fn new() -> Self {
        Self {
            particles: vec![],
            constraints: vec![],
            gravity_accel: glam::Vec2::ZERO
        }
    }

    pub fn add_particle(&mut self, particle: Particle) {
        self.particles.push(particle);
    }
    pub fn add_constraint(&mut self, constraint: impl Constraint + 'a) {
        self.constraints.push(Box::new(constraint));
    }

    pub fn solve_constraints(&mut self, steps: u32) {
        for _ in 0..steps {
            for constraint in &self.constraints {
                for particle in &mut self.particles {
                    constraint.constrain(particle);
                }
            }
        }
    }

    pub fn solve_pbd(&mut self, dt: f32) {
        for particle in &mut self.particles {
            let v = (particle.position - particle.last_position) / dt + dt * self.gravity_accel;

            particle.last_position = particle.position;
            particle.position += dt * v;
        }

        self.solve_constraints(1);
    }
}

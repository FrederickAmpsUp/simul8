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
    RequestFrame(u32),
    StoreFrame(u32, SimulationState)
}

pub enum SimulationResponse {
    Frame(u32, SimulationState)
}

pub struct SimulationInterface {
    manager_tx: futures::channel::mpsc::Sender<SimulationCommand>,
    manager_rx: futures::channel::mpsc::Receiver<SimulationResponse>,

    frame_cache: std::collections::BTreeMap<u32, SimulationState>
}

pub struct SimulationManager {
    frame_cache: Vec<SimulationState>,

    fps: f32,

    interface_tx: futures::channel::mpsc::Sender<SimulationResponse>,
    interface_rx: futures::channel::mpsc::Receiver<SimulationCommand>
}

impl SimulationInterface {
    pub fn new(manager_tx: futures::channel::mpsc::Sender<SimulationCommand>, manager_rx: futures::channel::mpsc::Receiver<SimulationResponse>) -> Self {
        Self {
            manager_tx, manager_rx, frame_cache: std::collections::BTreeMap::new()
        }
    }

    pub fn load_frame(&mut self, frame: u32) {
        use futures::SinkExt;
        let request = SimulationCommand::RequestFrame(frame);

        let mut tx = self.manager_tx.clone();

        crate::util::spawn(async move { let _ = tx.send(request).await; () });
    }

    pub fn store_frame(&mut self, frame: u32, state: SimulationState) {
        use futures::SinkExt;
        let request = SimulationCommand::StoreFrame(frame, state);

        let mut tx = self.manager_tx.clone();

        crate::util::spawn(async move { let _ = tx.send(request).await; log::info!("Requested!"); () });
    }

    pub fn process_requests(&mut self) {
        loop {
            if let Ok(Some(res)) = self.manager_rx.try_next() {
                match res {
                    SimulationResponse::Frame(idx, frame) => {
                        let _ = self.frame_cache.insert(idx, frame);
                    },
                    #[allow(unreachable_patterns)]
                    _ => log::warn!("Unhandled response!")
                }
            } else { break; }
        }
    }

    pub fn try_get_frame(&mut self, frame: u32) -> Option<&SimulationState> {
        self.frame_cache.get(&frame)
    }
}

impl SimulationManager {
    pub fn new(interface_tx: futures::channel::mpsc::Sender<SimulationResponse>, interface_rx: futures::channel::mpsc::Receiver<SimulationCommand>) -> Self {
        Self {
            frame_cache: vec![],
            fps: 60.0,
            interface_tx, interface_rx
        }
    }

    pub fn run_frame(&mut self) {
        let mut last_frame = (self.frame_cache.last()).unwrap_or(&SimulationState::new()).clone();

        last_frame.single_step(1.0 / self.fps);

        self.frame_cache.push(last_frame);
    }

    pub fn get_frame(&mut self, frame: u32) -> &SimulationState {
        while self.frame_cache.len() as u32 <= frame {
            self.run_frame();
        }

        &self.frame_cache[frame as usize]
    }

    pub fn get_frame_mut(&mut self, frame: u32) -> &mut SimulationState {
        self.frame_cache.truncate(frame as usize + 1);

        while self.frame_cache.len() as u32 <= frame {
            self.run_frame();
        }

        &mut self.frame_cache[frame as usize]
    }

    pub fn process_requests(&mut self) {
        loop {
            if let Ok(Some(cmd)) = self.interface_rx.try_next() {
                match cmd {
                    SimulationCommand::RequestFrame(frame_idx) => {
                        use futures::SinkExt;

                        let frame = self.get_frame(frame_idx).clone();

                        let mut tx = self.interface_tx.clone();
                        let res = SimulationResponse::Frame(frame_idx, frame);

                        crate::util::spawn(async move { let _ = tx.send(res).await; () });
                    },
                    SimulationCommand::StoreFrame(frame_idx, state) => {
                        let frame = self.get_frame_mut(frame_idx);
                        *frame = state;
                        log::info!("Stored frame {}", frame_idx);
                    }
                    #[allow(unreachable_patterns)]
                    _ => log::warn!("Unhandled simulation command !")
                }
            } else {
                break;
            }
        }
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
        log::info!("Step!");
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

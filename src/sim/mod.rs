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

pub trait Constraint: Send + dyn_clone::DynClone + rendering::RenderableTool {
    fn constrain(&self, particle: &mut Particle);
    fn draw_sim(&self, _renderer: &dyn rendering::SimRenderer, _ui: &mut egui::Ui, _r: &rendering::RenderState) {}
}
dyn_clone::clone_trait_object!(Constraint);

#[derive(Clone)]
pub struct SimulationState {
    pub particles: Vec<Particle>,
    pub constraints: Vec<Box<dyn Constraint>>,
    
    pub trigger_managers: Vec<event::TriggerManager>,

    pub gravity_accel: glam::Vec2,
}

pub enum SimulationCommand {
    RequestFrame(u32),
    StoreFrame(u32, SimulationState),
    GetCached,
    ClearCache
}

pub enum SimulationResponse {
    Frame(u32, SimulationState),
    Cached(u32)
}

pub struct SimulationInterface {
    manager_tx: flume::Sender<SimulationCommand>,
    manager_rx: flume::Receiver<SimulationResponse>,

    frame_cache: std::collections::BTreeMap<u32, SimulationState>,
    manager_cached: u32
}

pub struct SimulationManager {
    frame_cache: Vec<SimulationState>,

    fps: f32,

    interface_tx: flume::Sender<SimulationResponse>,
    interface_rx: flume::Receiver<SimulationCommand>
}

trait EasySend<T> {
    fn ez_send(&mut self, data: T);
}

impl<T: Send + 'static> EasySend<T> for flume::Sender<T> {
    fn ez_send(&mut self, data: T) {
        let _ = self.send(data);
    }
}

impl SimulationInterface {
    pub fn new(manager_tx: flume::Sender<SimulationCommand>, manager_rx: flume::Receiver<SimulationResponse>) -> Self {
        Self {
            manager_tx, manager_rx, frame_cache: std::collections::BTreeMap::new(),
            manager_cached: 0
        }
    }

    pub fn load_frame(&mut self, frame: u32) {
        let request = SimulationCommand::RequestFrame(frame);

        self.manager_tx.ez_send(request);
    }

    pub fn load_cached(&mut self) {
        self.manager_tx.ez_send(SimulationCommand::GetCached);
    }

    pub fn store_frame(&mut self, frame: u32, state: SimulationState) {
        let request = SimulationCommand::StoreFrame(frame, state);
        self.frame_cache.split_off(&(frame + 1));
        self.manager_tx.ez_send(request);
    }

    pub fn clear_frame_cache(&mut self) {
        self.manager_tx.ez_send(SimulationCommand::ClearCache);
        self.frame_cache = std::collections::BTreeMap::new();
    }

    pub fn clear_local_cache(&mut self) {
        self.frame_cache = std::collections::BTreeMap::new();
    }

    pub fn process_requests(&mut self) {
        loop {
            if let Ok(res) = self.manager_rx.try_recv() {
                match res {
                    SimulationResponse::Frame(idx, frame) => {
                        let _ = self.frame_cache.insert(idx, frame);
                    },
                    SimulationResponse::Cached(count) => {
                        self.manager_cached = count;
                        self.frame_cache.split_off(&(count + 1));
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

    pub fn get_cached(&mut self) -> u32 {
        self.manager_cached
    }
}

impl SimulationManager {
    pub fn new(interface_tx: flume::Sender<SimulationResponse>, interface_rx: flume::Receiver<SimulationCommand>) -> Self {
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
            if let Ok(cmd) = self.interface_rx.try_recv() {
                match cmd {
                    SimulationCommand::RequestFrame(frame_idx) => {
                        let frame = self.get_frame(frame_idx).clone();

                        let res = SimulationResponse::Frame(frame_idx, frame);
                        
                        self.interface_tx.ez_send(res);
                    },
                    SimulationCommand::StoreFrame(frame_idx, state) => {
                        let frame = self.get_frame_mut(frame_idx);
                        *frame = state;
                    },
                    SimulationCommand::ClearCache => {
                        self.frame_cache = vec![]
                    },
                    SimulationCommand::GetCached => {
                        let res = SimulationResponse::Cached(self.frame_cache.len() as u32);

                        self.interface_tx.ez_send(res);
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

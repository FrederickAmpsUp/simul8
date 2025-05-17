pub struct CircleConstraint {
    radius: f32
}

impl CircleConstraint {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl super::Constraint for CircleConstraint {
    fn constrain(&self, particle: &mut super::Particle) {
        let dist = particle.position.length() + particle.radius;

        let dist_over = (dist - self.radius).max(0.0);

        particle.position -= particle.position * dist_over;
    }
}

//! Ambient particle effects for visual atmosphere.

use super::theme::ParticleStyle;

/// A single floating particle.
#[derive(Clone, Debug)]
pub struct Particle {
	pub x: f64,
	pub y: f64,
	pub vx: f64,
	pub vy: f64,
	pub size: f64,
	pub alpha: f64,
	pub phase: f64, // For twinkling
}

/// Manages ambient background particles.
pub struct ParticleSystem {
	pub particles: Vec<Particle>,
	width: f64,
	height: f64,
}

impl ParticleSystem {
	pub fn new(style: &ParticleStyle, width: f64, height: f64) -> Self {
		let mut particles = Vec::with_capacity(style.count);

		for i in 0..style.count {
			// Use deterministic pseudo-random based on index for consistent look
			let seed = i as f64;
			let px = Self::pseudo_random(seed * 1.1) * width;
			let py = Self::pseudo_random(seed * 2.3) * height;
			let angle = Self::pseudo_random(seed * 3.7) * std::f64::consts::TAU;
			let speed = style.speed * (0.5 + Self::pseudo_random(seed * 4.1) * 0.5);

			particles.push(Particle {
				x: px,
				y: py,
				vx: angle.cos() * speed,
				vy: angle.sin() * speed,
				size: style.size_min
					+ Self::pseudo_random(seed * 5.3) * (style.size_max - style.size_min),
				alpha: style.opacity * (0.3 + Self::pseudo_random(seed * 6.7) * 0.7),
				phase: Self::pseudo_random(seed * 7.9) * std::f64::consts::TAU,
			});
		}

		Self {
			particles,
			width,
			height,
		}
	}

	/// Simple pseudo-random function (deterministic)
	fn pseudo_random(seed: f64) -> f64 {
		let x = (seed * 12.9898 + seed * 78.233).sin() * 43758.5453;
		x - x.floor()
	}

	/// Update particle positions
	pub fn update(&mut self, dt: f64) {
		for p in &mut self.particles {
			p.x += p.vx * dt * 60.0;
			p.y += p.vy * dt * 60.0;
			p.phase += dt * 2.0;

			// Wrap around screen edges
			if p.x < -10.0 {
				p.x = self.width + 10.0;
			} else if p.x > self.width + 10.0 {
				p.x = -10.0;
			}
			if p.y < -10.0 {
				p.y = self.height + 10.0;
			} else if p.y > self.height + 10.0 {
				p.y = -10.0;
			}
		}
	}

	/// Resize the particle system bounds
	pub fn resize(&mut self, width: f64, height: f64) {
		// Scale particle positions proportionally
		let scale_x = width / self.width;
		let scale_y = height / self.height;

		for p in &mut self.particles {
			p.x *= scale_x;
			p.y *= scale_y;
		}

		self.width = width;
		self.height = height;
	}

	/// Get twinkle alpha for a particle
	pub fn twinkle_alpha(&self, particle: &Particle, time: f64) -> f64 {
		let twinkle = ((time * 1.5 + particle.phase).sin() * 0.5 + 0.5) * 0.4 + 0.6;
		particle.alpha * twinkle
	}
}

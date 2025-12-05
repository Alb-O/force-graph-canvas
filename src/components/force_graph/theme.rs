//! Visual theming for the force graph.
//!
//! Provides color palettes, gradients, and visual style configuration.

/// RGBA color representation.
#[derive(Clone, Copy, Debug)]
pub struct Color {
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub a: f64,
}

impl Color {
	pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
		Self { r, g, b, a: 1.0 }
	}

	pub const fn rgba(r: u8, g: u8, b: u8, a: f64) -> Self {
		Self { r, g, b, a }
	}

	pub fn with_alpha(self, a: f64) -> Self {
		Self { a, ..self }
	}

	/// Lighten the color by a factor (0.0 = unchanged, 1.0 = white)
	pub fn lighten(self, factor: f64) -> Self {
		let f = factor.clamp(0.0, 1.0);
		Self {
			r: (self.r as f64 + (255.0 - self.r as f64) * f) as u8,
			g: (self.g as f64 + (255.0 - self.g as f64) * f) as u8,
			b: (self.b as f64 + (255.0 - self.b as f64) * f) as u8,
			a: self.a,
		}
	}

	/// Darken the color by a factor (0.0 = unchanged, 1.0 = black)
	pub fn darken(self, factor: f64) -> Self {
		let f = 1.0 - factor.clamp(0.0, 1.0);
		Self {
			r: (self.r as f64 * f) as u8,
			g: (self.g as f64 * f) as u8,
			b: (self.b as f64 * f) as u8,
			a: self.a,
		}
	}

	/// Linear interpolation between two colors
	pub fn lerp(self, other: Color, t: f64) -> Self {
		let t = t.clamp(0.0, 1.0);
		Self {
			r: (self.r as f64 * (1.0 - t) + other.r as f64 * t) as u8,
			g: (self.g as f64 * (1.0 - t) + other.g as f64 * t) as u8,
			b: (self.b as f64 * (1.0 - t) + other.b as f64 * t) as u8,
			a: self.a * (1.0 - t) + other.a * t,
		}
	}

	pub fn to_css(self) -> String {
		if (self.a - 1.0).abs() < 0.001 {
			format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
		} else {
			format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
		}
	}

	pub fn to_css_rgb(self) -> String {
		format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
	}
}

/// A curated color palette for nodes.
#[derive(Clone, Debug)]
pub struct NodePalette {
	pub colors: Vec<Color>,
}

impl NodePalette {
	/// Muted, harmonious palette - slate blues and teals (default)
	pub fn slate() -> Self {
		Self {
			colors: vec![
				Color::rgb(94, 129, 172),  // Steel blue
				Color::rgb(129, 161, 193), // Light steel
				Color::rgb(100, 148, 160), // Teal gray
				Color::rgb(136, 160, 175), // Cadet blue
				Color::rgb(108, 142, 173), // Air force blue
				Color::rgb(119, 158, 165), // Desaturated cyan
				Color::rgb(143, 163, 180), // Cool gray
				Color::rgb(122, 153, 168), // Dusty blue
			],
		}
	}

	/// Warm earth tones - muted oranges and browns
	pub fn earth() -> Self {
		Self {
			colors: vec![
				Color::rgb(180, 136, 100), // Tan
				Color::rgb(160, 125, 100), // Taupe
				Color::rgb(170, 145, 115), // Khaki
				Color::rgb(145, 120, 95),  // Umber
				Color::rgb(175, 150, 120), // Sand
				Color::rgb(155, 130, 105), // Bronze
				Color::rgb(165, 140, 110), // Camel
				Color::rgb(150, 125, 100), // Mocha
			],
		}
	}

	/// Soft pastel palette - gentle, pleasing colors
	pub fn pastel() -> Self {
		Self {
			colors: vec![
				Color::rgb(200, 180, 190), // Dusty rose
				Color::rgb(180, 195, 205), // Powder blue
				Color::rgb(190, 200, 180), // Sage
				Color::rgb(205, 195, 180), // Cream
				Color::rgb(185, 190, 200), // Lavender gray
				Color::rgb(195, 185, 175), // Mushroom
				Color::rgb(180, 200, 195), // Seafoam
				Color::rgb(200, 190, 185), // Blush
			],
		}
	}

	/// Ocean depths palette - blues and teals
	pub fn ocean() -> Self {
		Self {
			colors: vec![
				Color::rgb(70, 110, 140),  // Deep blue
				Color::rgb(80, 130, 150),  // Cerulean
				Color::rgb(100, 145, 160), // Steel teal
				Color::rgb(90, 125, 145),  // Slate blue
				Color::rgb(85, 135, 155),  // Ocean
				Color::rgb(95, 120, 140),  // Denim
				Color::rgb(75, 115, 135),  // Navy gray
				Color::rgb(88, 128, 148),  // Cadet
			],
		}
	}

	/// Sunset palette - warm muted tones
	pub fn sunset() -> Self {
		Self {
			colors: vec![
				Color::rgb(180, 120, 100), // Terracotta
				Color::rgb(170, 130, 95),  // Sienna
				Color::rgb(185, 145, 110), // Amber
				Color::rgb(165, 115, 90),  // Rust
				Color::rgb(175, 125, 105), // Clay
				Color::rgb(160, 135, 100), // Ochre
				Color::rgb(170, 140, 115), // Copper
				Color::rgb(155, 120, 95),  // Chestnut
			],
		}
	}

	/// Aurora palette - cool teals and purples
	pub fn aurora() -> Self {
		Self {
			colors: vec![
				Color::rgb(100, 145, 135), // Eucalyptus
				Color::rgb(115, 135, 155), // Slate
				Color::rgb(130, 120, 150), // Wisteria
				Color::rgb(105, 140, 145), // Teal
				Color::rgb(120, 130, 160), // Periwinkle
				Color::rgb(125, 145, 140), // Sage
				Color::rgb(110, 125, 155), // Storm
				Color::rgb(135, 140, 150), // Pewter
			],
		}
	}

	pub fn get(&self, index: usize) -> Color {
		self.colors[index % self.colors.len()]
	}
}

/// Background style configuration.
#[derive(Clone, Debug)]
pub struct BackgroundStyle {
	/// Primary background color
	pub color: Color,
	/// Secondary color for gradients
	pub color_secondary: Color,
	/// Whether to use radial gradient
	pub use_gradient: bool,
	/// Vignette intensity (0.0 = none, 1.0 = strong)
	pub vignette: f64,
}

/// Edge visual style.
#[derive(Clone, Debug)]
pub struct EdgeStyle {
	/// Base edge color
	pub color: Color,
	/// Glow color (usually lighter version)
	pub glow_color: Color,
	/// Edge glow intensity
	pub glow_intensity: f64,
	/// Whether to use curved edges
	pub curved: bool,
	/// Curve tension (0.0 = straight, 1.0 = very curved)
	pub curve_tension: f64,
}

/// Node visual style.
#[derive(Clone, Debug)]
pub struct NodeStyle {
	/// Whether nodes have inner gradients
	pub use_gradient: bool,
	/// Outer glow intensity
	pub glow_intensity: f64,
	/// Glow color multiplier (how much node color affects glow)
	pub glow_saturation: f64,
	/// Border/stroke width (0 = no border)
	pub border_width: f64,
	/// Border color
	pub border_color: Color,
	/// Pulsing animation intensity (0.0 = none)
	pub pulse_intensity: f64,
	/// Pulsing animation speed
	pub pulse_speed: f64,
}

/// Particle effect configuration.
#[derive(Clone, Debug)]
pub struct ParticleStyle {
	/// Whether particles are enabled
	pub enabled: bool,
	/// Number of particles
	pub count: usize,
	/// Particle color
	pub color: Color,
	/// Minimum particle size
	pub size_min: f64,
	/// Maximum particle size
	pub size_max: f64,
	/// Particle movement speed
	pub speed: f64,
	/// Particle opacity
	pub opacity: f64,
}

/// Complete visual theme.
#[derive(Clone, Debug)]
pub struct Theme {
	pub name: &'static str,
	pub background: BackgroundStyle,
	pub edge: EdgeStyle,
	pub node: NodeStyle,
	pub particles: ParticleStyle,
	pub palette: NodePalette,
}

impl Theme {
	/// Clean modern theme with subtle effects (default)
	pub fn default_theme() -> Self {
		Self {
			name: "default",
			background: BackgroundStyle {
				color: Color::rgb(22, 27, 34),
				color_secondary: Color::rgb(30, 35, 42),
				use_gradient: true,
				vignette: 0.15,
			},
			edge: EdgeStyle {
				color: Color::rgba(140, 160, 180, 0.5),
				glow_color: Color::rgba(140, 160, 180, 0.1),
				glow_intensity: 0.0,
				curved: false,
				curve_tension: 0.0,
			},
			node: NodeStyle {
				use_gradient: true,
				glow_intensity: 0.0,
				glow_saturation: 0.0,
				border_width: 0.0,
				border_color: Color::rgba(255, 255, 255, 0.0),
				pulse_intensity: 0.0,
				pulse_speed: 0.0,
			},
			particles: ParticleStyle {
				enabled: false,
				count: 0,
				color: Color::rgba(0, 0, 0, 0.0),
				size_min: 0.0,
				size_max: 0.0,
				speed: 0.0,
				opacity: 0.0,
			},
			palette: NodePalette::slate(),
		}
	}

	/// Elegant dark theme with subtle effects
	pub fn midnight() -> Self {
		Self {
			name: "midnight",
			background: BackgroundStyle {
				color: Color::rgb(18, 20, 28),
				color_secondary: Color::rgb(25, 28, 38),
				use_gradient: true,
				vignette: 0.2,
			},
			edge: EdgeStyle {
				color: Color::rgba(100, 120, 150, 0.45),
				glow_color: Color::rgba(100, 120, 150, 0.1),
				glow_intensity: 0.0,
				curved: false,
				curve_tension: 0.0,
			},
			node: NodeStyle {
				use_gradient: true,
				glow_intensity: 0.0,
				glow_saturation: 0.0,
				border_width: 0.0,
				border_color: Color::rgba(255, 255, 255, 0.0),
				pulse_intensity: 0.0,
				pulse_speed: 0.0,
			},
			particles: ParticleStyle {
				enabled: false,
				count: 0,
				color: Color::rgba(0, 0, 0, 0.0),
				size_min: 0.0,
				size_max: 0.0,
				speed: 0.0,
				opacity: 0.0,
			},
			palette: NodePalette::aurora(),
		}
	}

	/// Warm earth tones theme
	pub fn ember() -> Self {
		Self {
			name: "ember",
			background: BackgroundStyle {
				color: Color::rgb(28, 24, 22),
				color_secondary: Color::rgb(35, 30, 28),
				use_gradient: true,
				vignette: 0.18,
			},
			edge: EdgeStyle {
				color: Color::rgba(160, 130, 110, 0.45),
				glow_color: Color::rgba(160, 130, 110, 0.1),
				glow_intensity: 0.0,
				curved: false,
				curve_tension: 0.0,
			},
			node: NodeStyle {
				use_gradient: true,
				glow_intensity: 0.0,
				glow_saturation: 0.0,
				border_width: 0.0,
				border_color: Color::rgba(255, 255, 255, 0.0),
				pulse_intensity: 0.0,
				pulse_speed: 0.0,
			},
			particles: ParticleStyle {
				enabled: false,
				count: 0,
				color: Color::rgba(0, 0, 0, 0.0),
				size_min: 0.0,
				size_max: 0.0,
				speed: 0.0,
				opacity: 0.0,
			},
			palette: NodePalette::earth(),
		}
	}

	/// Ocean/deep blue theme
	pub fn deep_sea() -> Self {
		Self {
			name: "deep_sea",
			background: BackgroundStyle {
				color: Color::rgb(15, 25, 35),
				color_secondary: Color::rgb(20, 32, 45),
				use_gradient: true,
				vignette: 0.2,
			},
			edge: EdgeStyle {
				color: Color::rgba(90, 130, 160, 0.45),
				glow_color: Color::rgba(90, 130, 160, 0.1),
				glow_intensity: 0.0,
				curved: false,
				curve_tension: 0.0,
			},
			node: NodeStyle {
				use_gradient: true,
				glow_intensity: 0.0,
				glow_saturation: 0.0,
				border_width: 0.0,
				border_color: Color::rgba(255, 255, 255, 0.0),
				pulse_intensity: 0.0,
				pulse_speed: 0.0,
			},
			particles: ParticleStyle {
				enabled: false,
				count: 0,
				color: Color::rgba(0, 0, 0, 0.0),
				size_min: 0.0,
				size_max: 0.0,
				speed: 0.0,
				opacity: 0.0,
			},
			palette: NodePalette::ocean(),
		}
	}

	/// Minimal, ultra-clean theme
	pub fn minimal() -> Self {
		Self {
			name: "minimal",
			background: BackgroundStyle {
				color: Color::rgb(25, 28, 35),
				color_secondary: Color::rgb(25, 28, 35),
				use_gradient: false,
				vignette: 0.0,
			},
			edge: EdgeStyle {
				color: Color::rgba(130, 145, 165, 0.4),
				glow_color: Color::rgba(130, 145, 165, 0.0),
				glow_intensity: 0.0,
				curved: false,
				curve_tension: 0.0,
			},
			node: NodeStyle {
				use_gradient: false,
				glow_intensity: 0.0,
				glow_saturation: 0.0,
				border_width: 0.0,
				border_color: Color::rgba(255, 255, 255, 0.0),
				pulse_intensity: 0.0,
				pulse_speed: 0.0,
			},
			particles: ParticleStyle {
				enabled: false,
				count: 0,
				color: Color::rgba(0, 0, 0, 0.0),
				size_min: 0.0,
				size_max: 0.0,
				speed: 0.0,
				opacity: 0.0,
			},
			palette: NodePalette::pastel(),
		}
	}
}

impl Default for Theme {
	fn default() -> Self {
		Self::default_theme()
	}
}

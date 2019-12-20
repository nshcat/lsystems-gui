use nalgebra_glm::Vec3;

/// A struct containing all information used to cast light into the scene
#[derive(Clone)]
pub struct LightingContext {
    /// Ambient light intensity
    pub ambient_intensity: Vec3,
    /// Directional light angle
    pub directional_light: Vec3,
    /// Directional light intensity
    pub directional_intensity: Vec3
}

impl LightingContext {
    /// Constructs a lighting context with default values
    pub fn new_default() -> LightingContext {
        LightingContext {
            ambient_intensity: Vec3::new(0.4, 0.4, 0.4),
            directional_light: Vec3::new(0.0, 1.0, 1.0),
            directional_intensity: Vec3::new(0.8, 0.8, 0.8)
        }
    }
}
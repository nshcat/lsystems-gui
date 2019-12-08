use serde::*;
#[macro_use]
use serde_derive::*;
use std::string::*;
use std::collections::*;
use nalgebra_glm::Vec3;
use lsystems_core::drawing::{DrawingParameters, DrawOperation};

/// A struct containing all the information that describes a single LSystem.
#[derive(Serialize, Deserialize)]
pub struct LSystemParameters {
	pub drawing_parameters: DrawingParameters,
	/// Whether the camera position will be modified when this LSystem gets loaded
	pub modify_camera: bool,
	pub camera_radius: f64,
	pub camera_phi: f64,
	pub camera_theta: f64,
	pub axiom: String,
	pub seed: u64,
	pub iteration_depth: u32,
	pub rules: Vec<String>,
	pub interpretations: HashMap<char, DrawOperation>,
	pub color_palette: Vec<Vec3>
}





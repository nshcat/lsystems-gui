use serde::*;
#[macro_use]
use serde_derive::*;
use serde_json::*;
use std::string::*;
use std::collections::*;
use nalgebra_glm::Vec3;
use lsystems_core::*;
use lsystems_core::drawing::{DrawingParameters, DrawOperation};
use crate::data::patches::*;


pub mod presets;
pub mod patches;



/// A special structure used to represent a single interpretation mapping.
/// This is only used with the GUI, and the Option allows the user to have interpretations
/// with an empty symbol field, which improves UX.
#[derive(Serialize, Deserialize, Clone)]
pub struct Interpretation {
	pub symbol: Option<char>,
	pub operation: DrawOperation
}

/// Struct containing application-wide settings
#[derive(Serialize, Deserialize, Clone)]
pub struct ApplicationSettings {
	/// Whether the displayed LSystem should be refreshed on parameter change.
	pub auto_refresh: bool,
	/// Whether to draw the bounding box around the lsystem
	pub draw_bounding_box: bool,
	/// Whether to automatically center camera on lsystem center on reload
	pub auto_center_camera: bool,
	/// Whether to additionally adjust the camera radius in order to have the full system in view.
	/// Is only relevant if auto centering is active.
	pub auto_adjust_radius: bool,
	/// The color of the bounding box wireframe
	pub bounding_box_color: Vec3,
	/// Whether to show the normal vectors of polygons (debug feature)
	pub show_normals: bool,
	/// Whether to draw polygons as wireframes (debug features)
	pub draw_wireframe: bool
}

impl ApplicationSettings {
	/// The default settings
	pub fn default_settings() -> ApplicationSettings {
		ApplicationSettings {
			auto_refresh: true,
			draw_bounding_box: false,
			auto_center_camera: true,
			auto_adjust_radius: true,
			bounding_box_color: Vec3::new(1.0, 1.0, 1.0),
			show_normals: false,
			draw_wireframe: false
		}
	}

	/// Read a new instance from JSON string.
	pub fn from_string(input: &str) -> ApplicationSettings {
		serde_json::from_str(input).expect("Failed to read ApplicationSettings from JSON")
	}
}

/// A struct containing all the information that describes a single LSystem.
#[derive(Serialize, Deserialize, Clone)]
pub struct LSystemParameters {
    pub name: String,
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
	/// The usage of a Vec instead of a associative container is done in order to preserve
	/// order of interpretations and thus obtain some degree of consistency when it comes to
	/// gui rendering.
	pub interpretations: Vec<Interpretation>,
	pub color_palette: Vec<Vec3>,
	pub bezier_models: Vec<BezierModelParameters>
}

impl LSystemParameters {
	/// Read a new instance from JSON string.
	pub fn from_string(input: &str) -> LSystemParameters {
		serde_json::from_str(input).expect("Failed to read LSystemParameters from JSON")
	}
}






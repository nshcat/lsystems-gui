use nalgebra_glm::*;
use crate::rendering::*;
use crate::rendering::meshes::*;
use crate::rendering::traits::*;
use std::rc::*;

/// Mesh manager that can either take full ownership of a meshes or just store a reference
/// counted pointer to one
enum MeshStorage {
	/// Storage manages ref counted pointers to meshes.
	/// This allows meshes to be shared between multiple models.
	RefCounted{ meshes: Vec<Rc<Box<Mesh>>> },
	/// The storage is the sole owner of the meshes.
	Owned{ meshes: Vec<Mesh> }
}

impl Render for MeshStorage {
	fn render(&self, rp: &mut RenderParameters) {
		match self {
			MeshStorage::RefCounted{ meshes } => {
				for mesh in meshes {
					mesh.render(rp);
				}
			},
			MeshStorage::Owned{ meshes } => {
				for mesh in meshes {
					mesh.render(rp);
				}
			}
		}
	}
}


/// A model is a set of meshes combined with a model transformation matrix.
/// The meshes can either be owned or referenced via Rc pointers, which allows sharing
/// of meshes between multiple models.
pub struct Model {
    /// Storage instance managing the meshes associated with this model
    pub storage: MeshStorage,
    /// The model transformation matrix. Converts local model space into world space.
    pub transform: Mat4
}

impl Model {
    pub fn with_position(mesh: Rc<Box<Mesh>>, position: &Vec3) -> Model {
        Model {
            mesh: mesh,
            transform: Mat4::new_translation(position)
        }
    }
}

impl Render for Model {
    fn render(&self, params: &mut RenderParameters) {
        params.push_matrix();
        params.add_matrix(&self.transform);

        self.storage.render(params);

        params.pop_matrix();
    }
}

/// A multi model is a collection of multiple models that form one logical unit. By using
/// the transformation matrix of the multi model, hierachical rendering may be implemented.
/// The difference to a Model containing multiple meshes is that the various models
/// can, in this case, have different model transformation matrices.
pub struct MultiModel {
	/// Collection of sub models.
    pub models: Vec<Model>,
    /// The model matrix that will be applied to all sub models, in addition to their
    /// specific model matrices.
    pub transforma: Mat4
}

impl MultiModel {
	pub fn new( models.. mutable slice
}

impl Render for MultiModel {
    fn render(&self, params: &mut RenderParameters) {
    	params.push_matrix();
    	params.add_matrix(&self.transform);
    	
        for model in &self.models {
            model.render(params);
        }
        
        params.pop_matrix();
    }
}

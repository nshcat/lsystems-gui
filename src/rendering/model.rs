use nalgebra_glm::*;
use crate::rendering::*;
use crate::rendering::meshes::*;
use crate::rendering::traits::*;
use std::rc::*;

/// Mesh manager that can either take full ownership of a meshes or just store a reference
/// counted pointer to one
enum MeshStorage {
	/// The storage manages ref counted pointers to meshes.
	/// This allows meshes to be shared between multiple models.
	RefCounted{ meshes: Vec<Rc<Mesh>> },
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
    storage: MeshStorage,
    /// The model transformation matrix. Converts local model space into world space.
    pub transform: Mat4
}

impl Model {
    /// Create model from given mesh. The model will take ownership of the mesh.
    pub fn from_mesh(mesh: Mesh) -> Model {
        Model {
            storage: MeshStorage::Owned{ meshes: vec![mesh] },
            transform: Mat4::identity()
        }
    }

    /// Create model from collection of models, taking ownership in the process.
    pub fn from_meshes(meshes: impl IntoIterator<Item = Mesh>) -> Model {
        Model {
            storage: MeshStorage::Owned{ meshes: meshes.into_iter().collect() },
            transform: Mat4::identity()
        }
    }

    /// Create model from given mesh and model transformation. The model will take ownership of the mesh.
    pub fn from_mesh_transformed(mesh: Mesh, trans: Mat4) -> Model {
        Model {
            storage: MeshStorage::Owned{ meshes: vec![mesh] },
            transform: trans
        }
    }

    /// Create model from collection of models, taking ownership in the process.
    pub fn from_meshes_transformed(meshes: impl IntoIterator<Item = Mesh>, trans: Mat4) -> Model {
        Model {
            storage: MeshStorage::Owned{ meshes: meshes.into_iter().collect() },
            transform: trans
        }
    }

    /// Create model from reference counted mesh reference.
    pub fn from_mesh_rc(mesh: Rc<Mesh>) -> Model {
        Model {
            storage: MeshStorage::RefCounted{ meshes: vec![mesh] },
            transform: Mat4::identity()
        }
    }

    /// Create model from collection of reference counted mesh references.
    pub fn from_meshes_rc(meshes: &[Rc<Mesh>]) -> Model {
        Model {
            storage: MeshStorage::RefCounted{ meshes: meshes.to_vec() },
            transform: Mat4::identity()
        }
    }

    /// Create model from reference counted mesh reference and model transformation matrix.
    pub fn from_mesh_transformed_rc(mesh: Rc<Mesh>, trans: Mat4) -> Model {
        Model {
            storage: MeshStorage::RefCounted{ meshes: vec![mesh] },
            transform: trans
        }
    }

    /// Create model from collection of reference counted mesh references.
    pub fn from_meshes_transformed_rc(meshes: &[Rc<Mesh>], trans: Mat4) -> Model {
        Model {
            storage: MeshStorage::RefCounted{ meshes: meshes.to_vec() },
            transform: trans
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
    pub transform: Mat4
}

impl MultiModel {
    /// Create multi model from given model collection, taking ownership of them in the process.
    pub fn from_models(models: impl IntoIterator<Item = Model>) -> MultiModel {
        MultiModel {
            models: models.into_iter().collect(),
            transform: Mat4::identity()
        }
    }

    /// Create multi model from given model collection and model transformation matrix, taking
    /// ownership of them in the process.
    pub fn from_models_transformed(models: impl IntoIterator<Item = Model>, trans: Mat4) -> MultiModel {
        MultiModel {
            models: models.into_iter().collect(),
            transform: trans
        }
    }
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

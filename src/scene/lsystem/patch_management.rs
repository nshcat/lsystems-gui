use std::rc::*;
use std::collections::HashMap;
use crate::rendering::bezier::*;
use crate::rendering::meshes::*;
use crate::rendering::materials::*;
use crate::data::bezier::*;



/// Type that manages the meshes for the various bezier models. This allows
/// different patch instantiations to share the same mesh instance.
/// This type maps bezier patch identifiers to the mesh collections and
/// allows updating of those meshes as reaction to patch updates. 
pub struct BezierMeshManager {
    /// Mapping between patch model identifier and associated patch meshes
    mesh_map: HashMap<char, Vec<Rc<Mesh>>>
}

impl BezierMeshManager {
    /// Create a new bezier mesh manager instance.
    pub fn new() -> BezierMeshManager {
        BezierMeshManager {
            mesh_map: HashMap::new()
        }
    }

    /// Update stored patch meshes for bezier model with given identifier. Will create a new entry
    /// if it does not already exist.
    pub fn update_patch(&mut self, parameters: &BezierModelParameters) {
        // Ignore models that dont have any parameters set
        if let Some(identifier) = parameters.symbol {
            // If there is already an entry for this identifier, remove that entry. Its obsolete.
            if self.has_patch(identifier) {
                self.mesh_map.remove(&identifier);     
            }

            self.mesh_map.insert(identifier, Self::create_meshes(parameters));
        }   
    }

    /// Check if a bezier model with given identifier is known.
    pub fn has_patch(&self, identifier: char) -> bool {
        self.mesh_map.contains_key(&identifier)
    }

    /// Create the patch meshes for bezier model described by given parameters.
    fn create_meshes(parameters: &BezierModelParameters) -> Vec<Rc<Mesh>> {
        let mut meshes = Vec::new();

        // Create mesh for each patch
        for patch in &parameters.patches {
            let geometry = BezierGeometry::new(patch, 30, 30);
            let material = Box::new(ShadedMaterial::new());

            meshes.push(Rc::new(Mesh::new_indexed(
                PrimitiveType::TriangleStrip,
                material,
                &geometry
            )));
        }

        meshes
    }

    /// Retrieve patch meshes for bezier model with given identifier.
    pub fn retrieve_meshes(&self, identifier: char) -> Vec<Rc<Mesh>> {
        self.mesh_map.get(&identifier).unwrap().clone()
    }
}


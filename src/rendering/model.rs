use nalgebra_glm::*;
use crate::rendering::*;
use crate::rendering::meshes::*;
use crate::rendering::traits::*;
use std::rc::*;

/// A model is a mesh combined with a model space transformation.
#[derive(Clone)]
pub struct Model {
    /// The mesh associated with this model
    pub mesh: Rc<Box<Mesh>>,
    /// The model transformation
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

        self.mesh.render(params);

        params.pop_matrix();
    }
}


pub struct MultiModel {
    pub models: Vec<Model>
}

impl Render for MultiModel {
    fn render(&self, params: &mut RenderParameters) {
        for model in &self.models {
            model.render(params);
        }
    }
}

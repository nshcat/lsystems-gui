use nalgebra_glm::Vec3;

use crate::rendering::*;
use crate::rendering::meshes::*;
use crate::rendering::materials::*;
use crate::rendering::traits::*;
use crate::rendering::shaders::*;

/// A gizmo visualizing the coordinate system origin and the three cardinal axises
pub struct OriginGizmo {
    /// The mesh containing the line data for this gizmo
    mesh: Mesh
}

impl OriginGizmo {
    /// Create new origin gizmo with the given axis length and thickness
    pub fn new(axis_length: f32, axis_thickness: f32) -> OriginGizmo {
        let mut vertices = Vec::new();

        // x Axis is red
        vertices.push(Vertex::new(Vec3::zeros(), Vec3::new(1.0, 0.0, 0.0)));
        vertices.push(Vertex::new(Vec3::x() * axis_length, Vec3::new(1.0, 0.0, 0.0)));

        // y Axis is green
        vertices.push(Vertex::new(Vec3::zeros(), Vec3::new(0.0, 1.0, 0.0)));
        vertices.push(Vertex::new(Vec3::y() * axis_length, Vec3::new(0.0, 1.0, 0.0)));

        // z Axis is blue
        vertices.push(Vertex::new(Vec3::zeros(), Vec3::new(0.0, 0.0, 1.0)));
        vertices.push(Vertex::new(Vec3::z() * axis_length, Vec3::new(0.0, 0.0, 1.0)));

        let geometry = BasicGeometry::from_vertices(&vertices);

        let material = Box::new(SimpleMaterial::new());

        let mut mesh = Mesh::new(PrimitiveType::Lines, material, &geometry);
        mesh.line_width = axis_thickness;

        OriginGizmo {
            mesh: mesh
        }
    }
}

impl Render for OriginGizmo {
    fn render(&self, rp: &mut RenderParameters) {
        self.mesh.render(rp);
    }
}
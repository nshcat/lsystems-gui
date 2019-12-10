use ncollide3d::bounding_volume::*;
use nalgebra_glm::*;
use nalgebra::*;
use std::any::*;

use crate::rendering::shaders::*;
use crate::rendering::materials::*;
use crate::rendering::uniforms::*;
use crate::rendering::*;
use crate::rendering::meshes::*;
use crate::rendering::traits::*;

/// Geometry for the bounding box mesh
struct BoundingBoxGeometry {
    /// Position information
    position: AttributeArray<Vec3>
}

impl Geometry for BoundingBoxGeometry {
    fn retrieve_attributes(&self) -> Vec<&dyn AttributeArrayBase> {
        vec![&self.position]
    }
}

impl BoundingBoxGeometry {
    /// Create new bounding box geometry from bounding box
    pub fn new(aabb: &AABB<f32>) -> BoundingBoxGeometry {
        // Retrieve smallest and biggested point
        let min = aabb.mins().coords;
        let max = aabb.maxs().coords;

        let mut geom = BoundingBoxGeometry {
            position: AttributeArray::new(0, "position")
        };

        let buffer = &mut geom.position.local_buffer;

        // Front face
        // y           tl --- tr
        // ^            |      |
        // + > x       bl --- br

        let bl = min.clone();
        let br = Vec3::new(max.x, min.y, min.z);
        let tl = Vec3::new(min.x, max.y, min.z);
        let tr = Vec3::new(max.x, max.y, min.z);

        buffer.push(bl); buffer.push(br);
        buffer.push(br); buffer.push(tr);
        buffer.push(tr); buffer.push(tl);
        buffer.push(tl); buffer.push(bl);

        // Back face
        // y           btl -- btr
        // ^            |      |
        // + > x       bbl -- bbr
        let btr = max.clone();
        let btl = Vec3::new(min.x, max.y, max.z);
        let bbl = Vec3::new(min.x, min.y, max.z);
        let bbr = Vec3::new(max.x, min.y, max.z);
        
        buffer.push(bbl); buffer.push(bbr);
        buffer.push(bbr); buffer.push(btr);
        buffer.push(btr); buffer.push(btl);
        buffer.push(btl); buffer.push(bbl);

        // Connections
        buffer.push(tl); buffer.push(btl);
        buffer.push(bl); buffer.push(bbl);
        buffer.push(tr); buffer.push(btr);
        buffer.push(br); buffer.push(bbr);

        geom
    }
}

pub struct BoundingBox {
    /// The mesh containing the bounding box lines
    mesh: Mesh,
    /// The AABB instance used to crate the outline and calculate the center
    pub aabb: AABB<f32>
}

impl BoundingBox {
    pub fn new(vertices: &[Vec3]) -> BoundingBox {
        // The AABB sadly only accepts points, so we have to convert them.
        let points: Vec<Point3<f32>> = vertices
            .iter()
            .map(|v| Point3::<f32>::new(v.x, v.y, v.z))
            .collect();

        // Create aabb instance
        let bx: AABB<f32> = point_cloud_aabb(
            &Isometry3::<f32>::new(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 0.0)
            ),
            &points
        );

        // Create mesh
        let mesh = Mesh::new(
            PrimitiveType::Lines,
            Box::new(BoundingBoxMaterial::new(&Vec3::new(1.0, 1.0, 1.0))),
            &BoundingBoxGeometry::new(&bx)
        );

        BoundingBox {
            aabb: bx,
            mesh: mesh
        }
    }

    pub fn set_color(&mut self, clr: &Vec3) {
        self.mesh.retrieve_material_mut_ref::<BoundingBoxMaterial>()
            .color = clr.clone();
    }

    /// Retrieve a radius of a sphere completely containing this AABB
    pub fn radius(&self) -> f64 {
        let sphere = self.aabb.bounding_sphere();

        sphere.radius() as _
    }
}

impl Render for BoundingBox {
    fn render(&self, params: &mut RenderParameters) {
        self.mesh.render(params);
    }
}


/// Material used by the bounding box scene element
struct BoundingBoxMaterial {
    /// The shader program used by this material
    shader: Program,
    /// The color of the bounding box. This is set as a uniform
    pub color: Vec3
}

impl BoundingBoxMaterial {
    pub fn new(color: &Vec3) -> BoundingBoxMaterial {
        BoundingBoxMaterial {
            color: color.clone(),
            shader: Program::from_source(Self::VERTEX_SHADER_SOURCE, Self::FRAGMENT_SHADER_SOURCE).unwrap()
        }
    }
}

impl Material for BoundingBoxMaterial {
    fn enable_material(&self, params: &mut RenderParameters) {
        self.shader.use_program();

        self.shader.set_uniform_mat4("projection", &params.projection);
        self.shader.set_uniform_mat4("view", &params.view);
        self.shader.set_uniform_mat4("model", &params.model);
        self.shader.set_uniform_vec3("color", &self.color);
    }

    /// Retrieve this instance as a reference to Any. This is used for downcasting.
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// Retrieve this instance as a mutable reference to Any. This is used for downcasting.
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

/// Shader source code
impl BoundingBoxMaterial  {
    /// The vertex shader source for this material
    pub const VERTEX_SHADER_SOURCE: &'static str = r#"
        #version 330 core

        layout (location = 0) in vec3 Position;

        uniform mat4 projection;
        uniform mat4 view;
        uniform mat4 model;
        uniform vec3 color;

        out VS_OUTPUT {
            vec3 Color;
        } OUT;

        void main()
        {
            gl_Position = projection * view * model * vec4(Position, 1.0);
            OUT.Color = color;
        }
    "#;

    /// The fragment shader source for this material
    const FRAGMENT_SHADER_SOURCE: &'static str = r#"
        #version 330 core

        in VS_OUTPUT {
            vec3 Color;
        } IN;

        out vec4 Color;

        void main()
        {
            Color = vec4(IN.Color, 1.0f);
        }
    "#;
}

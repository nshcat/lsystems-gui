use nalgebra_glm::{Vec3, Mat4};
use std::any::*;
use std::ops::DerefMut;
use crate::rendering::*;
use crate::rendering::materials::*;
use crate::rendering::shaders::*;
use crate::rendering::uniforms::*;

/// A material that displays normal vectors as colors
pub struct NormalColorTestMaterial {
    /// The shader program used by this material
    program: Program,
}

impl NormalColorTestMaterial {
    pub fn new() -> NormalColorTestMaterial {
        let mut shaders = vec![
            Shader::new_vertex(Self::VERTEX_SHADER_SOURCE).unwrap(),
            Shader::new_fragment(Self::FRAGMENT_SHADER_SOURCE).unwrap()
        ];

        NormalColorTestMaterial {
            program: Program::from_shaders(
                &mut shaders
            ).unwrap()
        }
    }
}

impl Material for NormalColorTestMaterial {
    fn enable_material(&self, params: &mut RenderParameters) {
        self.program.use_program();

        self.program.set_uniform_mat4("projection", &params.projection);
        self.program.set_uniform_mat4("view", &params.view);
        self.program.set_uniform_mat4("model", &params.model);
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

/// The shader source code
impl NormalColorTestMaterial {
    /// The vertex shader source for this material
    const VERTEX_SHADER_SOURCE: &'static str = r#"
        #version 330 core

        layout (location = 0) in vec3 Position;
        layout (location = 1) in vec3 Color;
        layout (location = 2) in vec3 Normal; 

        uniform mat4 projection;
        uniform mat4 view;
        uniform mat4 model;

        out Vertex
        {
            vec4 normal;
            vec4 color;
        } vertex;

        void main()
        {
            gl_Position = projection * view * model * vec4(Position, 1.0);
            vertex.color = vec4(Color, 1.0);
            vertex.normal = vec4(Normal, 1.0);
        }
    "#;

    /// The fragment shader source for this material
    const FRAGMENT_SHADER_SOURCE: &'static str = r#"
        #version 330 core

        in Vertex 
        {
            vec4 normal;
            vec4 color;
        } vertex;

        out vec4 Out_Color;

        void main()
        {
            Out_Color = vertex.normal;
        }
    "#;
}

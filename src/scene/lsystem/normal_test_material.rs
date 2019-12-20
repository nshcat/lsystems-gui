use nalgebra_glm::{Vec3, Mat4};
use std::any::*;
use std::ops::DerefMut;
use crate::rendering::*;
use crate::rendering::materials::*;
use crate::rendering::shaders::*;
use crate::rendering::uniforms::*;

/// A material that uses a geometry shader in order to display the normal vectors
/// of a mesh.
pub struct NormalTestMaterial {
    /// The shader program used by this material
    program: Program,
    /// Length of the rendered normal vectors
    pub normal_length: f32,
    /// Color of the normal vectors
    pub normal_color: Vec3
}

impl NormalTestMaterial {
    pub fn new(length: f32, color: &Vec3) -> NormalTestMaterial {
        let mut shaders = vec![
            Shader::new_vertex(Self::VERTEX_SHADER_SOURCE).unwrap(),
            Shader::new_fragment(Self::FRAGMENT_SHADER_SOURCE).unwrap(),
            Shader::new_geometry(Self::GEOMETRY_SHADER_SOURCE).unwrap()
        ];

        NormalTestMaterial {
            program: Program::from_shaders(
                &mut shaders
            ).unwrap(),
            normal_length: length,
            normal_color: color.clone()
        }
    }
}

impl Material for NormalTestMaterial {
    fn enable_material(&self, params: &mut RenderParameters) {
        self.program.use_program();

        self.program.set_uniform_mat4("projection", &params.projection);
        self.program.set_uniform_mat4("view", &params.view);
        self.program.set_uniform_mat4("model", &params.model);

        self.program.set_uniform_vec3("normal_color", &self.normal_color);
        self.program.set_uniform_float("normal_length", self.normal_length);
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
impl NormalTestMaterial {
    /// The vertex shader source for this material
    const VERTEX_SHADER_SOURCE: &'static str = r#"
        #version 330 core

        layout (location = 0) in vec3 Position;
        layout (location = 1) in vec3 Color;
        layout (location = 2) in vec3 Normal; 

        out Vertex
        {
            vec4 normal;
            vec4 color;
        } vertex;

        void main()
        {
            gl_Position = vec4(Position, 1.0);
            vertex.color = vec4(Color, 1.0);
            vertex.normal = vec4(Normal, 1.0);
        }
    "#;

    /// The geometry shader source for this material
    const GEOMETRY_SHADER_SOURCE: &'static str = r#"
        #version 330 core

        layout(triangles) in;
        layout(line_strip, max_vertices=6) out;

        uniform mat4 projection;
        uniform mat4 view;
        uniform mat4 model;
        uniform float normal_length;
        uniform vec3 normal_color;

        in Vertex 
        {
            vec4 normal;
            vec4 color;
        } vertex[];

        out vec4 vertex_color;

        void main()
        {
            mat4 mvp = projection * view * model;

            int i;
            for(i = 0; i < gl_in.length(); ++i) 
            {
                vec3 P = gl_in[i].gl_Position.xyz;
                vec3 N = vertex[i].normal.xyz;

                gl_Position = mvp * vec4(P, 1.0);
                vertex_color = vec4(normal_color, 1.0);
                EmitVertex();

                gl_Position = mvp * vec4(P + N * normal_length, 1.0);
                vertex_color = vec4(normal_color, 1.0);
                EmitVertex();
                EndPrimitive();
            }
        }
    "#;

    /// The fragment shader source for this material
    const FRAGMENT_SHADER_SOURCE: &'static str = r#"
        #version 330 core

        in vec4 vertex_color;
        out vec4 Out_Color;

        void main()
        {
            Out_Color = vertex_color;
        }
    "#;
}

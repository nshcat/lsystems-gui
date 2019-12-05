use gl::types::*;
use crate::rendering::RenderParameters;
use crate::rendering::shaders::Program;
use crate::rendering::uniforms::*;
use nalgebra_glm::{Mat4, Vec3};

/// A trait describing a material. A material is an entity which cam have various shader uniforms, which
/// need to be uploaded to the shader program contained within.
/// This trait provides no way to access the internal shader program object, since access to it is of no
/// particular interest, since the material is expected to handle all shader specific operations on its own.
pub trait Material {
    /// Enable this material and prepare it for rendering.
    /// This function will cause all matrices to be extracted from the provided render parameters, as well
    /// as all shader-specific uniforms 
    fn enable_material(&self, params: &mut RenderParameters);
}


/// A simple material that applies no shading.
struct SimpleMaterial {
    /// The shader program associated with this material
    program: Program
}

/// Construction
impl SimpleMaterial {
    /// Create a new simple material instance
    pub fn new() -> SimpleMaterial {
        SimpleMaterial {
            program: Program::from_source(Self::VERTEX_SHADER_SOURCE, Self::FRAGMENT_SHADER_SOURCE).unwrap()
        }
    }
}

impl Material for SimpleMaterial {
    fn enable_material(&self, params: &mut RenderParameters) {
        self.program.use_program();

        self.program.set_uniform_mat4("projection", &params.projection);
        self.program.set_uniform_mat4("view", &params.view);
        self.program.set_uniform_mat4("model", &params.model);
    }
}

/// Shader source code
impl SimpleMaterial {
    /// The vertex shader source for this material
    pub const VERTEX_SHADER_SOURCE: &'static str = r#"
        #version 330 core

        layout (location = 0) in vec3 Position;
        layout (location = 1) in vec3 Color;
        layout (location = 2) in vec3 Normal;

        uniform mat4 projection;
        uniform mat4 view;
        uniform mat4 model;

        out VS_OUTPUT {
            vec3 Color;
        } OUT;

        void main()
        {
            gl_Position = projection * view * model * vec4(Position, 1.0);
            OUT.Color = Color;
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
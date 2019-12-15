use gl::types::*;
use std::any::*;
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
    /// Retrieve this instance as a reference to Any. This is used for downcasting.
    fn as_any(&self) -> &dyn Any;
    /// Retrieve this instance as a mutable reference to Any. This is used for downcasting.
    fn as_mut_any(&mut self) -> &mut dyn Any;
}


/// A simple material that applies no shading.
pub struct SimpleMaterial {
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
impl SimpleMaterial {
    /// The vertex shader source for this material
    const VERTEX_SHADER_SOURCE: &'static str = r#"
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


/// A simple shaded material that uses the lighting information stored in the lighting context to
/// apply diffuse and specular lighting to the object.
pub struct ShadedMaterial {
    /// The shader program associated with this material
    program: Program,/*
    /// How the surface reacts to diffuse lighting. This basically is the base color.
    pub diffuse_reflectivity: Vec3,
    /// How the surface reacts to ambient lighting.
    pub ambient_reflectivity: Vec3,
    /// How reflective the surface is to specular highlights
    pub specular_reflectivity: Vec3,
    /// How shiny the surface is
    pub specular_shininess: f32*/
}

/// Construction
impl ShadedMaterial {
    /// Create a new simple material instance
    pub fn new(/*diffuse: Vec3, ambient: Vec3, specular: Vec3, shininess: f32*/) -> ShadedMaterial {
        ShadedMaterial {
            program: Program::from_source(Self::VERTEX_SHADER_SOURCE, Self::FRAGMENT_SHADER_SOURCE).unwrap()//,
            /*diffuse_reflectivity: diffuse,
            ambient_reflectivity: ambient,
            specular_reflectivity: specular,
            specular_shininess: shininess*/
        }
    }
}

impl Material for ShadedMaterial {
    fn enable_material(&self, params: &mut RenderParameters) {
        self.program.use_program();

        self.program.set_uniform_mat4("projection", &params.projection);
        self.program.set_uniform_mat4("view", &params.view);
        self.program.set_uniform_mat4("model", &params.model);
        //self.program.set_uniform_vec3("Kd", &self.diffuse_reflectivity);
        //self.program.set_uniform_vec3("Ka", &self.ambient_reflectivity);
        //self.program.set_uniform_vec3("Ks", &self.specular_reflectivity);
        //self.program.set_uniform_float("Shininess", self.specular_shininess);

        self.program.set_uniform_vec3("AmbientIntensity", &params.lighting.ambient_intensity);
        self.program.set_uniform_vec3("DirectionalIntensity", &params.lighting.directional_intensity);
        self.program.set_uniform_vec3("DirectionalLight", &params.lighting.directional_light);
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
impl ShadedMaterial {
    /// The vertex shader source for this material
    const VERTEX_SHADER_SOURCE: &'static str = r#"
        #version 330 core

        layout (location = 0) in vec3 Position;
        layout (location = 1) in vec3 Color;
        layout (location = 2) in vec3 Normal;

        uniform mat4 projection;
        uniform mat4 view;
        uniform mat4 model;

        out VS_OUTPUT {
            vec3 FragPos;
            vec3 Normal;
            vec3 Color;
        } OUT;

        void main()
        {
            gl_Position = projection * view * model * vec4(Position, 1.0);
            OUT.FragPos = vec3(model * vec4(Position, 1.0));

            vec3 posView = (view * model * vec4(Position, 1.0)).xyz;
            vec3 viewNormal = (mat3(transpose(inverse(view * model))) * Normal);

            if (dot(viewNormal, posView) < 0.0)
            {
                OUT.Normal = mat3(transpose(inverse(model))) * Normal;
            }
            else 
            {
                OUT.Normal = mat3(transpose(inverse(model))) * -Normal;
            }
            
            OUT.Color = Color;
        }
    "#;

    /// The fragment shader source for this material
    const FRAGMENT_SHADER_SOURCE: &'static str = r#"
        #version 330 core

        uniform vec3 AmbientIntensity;
        uniform vec3 DirectionalIntensity;
        uniform vec3 DirectionalLight;

        in VS_OUTPUT {
            vec3 FragPos;
            vec3 Normal;
            vec3 Color;
        } IN;

        out vec4 Color;

        void main()
        {
            vec3 ambient = AmbientIntensity;
            
            float diff = max(dot(normalize(IN.Normal), normalize(DirectionalLight)), 0.0);
            vec3 diffuse = diff * DirectionalIntensity;

            vec3 result = (diffuse + ambient) * IN.Color;

            Color = vec4(result, 1.0f);
        }
    "#;
}
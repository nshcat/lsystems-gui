use nalgebra_glm::{Mat4, Vec3, Vec2};
use std::any::*;
use crate::rendering::*;
use crate::rendering::meshes::*;
use crate::rendering::materials::*;
use crate::rendering::shaders::*;
use crate::rendering::uniforms::*;
use crate::rendering::traits::*;

/// Geometry type used by all types of lines. Vertices contain both color and line width
/// attributes.
pub struct LineGeometry {
    pub positions: AttributeArray<Vec3>,
    pub colors: AttributeArray<Vec3>,
    pub widths: AttributeArray<f32>,
    pub indices: Vec<u32>
}

impl LineGeometry {
    /// Create a new, empty line geometry instance.
    pub fn new() -> LineGeometry {
        LineGeometry {
            positions: AttributeArray::new(0, "position"),
            colors: AttributeArray::new(1, "color"),
            widths: AttributeArray::new(2, "width"),
            indices: Vec::new()
        }
    }

    /// Add line segment with given data to the line geometry.
    pub fn add_segment(&mut self, begin: Vec3, end: Vec3, color: Vec3, width: f32) {
        self.positions.local_buffer.push(begin);
        self.positions.local_buffer.push(end);
        self.colors.local_buffer.push(color.clone());
        self.colors.local_buffer.push(color);
        self.widths.local_buffer.push(width);
        self.widths.local_buffer.push(width);

        let index0 = self.positions.local_buffer.len() - 2;
        let index1 = index0 + 1;
        self.indices.push(index0 as _);
        self.indices.push(index1 as _);
    }
}

impl Geometry for LineGeometry {
    fn retrieve_attributes(&self) -> Vec<&dyn AttributeArrayBase> {
        vec![&self.positions, &self.colors, &self.widths]
    }
}

impl IndexedGeometry for LineGeometry {
    fn retrieve_indices(&self) -> &[u32] {
        return &self.indices;
    }
}


/// A material that uses a geometry shader to turn line segments into 2D lines
/// based on triangle strips.
pub struct Line2DMaterial {
    /// The underlying shader program
    program: Program,
    /// The dimensions of the screen
    pub screen_dimensions: (u32, u32)
}

impl Line2DMaterial {
    /// Create a new instance of this material.
    pub fn new(screen_dimensions: (u32, u32)) -> Line2DMaterial {
        let mut shaders = vec![
            Shader::new_vertex(Self::VERTEX_SHADER_SOURCE).unwrap(),
            Shader::new_fragment(Self::FRAGMENT_SHADER_SOURCE).unwrap(),
            Shader::new_geometry(Self::GEOMETRY_SHADER_SOURCE).unwrap()
        ];

        Line2DMaterial {
            program: Program::from_shaders(
                &mut shaders
            ).unwrap(),
            screen_dimensions: screen_dimensions
        }
    }
}

impl Material for Line2DMaterial {
    fn enable_material(&self, params: &mut RenderParameters) {
        self.program.use_program();

        self.program.set_uniform_mat4("projection", &params.projection);
        self.program.set_uniform_mat4("view", &params.view);
        self.program.set_uniform_mat4("model", &params.model);

        let dims = Vec2::new(self.screen_dimensions.0 as _, self.screen_dimensions.1 as _);
        self.program.set_uniform_vec2("viewport", &dims);
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

impl Line2DMaterial {
    /// The vertex shader source for this material
    const VERTEX_SHADER_SOURCE: &'static str = r#"
        #version 330 core

        layout (location = 0) in vec3 Position;
        layout (location = 1) in vec3 Color;
        layout (location = 2) in float Width; 

        uniform mat4 projection;
        uniform mat4 view;
        uniform mat4 model;

        out Vertex
        {
            vec4 color;
            float width;
        } vertex;

        void main()
        {
            gl_Position = projection * view * model * vec4(Position, 1.0);
            vertex.color = vec4(Color, 1.0);
            vertex.width = Width;
        }
    "#;

    /// The geometry shader source for this material
    const GEOMETRY_SHADER_SOURCE: &'static str = r#"
        #version 330 core

        layout(lines) in;
        layout(triangle_strip, max_vertices = 4) out;

        // The screen dimensions in pixels
        uniform vec2 viewport;   

        in Vertex 
        {
            vec4 color;
            float width;
        } vertex[];

        out vec4 vertex_color;

        
        void main()
        {
            float line_width = vertex[0].width;

            vec3 ndc0 = gl_in[0].gl_Position.xyz / gl_in[0].gl_Position.w;
            vec3 ndc1 = gl_in[1].gl_Position.xyz / gl_in[1].gl_Position.w;

            vec2 lineScreenForward = normalize(ndc1.xy - ndc0.xy);
            vec2 lineScreenRight = vec2(-lineScreenForward.y, lineScreenForward.x);
            vec2 lineScreenOffset = (vec2(line_width) / viewport) * lineScreenRight;

            vec4 cpos0 = gl_in[0].gl_Position;
            gl_Position = vec4(cpos0.xy + lineScreenOffset*cpos0.w, cpos0.z, cpos0.w);
            vertex_color = vertex[0].color;
            EmitVertex();

            vec4 cpos1 = gl_in[0].gl_Position;
            gl_Position = vec4(cpos1.xy - lineScreenOffset*cpos1.w, cpos1.z, cpos1.w);
            vertex_color = vertex[0].color;
            EmitVertex();

            vec4 cpos2 = gl_in[1].gl_Position;
            gl_Position = vec4(cpos2.xy + lineScreenOffset*cpos2.w, cpos2.z, cpos2.w);
            vertex_color = vertex[1].color;
            EmitVertex();

            vec4 cpos3 = gl_in[1].gl_Position;
            gl_Position = vec4(cpos3.xy - lineScreenOffset*cpos3.w, cpos3.z, cpos3.w);
            vertex_color = vertex[1].color;
            EmitVertex();

            EndPrimitive();
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






/// A material that uses a geometry shader to turn line segments into 3D lines
/// based on triangle strips.
pub struct Line3DMaterial {
    /// The underlying shader program
    program: Program
}

impl Line3DMaterial {
    /// Create a new instance of this material.
    pub fn new() -> Line3DMaterial {
        let mut shaders = vec![
            Shader::new_vertex(Self::VERTEX_SHADER_SOURCE).unwrap(),
            Shader::new_fragment(Self::FRAGMENT_SHADER_SOURCE).unwrap(),
            Shader::new_geometry(Self::GEOMETRY_SHADER_SOURCE).unwrap()
        ];

        Line3DMaterial {
            program: Program::from_shaders(
                &mut shaders
            ).unwrap()
        }
    }
}

impl Material for Line3DMaterial {
    fn enable_material(&self, params: &mut RenderParameters) {
        self.program.use_program();

        self.program.set_uniform_mat4("projection", &params.projection);
        self.program.set_uniform_mat4("view", &params.view);
        self.program.set_uniform_mat4("model", &params.model);

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

impl Line3DMaterial {
    /// The vertex shader source for this material
    const VERTEX_SHADER_SOURCE: &'static str = r#"
        #version 330 core

        layout (location = 0) in vec3 Position;
        layout (location = 1) in vec3 Color;
        layout (location = 2) in float Width; 

        out Vertex
        {
            vec4 color;
            float width;
        } vertex;

        void main()
        {
            gl_Position = vec4(Position, 1.0);
            vertex.color = vec4(Color, 1.0);
            vertex.width = Width;
        }
    "#;

    /// The geometry shader source for this material
    const GEOMETRY_SHADER_SOURCE: &'static str = r#"
        #version 330 core

        layout(lines) in;
        layout(triangle_strip, max_vertices = 32) out;

        uniform mat4 projection;
        uniform mat4 view;
        uniform mat4 model;

        in Vertex 
        {
            vec4 color;
            float width;
        } vertex[];

        out vec4 vertex_color;
        out vec3 normal_vector;

        vec3 createPerp(vec3 p1, vec3 p2)
        {
            vec3 invec = normalize(p2 - p1);
            vec3 ret = cross( invec, vec3(0.0, 0.0, 1.0) );
            if ( length(ret) == 0.0 )
            {
                ret = cross(invec, vec3(0.0, 1.0, 0.0) );
            }
            return ret;
        }

        
        void main()
        {
            mat4 mvp = projection * view * model;

            vec3 axis = gl_in[1].gl_Position.xyz - gl_in[0].gl_Position.xyz;

            vec3 perpx = normalize(createPerp(gl_in[1].gl_Position.xyz, gl_in[0].gl_Position.xyz));
            vec3 perpy = cross(normalize(axis), perpx);

            float r1 = vertex[0].width / 1000.0;
            float r2 = vertex[0].width / 1000.0;

            int segs = 16;
            for(int i=0; i<segs; i++) {
                float a = i/float(segs-1) * 2.0 * 3.14159;
                float ca = cos(a); float sa = sin(a);
                vec3 normal = vec3( ca*perpx.x + sa*perpy.x,
                                ca*perpx.y + sa*perpy.y,
                                ca*perpx.z + sa*perpy.z );

                

                vec3 p1 = gl_in[0].gl_Position.xyz + r1*normal;
                vec3 p2 = gl_in[1].gl_Position.xyz + r2*normal;
                
                gl_Position = mvp * vec4(p1, 1.0);
                vertex_color = vertex[0].color;
                normal_vector = normal;
                EmitVertex();

                gl_Position = mvp * vec4(p2, 1.0);
                vertex_color = vertex[0].color;
                normal_vector = normal;
                EmitVertex();       
            }
            EndPrimitive();   
        }
    "#;

    /// The fragment shader source for this material
    const FRAGMENT_SHADER_SOURCE: &'static str = r#"
        #version 330 core

        uniform vec3 AmbientIntensity;
        uniform vec3 DirectionalIntensity;
        uniform vec3 DirectionalLight;

        in vec4 vertex_color;
        in vec3 normal_vector;

        out vec4 Color;

        void main()
        {
            vec3 ambient = AmbientIntensity;
            
            float diff = max(dot(normalize(normal_vector), normalize(DirectionalLight)), 0.0);
            vec3 diffuse = diff * DirectionalIntensity;

            vec3 result = (diffuse + ambient) * vertex_color.xyz;

            Color = vec4(result, 1.0f);
        }
    "#;
}

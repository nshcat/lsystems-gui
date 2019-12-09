use lsystems_core::*;
use lsystems_core::drawing::*;
use lsystems_core::drawing::primitives::*;
use lsystems_core::drawing::types::*;

use nalgebra_glm::Vec3;

use crate::data::*;
use crate::rendering::meshes::*;
use crate::rendering::*;
use crate::rendering::materials::*;
use crate::rendering::traits::*;




/// A struct managing the currently displayed LSystem and providing methods
/// to update certain parts of it.
pub struct LSystemManager {
    /// The parameters describing the currently displayed LSystem.
    pub lsystem_params: LSystemParameters,
    /// The lsystem instance
    lsystem: LSystem,
    /// The mesh containing all lines of the lsystem
    lines_mesh: Mesh
}

impl LSystemManager {
    /// Create LSystem manager instance with given initial lsystem
    pub fn new(params: &LSystemParameters) -> LSystemManager {
        let mut lsystem = LSystem::new();

        Self::setup_lsystem(&mut lsystem, params);

        lsystem.iterate();
        lsystem.interpret();

        let mesh = Self::retrieve_line_mesh(&lsystem, params);

        LSystemManager{
            lsystem_params: params.clone(),
            lines_mesh: mesh,
            lsystem
        }
    }

    /// Notify scene that the  drawing parameters have changed
    pub fn refresh_drawing_parameters(&mut self) {
        self.lsystem.set_drawing_parameters(&self.lsystem_params.drawing_parameters);
        self.draw_lsystem();
    }

    pub fn refresh_iteration_depth(&mut self) {
        self.lsystem.set_iteration_depth(self.lsystem_params.iteration_depth);
        self.iterate_lsystem();
        self.draw_lsystem();
    }

    pub fn refresh_rules(&mut self) {
        self.apply_rules();
        self.iterate_lsystem();
        self.draw_lsystem();
    }

    pub fn refresh_interpretations(&mut self) {
        self.apply_interpretations();
        self.iterate_lsystem();
        self.draw_lsystem();
    }

    /// Apply interpretations in the lsystem parameters to the current lsystem instance
    fn apply_interpretations(&mut self) {
        self.lsystem.interpretations.clear();

        for interpretation in &self.lsystem_params.interpretations {
            if let Some(symbol) = interpretation.symbol {
                self.lsystem.interpretations.associate(symbol, interpretation.operation);
            }
        }
    }

    /// Apply axiom and rules stored in the lsystem parameters to the current lsystem instance
    fn apply_rules(&mut self) {
        self.lsystem.parse(&self.lsystem_params.axiom, &self.lsystem_params.rules.join("\n"));
    }

    /// Fully reiterate the lsystem. This is necessary if the iteration depth, the axiom or one or more 
    /// rules changed.
    fn iterate_lsystem(&mut self) {
        self.lsystem.iterate();
    }

    /// Draw the lsystem, which means interpreting it and retrieving all scene objects from it
    fn draw_lsystem(&mut self) {
        self.lsystem.interpret();
        self.lines_mesh = Self::retrieve_line_mesh(&self.lsystem, &self.lsystem_params);
    }

    /// Setup new lsystem instance using given parameters. This will not start
    /// iteration!
    fn setup_lsystem(lsystem: &mut LSystem, params: &LSystemParameters) {
        lsystem.set_iteration_depth(params.iteration_depth);
        lsystem.set_drawing_parameters(&params.drawing_parameters);
        lsystem.parse(&params.axiom, &params.rules.join("\n"));
        lsystem.engine.set_seed(params.seed);

        for interp in &params.interpretations {
            if let Some(symbol) = interp.symbol {
                lsystem.interpretations.associate(symbol, interp.operation);
            }
        }
    }

    /// Create line mesh from interpreted lsystem
    fn retrieve_line_mesh(lsystem: &LSystem, params: &LSystemParameters) -> Mesh {
        // We are using a flat color material here.
        let mat = Box::new(SimpleMaterial::new());

        // Buffer for line vertices
        let mut vertices = Vec::new();

        // Convert LSystem-Core vector to GLM vector
        fn convert_vector(vec: &Vector3f) -> Vec3 {
            Vec3::new(vec.x as _, vec.y as _, vec.z as _)
        }

        for segment in &lsystem.line_segments {
            // Lookup color
            let color_index = if segment.color >= lsystem.parameters.color_palette_size as _ { 
                lsystem.parameters.color_palette_size - 1
            } else {
                segment.color as _
            };

            let color = if params.color_palette.len() == 0 {
                Vec3::repeat(1.0)
            } else {
                params.color_palette[color_index as usize]
            };

            let begin = Vertex::new(convert_vector(&segment.begin), color);
            let end = Vertex::new(convert_vector(&segment.end), color);
    
            vertices.push(begin);
            vertices.push(end);
        }

        Mesh::new(PrimitiveType::Lines, mat, &BasicGeometry::from_vertices(&vertices))
    }
}

impl Render for LSystemManager {
    /// Render lsystem to screen
    fn render(&self, params: &mut RenderParameters) {
        self.lines_mesh.render(params);
    }
}
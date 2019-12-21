use std::fmt::*;
use imgui::*;
use nalgebra_glm::*;
use ncollide3d::*;
use ncollide3d::math::*;
use ncollide3d::query::*;
use gl::*;
extern crate nalgebra;
use crate::scene::*;
use crate::data::bezier::*;
use crate::rendering::*;
use crate::rendering::bezier::*;
use crate::rendering::camera::*;
use crate::rendering::meshes::*;
use crate::rendering::materials::*;
use crate::rendering::traits::*;
use crate::rendering::model::*;
use crate::rendering::lighting::*;
use crate::scene::lsystem::normal_test_material::*;
use crate::scene::bezier::gizmos::*;
use crate::gui_utils::*;
extern crate glfw;

mod gizmos;

pub struct BezierEditorScene {
    /// Reference to the model to edit. This will only be modifed once the user
    /// hit "save".
    model: RcCell<BezierModelParameters>,
    /// The copy of the model parameters the GUI operates on.
    working_copy: BezierModelParameters,
    /// Camera used to view the patches
    camera: Camera,
    /// All meshes to render.
    meshes: Vec<Mesh>,
    /// Control point visualisation
    control_point_models: Vec<MultiModel>,
    /// Control curve visualisation
    control_curve_meshes: Vec<Mesh>,
    /// Normal vector visualisations
    normal_vector_vis: Vec<Mesh>,
    /// Whether to draw the control curves
    draw_control_curves: bool,
    /// Whether to draw the normal vectors
    draw_normal_vectors: bool,
    /// Screen width
    width: u32,
    /// Screen height
    height: u32,
    /// The sphere mesh used to visualize the control points. Its shared with all control point models.
    sphere_mesh: Rc<Mesh>,
    /// Where the mouse drag started
    drag_begin: Option<(u32, u32)>,
    /// Depth of the point we are dragging
    drag_depth: Option<f32>,
    /// The indices of the patch, curve and point that is currently being dragged.
    dragged_point: Option<(usize, usize, usize)>,
    /// Whether we are currently dragging
    in_drag: bool,
    /// The scenes lights
    lights: LightingContext,
    /// The gizmo visualizing the cardinal axises
    axis_gizmo: OriginGizmo,
    /// Flags describing whether the subpatches are shown in the viewport or not
    active: Vec<bool>,
    /// GUI helper that remembers for which bezier model a certain operation is refering to.
    /// This is needed since for popups to work, they have to be continuously be called, even
    /// long after the information about what button associated with what model has caused this.
    /// This is, for example, used with the popup that ask for confirmation when trying to delete a 
    /// bezier model.
    gui_cached_id: Option<usize>
}

impl BezierEditorScene {
    pub fn new(model: RcCell<BezierModelParameters>, w: u32, h: u32) -> BezierEditorScene {
        let mat = Box::new(SimpleMaterial::new());
        let sphere_geom = SphereGeometry::new(0.01, 40, 40, Vec3::new(1.0, 1.0, 1.0));

        let mut mesh = Mesh::new_indexed(PrimitiveType::TriangleStrip, mat, &sphere_geom);
        mesh.draw_wireframe = false;

        let working_copy = model.borrow().clone();
        let active = vec![true; working_copy.patches.len()];
        let mut scene = BezierEditorScene {
            working_copy: working_copy,
            model: model,
            camera: Camera::new(w, h, ProjectionType::Perspective(75.0)),
            meshes: Vec::new(),
            control_point_models: Vec::new(),
            control_curve_meshes: Vec::new(),
            normal_vector_vis: Vec::new(),
            draw_control_curves: true,
            width: w,
            height: h,
            sphere_mesh: Rc::new(mesh),
            in_drag: false,
            drag_depth: None,
            drag_begin: None,
            dragged_point: None,
            lights: LightingContext::new_default(),
            draw_normal_vectors: false,
            axis_gizmo: OriginGizmo::new(0.3, 3.5),
            active: active,
            gui_cached_id: None
        };

        scene.refresh_meshes();

        scene
    }
}

impl BezierEditorScene {
    /// Just refresh the mesh for the patch with given index
    fn refresh_mesh_for(&mut self, index: usize) {
        let patch = &self.working_copy.patches[index];

        let mesh = self.create_mesh(patch);
        self.meshes[index] = mesh;

        let control_point_model = self.create_control_point_model(patch);
        self.control_point_models[index] = control_point_model;

        let control_curve_mesh = self.create_control_curve_mesh(patch);
        self.control_curve_meshes[index] = control_curve_mesh;

        if self.draw_normal_vectors {
            let normal_mesh = self.create_normal_mesh(patch);
            self.normal_vector_vis[index] = normal_mesh;
        }
    }

    /// Refresh all patch meshes
    fn refresh_meshes(&mut self) {
        self.meshes = Vec::new();
        self.normal_vector_vis = Vec::new();

        for patch in &self.working_copy.patches {
            self.meshes.push(self.create_mesh(patch));

            if self.draw_normal_vectors {
                self.normal_vector_vis.push(self.create_normal_mesh(patch));
            }
        }

        self.refresh_control_meshes();
    }

    fn create_normal_mesh(&self, patch: &BezierPatchParameters) -> Mesh {
        let geometry = BezierGeometry::new(patch, 30, 30);

        let mat = Box::new(NormalTestMaterial::new(0.05, &Vec3::new(1.0, 1.0, 0.0)));

        let mut mesh = Mesh::new_indexed(PrimitiveType::TriangleStrip, mat, &geometry);
        mesh
    }
    
    /// Refresh the control point meshes only for the currently dragged point
    fn refresh_control_meshes_for_dragged(&mut self) {
        if let Some((i, j, k)) = self.dragged_point {
            let patch = &self.working_copy.patches[i];

            self.control_curve_meshes[i] = self.create_control_curve_mesh(patch);
            self.control_point_models[i] = self.create_control_point_model(patch);
        }
    }

    fn refresh_control_meshes(&mut self) {
        self.control_point_models = Vec::new();
        self.control_curve_meshes = Vec::new();

        for patch in &self.working_copy.patches {
            self.control_point_models.push(self.create_control_point_model(patch));
            self.control_curve_meshes.push(self.create_control_curve_mesh(patch));
        }
    }

    fn create_mesh(&self, patch: &BezierPatchParameters) -> Mesh {
        let geometry = BezierGeometry::new(patch, 30, 30);

        let mat = Box::new(ShadedMaterial::new());

        let mut mesh = Mesh::new_indexed(PrimitiveType::TriangleStrip, mat, &geometry);
        mesh
    }

    fn create_control_point_model(& self, patch: &BezierPatchParameters) -> MultiModel { 
        let mut spheres = Vec::new();
        
        for curve in &patch.curves {
            for i in 0..4 {
                let point = &curve.control_points[i];

                spheres.push(
                    Model::from_mesh_transformed_rc(self.sphere_mesh.clone(), Mat4::new_translation(&point))  
                );
            }
        }

        MultiModel::from_models(spheres)
    }

    fn create_control_curve_mesh(&self, patch: &BezierPatchParameters) -> Mesh {
        let mut points = Vec::new();

        for curve in &patch.curves {
            for i in 1..4 {
                points.push(curve.control_points[i-1].clone());
                points.push(curve.control_points[i].clone());
            }
        }

        let mut geom = BasicGeometry::new();
        geom.colors.local_buffer = vec![Vec3::new(1.0, 1.0, 0.0); points.len()];
        geom.normals.local_buffer = vec![Vec3::new(0.0, 0.0, 0.0); points.len()];
        geom.positions.local_buffer = points;

        let mat = Box::new(SimpleMaterial::new());

        let mut mesh = Mesh::new(PrimitiveType::Lines, mat, &geom);

        mesh
    }

    /// Unproject a given window position to a point in world space
    fn unproject(&self, x: u32, y: u32, depth: f32) -> Vec3 {
        unproject(
            &Vec3::new(x as _, (self.height - y) as _, depth),
            &self.camera.view,
            &self.camera.projection,
            Vec4::new(0.0, 0.0, self.width as _, self.height as _)
        )
    }

    /// Returns clicked control point and its depth
    fn find_clicked_control_point(&mut self, x: u32, y: u32) -> Option<(f32, usize, usize, usize)> {
        // Retrieve depth value
        let mut depth: f32 = 0.0;
        unsafe {
            gl::ReadPixels(
                x as _,
                (self.height - y) as _,
                1 as _,
                1 as _,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                &mut depth as *mut f32 as _
            );
        }

        let position = self.unproject(x, y, depth);

        // We now have the rough position. We now have to intersect a ball around it with the balls around all
        // the other control points to find the closest point to it.

        // The sphere used for all points
        let sphere = shape::Ball::<f32>::new(0.01);
        
        // Create translation for ball around point
        let position_isometry = Isometry::new(position.clone(), nalgebra::zero());    

        // Collect intersection results
        let mut control_point: Option<&mut Vec3> = None;

        // We check if a sphere around the clicked point intersects which spheres around any of the
        // control points, in order to retrieve which of the control points is the clicked one.
        for (i, patch) in self.working_copy.patches.iter().enumerate() {
            // If the patch is not currently set to be active, skip it. Otherwise, the user could
            // modify invisible control points, which is not good.
            if !self.active[i] {
                continue;
            }

            for (j, curve) in patch.curves.iter().enumerate() {
                for k in 0..4 {
                    let point = &curve.control_points[k];

                    let translation = Isometry::new(point.clone(), nalgebra::zero());

                    // TODO: it could be multiple intersections. Calculate actual center distance for each, 
                    // and return point with lowest value.
                    let result = proximity(
                        &position_isometry, &sphere,
                        &translation, &sphere, 0.01);

                    match result {
                        Proximity::Intersecting => {
                            return Some((depth, i, j, k));
                        },
                        _ => {}
                    };
                }
            }
        }

        return None;
    }
}

impl Scene for BezierEditorScene {
    /// Render scene to screen. This also includes any GUI components.
    fn render(&self) {
        let mut rp = self.camera.to_render_parameters();
        rp.lighting = self.lights.clone();

        self.axis_gizmo.render(&mut rp);

        // The use of a range based for loop here might seem odd, but we
        // need to be able to check for each patch if its currently activated.
        for i in 0..self.working_copy.patches.len() {
            // Skip this patch and all its associated meshes and models if its not activated.
            if !self.active[i] {
                continue;
            }

            self.meshes[i].render(&mut rp);
            self.control_point_models[i].render(&mut rp);

            if self.draw_control_curves {
                self.control_curve_meshes[i].render(&mut rp);
            }
            
            if self.draw_normal_vectors {
                self.normal_vector_vis[i].render(&mut rp);
            }
        }
    }

    fn do_logic(&mut self) {

    }

    /// Show imgui GUI if needed.
    fn do_gui(&mut self, ui: &Ui) -> SceneAction {
        let mut action = SceneAction::Nothing;

        imgui::Window::new(im_str!("Bezier Model Editor"))
            .size([250.0, 500.0], Condition::FirstUseEver)
            .position([0.0, 60.0], Condition::FirstUseEver)
            .build(&ui, || {
                if ui.collapsing_header(im_str!("Models"))
                    .default_open(true)
                    .build() {
                    ui.indent();

                    let mut modified: Option<usize> = None;
                    let mut refresh_all = false;
                    let mut to_remove: Option<usize> = None;

                    // This weird hack is needed since we can't open the menu directly inside the
                    // for loops, since we are using the id stack. For example "open_popup("bla")" would
                    // in reality refer to "some##id##stack##elements##bla", and we would not be able to
                    // actually draw this popup outside the loops since we cant reconstruct that id!
                    let mut show_delete_popup: Option<usize> = None;
                    let mut show_clone_menu: Option<usize> = None;

                    for (i, patch) in self.working_copy.patches.iter_mut().enumerate() {
                        let patch_id = ui.push_id(i as i32);

                        let mut label = ImString::with_capacity(128);
                        label.push_str(&format!("Model '{}'", i));

                        if ui.collapsing_header(&label)
                            .default_open(false)
                            .build() {
                            ui.indent();

                            ui.checkbox(im_str!("Active"), &mut self.active[i]);
                            ui.same_line(0.0);
                            help_marker(ui, im_str!("Inactive models and their control points and curves are not rendered in the editor viewport."));


                            ui.same_line(345.0);
                    
                            if ui.button(im_str!("Clone.."), [0.0, 0.0]) {
                                show_clone_menu = Some(i);     
                            }
                            
                            let colors = ui.push_style_colors(&[
                                (StyleColor::Button, [0.6, 0.239, 0.239, 1.0]),
                                (StyleColor::ButtonHovered, [0.7, 0.2117, 0.2117, 1.0]),
                                (StyleColor::ButtonActive, [0.8, 0.1607, 0.1607, 1.0])
                            ]);        
            
                            ui.same_line(412.0);

                            if ui.button(im_str!("Remove"), [0.0, 0.0]) {
                                show_delete_popup = Some(i);      
                            }
                    
                            colors.pop(ui);

                            let mut color: [f32; 3] = [patch.color.x, patch.color.y, patch.color.z];

                            if ColorEdit::new(im_str!("Model Color"), &mut color).build(ui) {
                                let new_color = Vec3::new(color[0], color[1], color[2]);
                                patch.color = new_color;
                                modified = Some(i);
                            }

                            if ui.collapsing_header(im_str!("Control Points"))
                                .default_open(false)
                                .build() {

                                ui.indent();
                            
                                for (j, curve) in patch.curves.iter_mut().enumerate() {
                                    let token = ui.push_id(j as i32);

                                    let mut label = ImString::with_capacity(48);
                                    label.push_str(&format!("Curve {}", j));

                                    ui.text(&label);

                                    ui.indent();

                                    for k in 0..4 {
                                        let point = &mut curve.control_points[k];

                                        let mut data = [point.x, point.y, point.z];

                                        let mut label = ImString::with_capacity(48);
                                        label.push_str(&format!("Point {}", k));

                                        if ui.drag_float3(&label, &mut data) 
                                            .min(-500.0)
                                            .max(500.0)
                                            .display_format(im_str!("%.2lf"))
                                            .speed(0.06)
                                            .build() {
                                                *point = Vec3::new(data[0], data[1], data[2]);
                                                modified = Some(i);
                                        }
                                    }

                                    ui.unindent();

                                    token.pop(ui);
                                }

                                ui.unindent();
                            }

                            ui.unindent();
                        }

                        patch_id.pop(ui);
                    }

                    let colors = ui.push_style_colors(&[
                        (StyleColor::Button, [0.349, 0.6, 0.239, 1.0]),
                        (StyleColor::ButtonHovered, [0.3568, 0.7019, 0.2117, 1.0]),
                        (StyleColor::ButtonActive, [0.3529, 0.8, 0.1607, 1.0])
                    ]);
                
                    if ui.button(im_str!("+"), [0.0, 0.0]) {
                        self.working_copy.patches.push(BezierPatchParameters::default());
                        self.active.push(true);
                        refresh_all = true;
                    }
                
                    colors.pop(ui);

                    ui.unindent();

                    if let Some(i) = show_delete_popup {
                        self.gui_cached_id = Some(i);
                        ui.open_popup(im_str!("Delete model?"));
                        show_delete_popup = None;
                    }

                    if let Some(i) = show_clone_menu {
                        self.gui_cached_id = Some(i);
                        ui.open_popup(im_str!("Clone"));
                        show_clone_menu = None;
                    }
                       
                    if let Some(button) = show_popup(ui, im_str!("Delete model?"), im_str!("Do you really want to delete the selected model?"), &vec![PopupButton::Yes, PopupButton::No]) {
                        match button {
                            PopupButton::Yes => {
                                // Handle deletion
                                let index = self.gui_cached_id.unwrap();

                                self.active.remove(index);
                                self.working_copy.patches.remove(index);

                                refresh_all = true;
                            },
                            _ => {}
                        }
                    }

                    ui.popup(im_str!("Clone"), || {
                        let mut clone_action: Option<MirrorPlane> = None;

                        if Selectable::new(im_str!("Simple Clone")).build(ui) {
                            clone_action = Some(MirrorPlane::None);
                        }

                        ui.separator();
                        ui.text(im_str!("Mirrored Clone"));
                        ui.same_line(0.0);
                        help_marker(ui, im_str!("A mirroring clone uses the selected plane to mirror all control points."));

                        if Selectable::new(im_str!(".. on XY plane")).build(ui) {
                            clone_action = Some(MirrorPlane::XY);
                        }

                        if Selectable::new(im_str!(".. on XZ plane")).build(ui) {
                            clone_action = Some(MirrorPlane::XZ);
                        }
                        
                        if Selectable::new(im_str!(".. on YZ plane")).build(ui) {
                            clone_action = Some(MirrorPlane::YZ);
                        }

                        if let Some(plane) = clone_action {
                            let new_patch = self.working_copy.patches[self.gui_cached_id.unwrap()].clone_mirrored(plane);
                            self.working_copy.patches.push(new_patch);
                            self.active.push(true);
                            refresh_all = true;
                        }       
                    });
                   

                    if refresh_all {
                        self.refresh_meshes();
                    } else if let Some(i) = modified {
                        self.refresh_mesh_for(i);
                    }
                }
                if ui.collapsing_header(im_str!("Lighting"))
                    .default_open(false)
                    .build() {
                    ui.indent();          

                    {
                        let mut data = [self.lights.ambient_intensity.x, self.lights.ambient_intensity.y, self.lights.ambient_intensity.z];

                        if ui.drag_float3(im_str!("Ambient Light"), &mut data) 
                            .min(0.0)
                            .max(1.0)
                            .display_format(im_str!("%.3lf"))
                            .speed(0.06)
                            .build() {
                                self.lights.ambient_intensity = Vec3::new(data[0], data[1], data[2]);
                        }
                    }

                    {
                        let mut data = [self.lights.directional_light.x, self.lights.directional_light.y, self.lights.directional_light.z];

                        if ui.drag_float3(im_str!("Directional Light Angle"), &mut data) 
                            .min(-5.0)
                            .max(5.0)
                            .display_format(im_str!("%.3lf"))
                            .speed(0.0006)
                            .build() {
                                self.lights.directional_light = Vec3::new(data[0], data[1], data[2]);
                        }
                    }   

                    {
                        let mut data = [self.lights.directional_intensity.x, self.lights.directional_intensity.y, self.lights.directional_intensity.z];

                        if ui.drag_float3(im_str!("Directional Light Intensity"), &mut data) 
                            .min(0.0)
                            .max(1.0)
                            .display_format(im_str!("%.3lf"))
                            .speed(0.06)
                            .build() {
                                self.lights.directional_intensity = Vec3::new(data[0], data[1], data[2]);
                        }
                    }  

                    ui.unindent();
                }

                if ui.collapsing_header(im_str!("Settings"))
                    .default_open(false)
                    .build() {
                    ui.indent();          

                    ui.checkbox(im_str!("Draw control curves"), &mut self.draw_control_curves);

                    if ui.checkbox(im_str!("Draw normal vectors"), &mut self.draw_normal_vectors) {
                        self.refresh_meshes();
                    }

                    ui.unindent();
                }

                ui.spacing();

                if ui.button(im_str!("Cancel"), [0.0, 0.0]) {
                    action = SceneAction::PopScene;
                }

                ui.same_line(0.0);

                if ui.button(im_str!("Save"), [0.0, 0.0]) {
                    *self.model.borrow_mut() = self.working_copy.clone();
                }

                ui.same_line(0.0);

                if ui.button(im_str!("Save and Exit"), [0.0, 0.0]) {
                    *self.model.borrow_mut() = self.working_copy.clone();
                    action = SceneAction::PopScene;
                }
        });

        action
    }

    /// Handle input event. This is only called if the UI does not want to grab input.
    fn handle_event(&mut self, window: &glfw::Window, event: &glfw::WindowEvent) {
        // MouseButton(MouseButton, Action, Modifiers)
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, glfw::Action::Press, _) => {
                let (x, y) = window.get_cursor_pos();
                // If the user has clicked on one of the control points of the bezier patch, start
                // drag process.
                if let Some((d, i, j, k)) = self.find_clicked_control_point(x as _, y as _) {
                    self.drag_begin = Some((x as _, y as _));
                    self.drag_depth = Some(d);
                    self.in_drag = true;
                    self.dragged_point = Some((i, j, k));
                }
            },
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, glfw::Action::Release, _) => {
                if self.in_drag {
                    self.in_drag = false;
                    self.refresh_meshes();
                }
            },
            glfw::WindowEvent::CursorPos(x, y) => {
                if self.in_drag {
                    // If the user drags the cursor outside of the window, stop dragging process.
                    if *x >= 0.0 && *x <= (self.width as f64) && *y >= 0.0 && *y <= (self.height as f64) {
                        // If we are in drag, we project the new mouse screen position into the scene with the same
                        // depth as the control point at the old position, and use that new 3D position
                        // as our new control position.
                        let curX = *x as u32;
                        let curY = *y as u32;

                        let (oldX, oldY) = self.drag_begin.unwrap();
                        let new_point = self.unproject(curX, curY, self.drag_depth.unwrap());
                        
                        let (i, j, k) = self.dragged_point.unwrap();
                        let p = &mut self.working_copy.patches[i].curves[j].control_points[k];
                        *p = new_point.clone();
            
                        self.drag_begin = Some((curX, curY));     
                        
                        // We only need to update the control meshes for the currently dragged point
                        self.refresh_control_meshes_for_dragged();
                    } else {
                        self.in_drag = false;
                        self.refresh_mesh_for(self.dragged_point.unwrap().0);
                    }
                }
            },
            _ => {}
        };

        // We do not want to move the camera when the user is currently dragging a control point.
        if !self.in_drag {
            self.camera.handle_event(window, event);
        }
    }

    /// Handle window resize event.
    fn handle_resize(&mut self, w: u32, h: u32) {
        self.camera.update(w, h);

        self.width = w;
        self.height = h;
    }
}
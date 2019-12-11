use imgui::{MenuItem, EditableColor, ColorEdit, ImStr, StyleColor, ImString, ImColor, Slider, Condition, Context as ImContext, Window as ImWindow, im_str, Ui};
use nalgebra_glm::Vec3;
use crate::scene::*;
use crate::data::*;
use lsystems_core::drawing::types::*;
use lsystems_core::drawing::DrawOperation;

/// Show a help marker that shows a tooltip with given message on hover-over.
/// Taken from the imgui demo window source code.
fn help_marker(ui: &Ui, text: &ImStr) {
    ui.text_disabled(im_str!("(?)"));
    if ui.is_item_hovered() {
        ui.tooltip_text(text);
    }
}

fn do_color_palette_entry(ui: &Ui, value: &mut Vec3, idx: usize) -> bool {
    let outer_id = ui.push_id(idx as i32);
    
    let mut color: [f32; 3] = [value.x, value.y, value.z];

    let mut changed = false;
    if ColorEdit::new(im_str!(""), &mut color).build(ui) {
        let new_color = Vec3::new(color[0], color[1], color[2]);
        *value = new_color;
        changed = true;
    }

    outer_id.pop(ui);
    return changed;
}

pub fn do_lsystem_params_gui(ui: &Ui, lsystem: &mut LSystemScene) {
    ImWindow::new(im_str!("LSystem Parameters"))
            .size([450.0, 550.0], Condition::FirstUseEver)
            .position([0.0, 60.0], Condition::FirstUseEver)
            .build(&ui, || {
                if ui.collapsing_header(im_str!("Drawing Parameters"))
                    .default_open(true)
                    .build() {
                    ui.indent();
                    do_drawing_parameters(ui, lsystem);
                    ui.unindent();
                }

                if ui.collapsing_header(im_str!("Rules"))
                    .default_open(true)
                    .build() {
                    ui.indent();
                    do_rules(ui, lsystem);
                    ui.unindent();
                }

                if ui.collapsing_header(im_str!("Interpretation Map"))
                    .default_open(true)
                    .build() {
                    ui.indent();
                    do_interpretations(ui, lsystem);
                    ui.unindent();
                }

                if ui.collapsing_header(im_str!("Color Palette"))
                    .default_open(false)
                    .build() {
                    ui.indent();
                    do_colors(ui, lsystem);
                    ui.unindent();
                }

                if ui.collapsing_header(im_str!("Application Settings"))
                    .default_open(true)
                    .build() {
                    ui.indent();
                    do_app_settings(ui, lsystem);
                    ui.unindent();
                }

                if ui.collapsing_header(im_str!("Debug Options"))
                    .default_open(false)
                    .build() {
                    ui.indent();
                    do_debug_options(ui, lsystem);
                    ui.unindent();
                }
            });
}

fn draw_operations() -> Vec<&'static ImStr> {
    vec![
        im_str!("Forward"),
        im_str!("ForwardNoDraw"),
        im_str!( "TurnRight"),
        im_str!("TurnLeft"),
        im_str!("SaveState"),
        im_str!("LoadState"),
        im_str!("Ignore"),
        im_str!("ForwardContracting"),
        im_str!("PitchDown"),
        im_str!("PitchUp"),
        im_str!("RollLeft"),
        im_str!("RollRight"),
        im_str!("TurnAround"),
        im_str!("BeginPolygon"),
        im_str!("EndPolygon"),
        im_str!("SubmitVertex"),
        im_str!("IncrementColor"),
        im_str!("DecrementColor"),
        im_str!("IncrementLineWidth"),
        im_str!("DecrementLineWidth"),
    ]
}

fn index_to_operation(index: usize) -> DrawOperation {
    match index {
        0 => DrawOperation::Forward,
        1 => DrawOperation::ForwardNoDraw,
        2 => DrawOperation::TurnRight,
        3 => DrawOperation::TurnLeft,
        4 => DrawOperation::SaveState,
        5 => DrawOperation::LoadState,
        6 => DrawOperation::Ignore,
        7 => DrawOperation::ForwardContracting,
        8 => DrawOperation::PitchDown,
        9 => DrawOperation::PitchUp,
        10 => DrawOperation::RollLeft,
        11 => DrawOperation::RollRight,
        12 => DrawOperation::TurnAround,
        13 => DrawOperation::BeginPolygon,
        14 => DrawOperation::EndPolygon,
        15 => DrawOperation::SubmitVertex,
        16 => DrawOperation::IncrementColor,
        17 => DrawOperation::DecrementColor,
        18 => DrawOperation::IncrementLineWidth,
        19 => DrawOperation::DecrementLineWidth,
        _ => panic!("Unknown draw operation value")
    }
}

fn do_colors(ui: &Ui, lsystem: &mut LSystemScene) {
    let mut was_changed = false;
    for (i, color) in &mut lsystem.lsystem_params.color_palette.iter_mut().enumerate() {
        if do_color_palette_entry(ui, color, i) {
            was_changed = true;
        }
    }

    if was_changed {
        lsystem.refresh_color_palette();
    }
}

pub fn do_main_menu_bar(ui: &Ui, lsystem: &mut LSystemScene) {
    if let Some(token) = ui.begin_main_menu_bar() {
        do_file_menu(ui, lsystem);
        token.end(ui);
    }
}

fn do_file_menu(ui: &Ui, lsystem: &mut LSystemScene) {
    if let Some(token) = ui.begin_menu(im_str!("File"), true) {
        if MenuItem::new(im_str!("New"))
            .shortcut(im_str!("      Ctrl+N"))
            .build(ui) {
                
        }

        if MenuItem::new(im_str!("Open"))
            .shortcut(im_str!("      Ctrl+O"))
            .build(ui) {

        }

        if MenuItem::new(im_str!("Save"))
            .shortcut(im_str!("      Ctrl+S"))
            .build(ui) {
            
        }

        token.end(ui);
    }
}

fn do_debug_options(ui: &Ui, lsystem: &mut LSystemScene) {
    if ui.checkbox(im_str!("Show normal vectors"), &mut lsystem.app_settings.show_normals) {
        lsystem.refresh_meshes();
    }

    if ui.checkbox(im_str!("Draw polygons as wireframe"), &mut lsystem.app_settings.draw_wireframe) {
        lsystem.refresh_wireframe_flag();
    }
}

fn do_interpretations(ui: &Ui, lsystem: &mut LSystemScene) {
    let mut modified = false;

    let params = &mut lsystem.lsystem_params;

    let mut to_delete: Option<usize> = None;

    // We need to push an outer ID here since we are using buttons with the same identifiers as the ones
    // used to remove and add rules.
    let outer_id = ui.push_id(2);

    for (i, interp) in params.interpretations.iter_mut().enumerate() {
        let id = ui.push_id(i as i32);

        let mut symbol_str = ImString::with_capacity(16);

        if let Some(symbol) = interp.symbol {
            symbol_str.push_str(&symbol.to_string());
        }

        let token = ui.push_item_width(20.0);

        if ui.input_text(im_str!("##sym"), &mut symbol_str).build() {
            let trimmed = symbol_str.to_str().trim();
            if trimmed.is_empty() {
                interp.symbol = None;
            } else {
                interp.symbol = Some(trimmed.chars().next().unwrap());
            }

            modified = true;
        }

        token.pop(ui);

        ui.same_line(0.0);
        ui.text(im_str!("->"));
        ui.same_line(0.0);
        let mut current_item: i32 = interp.operation as _;
        let items = draw_operations();

        if ui.combo(im_str!("##op"), &mut current_item, &items, 5) {
            let new_operation = index_to_operation(current_item as _);
            interp.operation = new_operation;
            modified = true;
        }

        let colors = ui.push_style_colors(&[
            (StyleColor::Button, [0.6, 0.239, 0.239, 1.0]),
            (StyleColor::ButtonHovered, [0.7, 0.2117, 0.2117, 1.0]),
            (StyleColor::ButtonActive, [0.8, 0.1607, 0.1607, 1.0])
        ]);

        ui.same_line(0.0);

        if ui.button(im_str!("-"), [0.0, 0.0]) {
            println!("To delete: {}", i);
            modified = true;
            to_delete = Some(i);
        }
        
        colors.pop(ui);

        id.pop(ui);     
    }  

    match to_delete {
        Some(i) => {
            params.interpretations.remove(i);
        }
        _ => {}
    };


    let colors = ui.push_style_colors(&[
        (StyleColor::Button, [0.349, 0.6, 0.239, 1.0]),
        (StyleColor::ButtonHovered, [0.3568, 0.7019, 0.2117, 1.0]),
        (StyleColor::ButtonActive, [0.3529, 0.8, 0.1607, 1.0])
    ]);

    if ui.button(im_str!("+"), [0.0, 0.0]) {
        params.interpretations.push(
            Interpretation{
                symbol: None,
                operation: DrawOperation::Forward
            }
        );

        println!("Added new interp, size is now {}", params.interpretations.len());
        modified = true;
    }

    colors.pop(ui);
    outer_id.pop(ui);

    if modified {
        lsystem.refresh_interpretations();
    }
}

fn do_rules(ui: &Ui, lsystem: &mut LSystemScene) {
    let mut modified = false;
    let params = &mut lsystem.lsystem_params;

    let mut axiom = ImString::with_capacity(256);
    axiom.push_str(&params.axiom);
    if ui.input_text(im_str!("Axiom"), &mut axiom).build() {
        params.axiom = axiom.to_str().to_string();
        modified = true;
    }

    ui.text(im_str!("Production rules:"));
    ui.indent();

    // The rule to delete. It can only ever be one per frame, so this is enough.
    let mut to_delete = None;

    for (i, rule) in params.rules.iter_mut().enumerate() {
        let mut rule_str = ImString::with_capacity(256);
        rule_str.push_str(rule);

        let id = ui.push_id(i as i32);

        if ui.input_text(im_str!("##rule"), &mut rule_str).build() {
            *rule = rule_str.to_str().to_string();
            modified = true;
        }

        let colors = ui.push_style_colors(&[
            (StyleColor::Button, [0.6, 0.239, 0.239, 1.0]),
            (StyleColor::ButtonHovered, [0.7, 0.2117, 0.2117, 1.0]),
            (StyleColor::ButtonActive, [0.8, 0.1607, 0.1607, 1.0])
        ]);

        ui.same_line(0.0);

        if ui.button(im_str!("-"), [0.0, 0.0]) {
            modified = true;
            to_delete = Some(i);
        }

        colors.pop(ui);
        id.pop(ui);
    }

    let colors = ui.push_style_colors(&[
        (StyleColor::Button, [0.349, 0.6, 0.239, 1.0]),
        (StyleColor::ButtonHovered, [0.3568, 0.7019, 0.2117, 1.0]),
        (StyleColor::ButtonActive, [0.3529, 0.8, 0.1607, 1.0])
    ]);

    if ui.button(im_str!("+"), [0.0, 0.0]) {
        params.rules.push(String::new());
        modified = true;
    }

    colors.pop(ui);



    ui.unindent();

    // Handle deletion request
    match to_delete {
        Some(i) => {
            params.rules.remove(i);
        },
        _ => {}
    };

    if modified {
        lsystem.refresh_rules();
    }
}

fn do_drawing_parameters(ui: &Ui, lsystem: &mut LSystemScene) {  
    {
        let mut modified = false;
        let params = &mut lsystem.lsystem_params.drawing_parameters;

        let mut start_pos: [f32; 2] = [params.start_position.x as _, params.start_position.y as _,];
        if ui.drag_float2(im_str!("Starting position"), &mut start_pos)
            .min(-500.0)
            .max(500.0)
            .display_format(im_str!("%.2lf"))
            .speed(0.06)
            .build() {
                params.start_position = Vector2f::new(start_pos[0] as _, start_pos[1] as _);
                modified = true;
        }

        let mut start_angle: f32 = params.start_angle.to_degrees() as _;
        if ui.drag_float(im_str!("Starting angle"), &mut start_angle)
            .min(0.0)
            .max(360.0)
            .display_format(im_str!("%.lf"))
            .speed(1.0)
            .build() {
                params.start_angle = (start_angle as f64).to_radians();
                modified = true;
        }

        let mut delta_angle: f32 = params.angle_delta.to_degrees() as _;
        if ui.drag_float(im_str!("Angle delta"), &mut delta_angle)
            .min(0.0)
            .max(360.0)
            .display_format(im_str!("%.lf"))
            .speed(1.0)
            .build() {
                params.angle_delta = (delta_angle as f64).to_radians();
                modified = true;
        }

        let mut step: f32 = params.step as _;
        if ui.drag_float(im_str!("Step"), &mut step)
            .min(0.0)
            .max(360.0)
            .display_format(im_str!("%.2lf"))
            .speed(0.01)
            .build() {
                params.step = step as _;
                modified = true;
        }

        if modified {
            lsystem.refresh_drawing_parameters();
        }
    }

    // Technically, the iteration depth is not path of the drawing parameters, but it is displayed in the same section. 
    {
        if Slider::<u32>::new(im_str!("Iterations"), 0..=13).build(ui, &mut lsystem.lsystem_params.iteration_depth) {
            lsystem.refresh_iteration_depth();
        }
    }
}

pub fn do_debug_gui(ui: &Ui) {
    ImWindow::new(im_str!("Debug"))
            .size([85.0, 55.0], Condition::Always)
            .position([0.0, 0.0], Condition::Always)
            .build(&ui, || {
                let fps = ui.io().framerate;
                ui.text(format!(
                    "FPS: {:.1}",
                    fps
                ));
                
            });
}

fn do_app_settings(ui: &Ui, lsystem: &mut LSystemScene) {
    ui.checkbox(im_str!("Auto refresh"), &mut lsystem.app_settings.auto_refresh);

    if !lsystem.app_settings.auto_refresh {
        ui.same_line_with_spacing(0.0, 30.0);
        if ui.button(im_str!("Reload"), [0.0, 0.0]) {
            lsystem.force_refresh_all();
        }
    } else {
        ui.same_line(0.0);
        help_marker(ui, im_str!("Auto refresh can make editing large L-Systems rather slow. Consider disabling the option when dealing with big systems."))
    }

    ui.spacing();

    ui.checkbox(im_str!("Center camera on reload"), &mut lsystem.app_settings.auto_center_camera);
    ui.same_line(0.0);
    help_marker(ui, im_str!("Causes the camera to be focused on the center of the L-System's bounding box on reload, which makes rotation more enjoyable."));
    ui.same_line_with_spacing(0.0, 30.0);
    if ui.button(im_str!("Center"), [0.0, 0.0]) {
        lsystem.center_camera();
    }

    if lsystem.app_settings.auto_center_camera {
        ui.indent();
        ui.checkbox(im_str!("Also adjust camera zoom"), &mut lsystem.app_settings.auto_adjust_radius);ui.same_line(0.0);
        help_marker(ui, im_str!("This will adjust the zoom level to always have the whole L-System in view."));    
        ui.unindent();
    }

    ui.spacing();

    ui.checkbox(im_str!("Draw bounding box"), &mut lsystem.app_settings.draw_bounding_box);

    if lsystem.app_settings.draw_bounding_box {
        let bbcolor = &mut lsystem.app_settings.bounding_box_color;
        let mut color: [f32; 3] = [bbcolor.x, bbcolor.y, bbcolor.z];

        ui.indent();
        if ColorEdit::new(im_str!("Box color"), &mut color).build(ui) {
            let new_color = Vec3::new(color[0], color[1], color[2]);
            lsystem.app_settings.bounding_box_color = new_color;
            lsystem.refresh_bounding_box_color();
        }
        ui.unindent();
    }
}
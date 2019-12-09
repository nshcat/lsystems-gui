use imgui::{StyleColor, ImString, ImColor, Slider, Condition, Context as ImContext, Window as ImWindow, im_str, Ui};
use crate::scene::*;
use lsystems_core::drawing::types::*;

pub fn do_lsystem_params_gui(ui: &Ui, lsystem: &mut LSystemManager) {
    ImWindow::new(im_str!("LSystem Parameters"))
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(&ui, || {
                if ui.collapsing_header(im_str!("Drawing Parameters")).build() {
                    ui.indent();
                    do_drawing_parameters(ui, lsystem);
                    ui.unindent();
                }

                if ui.collapsing_header(im_str!("Rules")).build() {
                    ui.indent();
                    do_rules(ui, lsystem);
                    ui.unindent();
                }
            });
}

fn do_rules(ui: &Ui, lsystem: &mut LSystemManager) {
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

fn do_drawing_parameters(ui: &Ui, lsystem: &mut LSystemManager) {  
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
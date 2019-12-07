
use lsystems_core::*;
use lsystems_core::drawing::types::*;
use lsystems_core::drawing::{DrawOperation, DrawingParameters};
use glfw::{Action, Key, Context};
use imgui::{Condition, Context as ImContext, Window as ImWindow, im_str};
use imgui_glfw_rs::glfw;
use imgui_glfw_rs::imgui;
use imgui_glfw_rs::ImguiGLFW;
use nalgebra_glm::{Mat4, Vec3};

use rendering::shaders::{Shader, ShaderType, Program};
use rendering::uniforms::*;
use rendering::buffers::*;
use rendering::meshes::*;
use rendering::traits::*;
use rendering::materials::*;
use rendering::{RenderParameters};

mod rendering;

fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));

    let (mut window, events) = glfw
        .create_window(
            1024,
            768,
            "lsystems-gui",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create window");

    window.make_current();
    window.set_all_polling(true);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
    }

    let mut imgui = ImContext::create();

    let mut imgui_glfw = ImguiGLFW::new(&mut imgui, &mut window);

    let mut show_menu = true;

    // ======== Scene setup =================


    let mat = Box::new(SimpleMaterial::new());

    let mut vertices = Vec::new();

    fn convert_vector(vec: &Vector3f) -> Vec3 {
        Vec3::new(vec.x as _, vec.y as _, vec.z as _)
    }

    let params = DrawingParameters {
        step: 0.1,
        angle_delta: 60.0_f64.to_radians(),
        .. DrawingParameters::new()
    };

    let mut system = LSystem::new();
    system.parse("F--F--F", "F -> F+F--F+F");
    system.interpretations.associate('F', DrawOperation::Forward);
    system.interpretations.associate('+', DrawOperation::TurnRight);
    system.interpretations.associate('-', DrawOperation::TurnLeft);
    system.set_drawing_parameters(&params);
    system.set_iteration_depth(2);
    system.iterate();
    system.interpret();

    for segment in &system.line_segments {
        let begin = Vertex::new(convert_vector(&segment.begin), Vec3::repeat(1.0));
        let end = Vertex::new(convert_vector(&segment.end), Vec3::repeat(1.0));

        vertices.push(begin);
        vertices.push(end);
    }


    let mesh = Mesh::new(PrimitiveType::Lines, mat, &BasicGeometry::from_vertices(&vertices));

    let mut rp = RenderParameters::identity();
    // ======================================

    while !window.should_close() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        mesh.render(&mut rp);

        let ui = imgui_glfw.frame(&mut window, &mut imgui);

        ImWindow::new(im_str!("Meow"))
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(&ui, || {
                ui.text(im_str!("Hello world!"));
                ui.text(im_str!("こんにちは世界！"));
                ui.text(im_str!("This...is...imgui-rs!"));
                ui.separator();
                let mouse_pos = ui.io().mouse_pos;
                ui.text(format!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos[0], mouse_pos[1]
                ));
            });

        imgui_glfw.draw(ui, &mut window);

        window.swap_buffers();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            imgui_glfw.handle_event(&mut imgui, &event);

            match event {
                glfw::WindowEvent::Key(glfw::Key::M, _, Action::Press, _) => {
                    show_menu = !show_menu;
                },
                _ => {},
            }
        }
    }
}
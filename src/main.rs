
use lsystems_core::*;
use lsystems_core::drawing::{DrawOperation};
use glfw::{Action, Key, Context};
use imgui::{Condition, Context as ImContext, Window as ImWindow, im_str};
use imgui_glfw_rs::glfw;
use imgui_glfw_rs::imgui;
use imgui_glfw_rs::ImguiGLFW;
use nalgebra_glm::{Mat4, Vec3};

use rendering::shaders::{Shader, ShaderType, Program};
use rendering::uniforms::*;
use rendering::buffers::*;

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

    let program = Program::from_source(vertexShaderSource, fragmentShaderSource).unwrap();

    let vbo = VertexBuffer::new(&vec![
        // Positions                // Colors
        Vec3::new(0.5, -0.5, 0.0),  Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(-0.5, -0.5, 0.0), Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.5, 0.0),   Vec3::new(0.0, 0.0, 1.0)
    ]);

    let vao = VertexArray::new(&vbo, 2);

    while !window.should_close() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        program.use_program();
        
        unsafe {
            vao.enable_array();
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            vao.disable_array();
        }

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



const vertexShaderSource: &str = r#"
    #version 330 core

    layout (location = 0) in vec3 Position;
    layout (location = 1) in vec3 Color;

    out VS_OUTPUT {
        vec3 Color;
    } OUT;

    void main()
    {
        gl_Position = vec4(Position, 1.0);
        OUT.Color = Color;
    }
"#;

const fragmentShaderSource: &str = r#"
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
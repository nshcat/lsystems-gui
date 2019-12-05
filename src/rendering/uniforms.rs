use nalgebra_glm::{Mat4, Vec3};
use gl::types::*;
use std::ffi::CString;
use std::ptr;
use crate::rendering::shaders::*;

/// Uniform handling
impl Program {
    /// Set 4x4 matrix uniform on this program object
    /// TODO: Taking adress of the matrix like this might not work!
    pub fn set_uniform_mat4(&self, name: &str, matrix: &Mat4) {
        let loc = self.query_location(name);

        unsafe {
            gl::UniformMatrix4fv(
                loc,
                1,
                gl::FALSE,
                matrix as *const Mat4 as *const _
            );
        }
    }

    /// Set Vec3 uniform on this program object
    pub fn set_uniform_vec3(&self, name: &str, vec: &Vec3) {
        let loc = self.query_location(name);

        unsafe {
            gl::Uniform3fv(
                loc,
                1,
                vec as *const Vec3 as *const _
            );
        }
    }

    /// Retrieve uniform location for given name string
    fn query_location(&self, name: &str) -> GLint {
        unsafe {
            let name_cstr = CString::new(name.as_bytes()).unwrap();
            let loc = gl::GetUniformLocation(self.handle, name_cstr.as_ptr());

            if loc == -1 {
                panic!("Could not find uniform location for uniform name \"{}\"", name);
            } else {
                loc
            }
        }
    }
}
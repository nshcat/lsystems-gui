use std::ffi::CString;
use std::ptr;
use std::fmt::Display;
use std::string::*;
use gl::types::*;
use crate::rendering::types::{GlHandle};

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum ShaderType {
    FragmentShader = gl::FRAGMENT_SHADER,
    VertexShader = gl::VERTEX_SHADER
}

impl Display for ShaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ShaderType::FragmentShader => write!(f, "fragment shader"),
            ShaderType::VertexShader => write!(f, "vertex shader")
        }
    }
}

/// A struct storing information about a failed program compilation or creation
#[derive(Debug, Clone)]
pub enum ProgramError {
    /// Some generic error not covered by the more specific cases, e.g. failure to create
    /// shader object.
    GenericError(&'static str),
    /// Compilation of shader source code failed
    ShaderCompilationError(ShaderType, String),
    /// Linking of program failed
    ProgramLinkError(String)
}

impl Display for ProgramError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            &ProgramError::GenericError(s) => write!(f, "Failed to create program/shader: {}", s),
            &ProgramError::ProgramLinkError(log) => write!(f, "Failed to link program: {}", log),
            &ProgramError::ShaderCompilationError(ty, log) => write!(f, "Failed to compile {}: {}", ty, log)
        }
    }
}

/// A struct encapsulating and managing an OpenGL shader object.
pub struct Shader {
    /// The handle to this shader object.
    handle: GlHandle
}

impl Shader {
    pub fn from_source(source: &str, ty: ShaderType) -> Result<Shader, ProgramError> {
        unsafe {
            // Create empty shader object
            let handle = gl::CreateShader(ty as GLenum);

            if handle == 0 {
                return Err(ProgramError::GenericError("Failed to create shader object"));
            }

            // Attach shader source
            let src_cstr = CString::new(source.as_bytes()).unwrap();
            gl::ShaderSource(handle, 1, [src_cstr.as_ptr()].as_ptr(), ptr::null());

            // Compile shader
            gl::CompileShader(handle);

            // Check result
            let mut compile_result: GLint = 0;
            gl::GetShaderiv(handle, gl::COMPILE_STATUS, &mut compile_result);

            if compile_result == gl::TRUE as GLint {
                return Ok(Shader {
                    handle: handle
                })
            } else {
                // Retrieve length of the info log
                let mut log_size: GLint = 0;
                gl::GetShaderiv(handle, gl::INFO_LOG_LENGTH, &mut log_size);

                let mut log: Vec<u8> = Vec::with_capacity(log_size as usize);
                gl::GetShaderInfoLog(handle, log_size, &mut log_size, log.as_mut_ptr() as *mut GLchar);
                log.set_len(log_size as usize);

                return match String::from_utf8(log) {
                    Ok(str) => Err(ProgramError::ShaderCompilationError(ty, str)),
                    Err(_) => Err(ProgramError::GenericError("Could not extract shader info log after failed compilation"))
                }
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.handle);
        }
    }
}

/// A struct representing an OpenGL program object, which consists of a fragment shader
/// and a vertex shader.
pub struct Program {
    /// The fragment shader instance
    fragment: Shader,
    /// The vertex shader instance
    vertex: Shader,
    /// The program object handle
    pub handle: GlHandle
}

/// Creation
impl Program {
    /// Create a new shader program from existing shader objects.
    pub fn from_shaders(vs: Shader, fs: Shader) -> Result<Program, ProgramError> {
        unsafe {
            let handle = gl::CreateProgram();

            if handle == 0 {
                return Err(ProgramError::GenericError("Failed to create program object"));
            }

            gl::AttachShader(handle, vs.handle);
            gl::AttachShader(handle, fs.handle);
            gl::LinkProgram(handle);

            let mut link_status: GLint = 0;
            gl::GetProgramiv(handle, gl::LINK_STATUS, &mut link_status);

            if link_status != gl::TRUE as GLint {
                // If the shader program failed to link, retrieve the info log for better error
                // reporting
                let mut log_size: GLint = 0;
                gl::GetProgramiv(handle, gl::INFO_LOG_LENGTH, &mut log_size);

                let mut log: Vec<u8> = Vec::with_capacity(log_size as usize);
                gl::GetProgramInfoLog(handle, log_size, &mut log_size, log.as_mut_ptr() as *mut GLchar);
                log.set_len(log_size as usize);


                let log_str = String::from_utf8(log);

                return match log_str {
                    Err(_) => Err(ProgramError::GenericError("Failed to retrieve info log after failed program linking")),
                    Ok(s) => Err(ProgramError::ProgramLinkError(s))
                }
            } else {
                return Ok(Program {
                    handle: handle,
                    fragment: fs,
                    vertex: vs
                })
            }
        }
    }

    /// Create a new shader program from vertex and fragment shader source code.
    pub fn from_source(vs_src: &str, fs_src: &str) -> Result<Program, ProgramError> {
        let vs = Shader::from_source(vs_src, ShaderType::VertexShader);

        match vs {
            Err(e) => return Err(e),
            _ => ()
        }

        let fs = Shader::from_source(fs_src, ShaderType::FragmentShader);

        match fs {
            Err(e) => return Err(e),
            _ => ()
        }

        Self::from_shaders(vs.unwrap(), fs.unwrap())
    }
}

/// Operations
impl Program {
    /// Activate this program object for the active OpenGL context.
    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.handle);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.handle);
        }
    }
}
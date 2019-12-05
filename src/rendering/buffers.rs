use gl::types::*;
use std::marker::PhantomData;
use crate::rendering::traits::*;
use crate::rendering::types::GlHandle;

/// A simple struct storing an error message regarding buffer creation and usage
pub struct BufferError(&'static str);

/// A struct encapsulating an OpenGL vertex buffer object (VBO)
pub struct VertexBuffer<T: GPUType>  {
    /// The handle to the VBO
    pub handle: GlHandle,
    /// Phantom data instance, which is needed since we are not really directly
    /// using the type T
    phantom: PhantomData<T>
}

impl<T: GPUType> VertexBuffer<T> {
    /// Create new VBO with data copied from given source buffer.
    pub fn new(data: &[T]) -> VertexBuffer<T> {
        let vbo = VertexBuffer::<T> {
            handle: Self::create_buffer(),
            phantom: PhantomData
        };

        vbo.fill_data(data);

        vbo
    }

    /// Create new buffer handle
    fn create_buffer() -> GlHandle {
        let mut handle: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut handle);
        }

        handle
    }

    /// Fill VBO data with given buffer
    fn fill_data(&self, data: &[T]) {
        unsafe {
            // Make sure the buffer is actually active
            gl::BindBuffer(gl::ARRAY_BUFFER, self.handle);

            gl::BufferData(
                gl::ARRAY_BUFFER,           // Target, in our case the currently active VBO
                data.raw_length(),          // The total length of the buffer data, in bytes
                data.to_buffer_raw_ptr(),   // Pointer to the data
                gl::STATIC_DRAW             // Usage hint for the driver
            );

            // Unbind buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    /// Bind this buffer to the array buffer target.
    pub fn enable_buffer(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.handle);
        }
    }

    /// Unbind this buffer fromthe array buffer target.
    pub fn disable_buffer(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}

/// A struct encapsulating an OpenGL vertex array object (VAO)
pub struct VertexArray {
    /// The handle to the VAO
    handle: GlHandle
}
use gl::types::*;
use std::marker::PhantomData;
use std::ptr::*;
use crate::rendering::traits::*;
use crate::rendering::types::GlHandle;

/// A simple struct storing an error message regarding buffer creation and usage
pub struct BufferError(&'static str);

/// A struct encapsulating an OpenGL vertex buffer object (VBO)
pub struct VertexBuffer<T: GPUType>  {
    /// The handle to the VBO
    handle: GlHandle,
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
pub struct VertexArray<T: GPUType> {
    /// The handle to the VAO
    handle: GlHandle,
    /// Phantom data instance, which is needed since we are not really directly
    /// using the type T
    phantom: PhantomData<T>
}

impl<T: GPUType> VertexArray<T> {
    fn create_array() -> GlHandle {
        let mut vao: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        }
        vao
    }

    /// Create a new vertex array object with interleaved attribute layout and given number of attributes
    pub fn new(vbo: &VertexBuffer<T>, num_attribs: usize) -> VertexArray<T> {
        let vao = VertexArray::<T>{
            handle: Self::create_array(),
            phantom: PhantomData
        };
        vao.activate_attributes(vbo, num_attribs);
        vao
    }

    /// Activate this vertex array object
    pub fn enable_array(&self) {
        unsafe {
            gl::BindVertexArray(self.handle);
        }
    }

    /// Deactivate this vertex array object
    pub fn disable_array(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }

    /// Activate a vertex attributes for all attributes.
    /// Note: The associated VBO is expected to store the data for the attributes
    /// in the order of the index. This is an additional restriction in comparison
    /// to general OpenGL, but for this project it simplifies the API a lot.
    fn activate_attributes(&self, vbo: &VertexBuffer<T>, num_attribs: usize) {
        self.enable_array();
        vbo.enable_buffer();

        unsafe {
            for i in 0..num_attribs {
                gl::EnableVertexAttribArray(i as _);
                gl::VertexAttribPointer(
                    i as _,
                    T::NUM_COMPONENTS as _,
                    T::ELEMENT_TYPE as _,
                    gl::FALSE,
                    (num_attribs * T::INSTANCE_SIZE) as _, // The stride, which is the length of one segment
                    (i * T::INSTANCE_SIZE) as _            // The offset into each segment
                );      
            }
        }

        vbo.disable_buffer();
        self.disable_array();
    }
}
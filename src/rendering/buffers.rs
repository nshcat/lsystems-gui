use gl::types::*;
use std::marker::PhantomData;
use std::ptr::*;
use crate::rendering::traits::*;
use crate::rendering::types::GlHandle;
use crate::rendering::meshes::*;

/// A simple struct storing an error message regarding buffer creation and usage
pub struct BufferError(&'static str);

/// A trait that allows code to manage buffers of different value types.
pub trait BufferBase {
    /// Bind this vertex buffer and enable it
    fn enable(&self);

    /// Disable and unbind this vertex buffer
    fn disable(&self);
}

/// Enumeration describing the different buffer types
enum BufferType {
    /// A vertex buffer object (VBO)
    VertexBuffer,
    /// An index buffer used for indexed rendering
    IndexBuffer
}

impl BufferType {
    /// Retrieve binding point constant for given buffer type
    pub fn binding_point(&self) -> GLuint {
        match *self {
            BufferType::VertexBuffer => gl::ARRAY_BUFFER,
            BufferType::IndexBuffer => gl::ELEMENT_ARRAY_BUFFER
        }
    }
}


/// A struct encapsulating an OpenGL vertex or index buffer object
pub struct Buffer<T: GPUType>  {
    /// The handle to the VBO
    handle: GlHandle,
    /// Phantom data instance, which is needed since we are not really directly
    /// using the type T
    phantom: PhantomData<T>,
    /// Type of this buffer
    buffer_type: BufferType
}

impl<T: GPUType> Buffer<T> {
    /// Create new VBO with data copied from given source buffer.
    pub fn new_vertex_buffer(data: &[T]) -> Buffer<T> {
        let vbo = Buffer::<T> {
            handle: Self::create_buffer(),
            phantom: PhantomData,
            buffer_type: BufferType::VertexBuffer
        };

        vbo.fill_data(data);

        vbo
    }
    
    /// Create new index buffer with given data
    pub fn new_index_buffer(data: &[T]) -> Buffer<T> {
        let vbo = Buffer::<T> {
            handle: Self::create_buffer(),
            phantom: PhantomData,
            buffer_type: BufferType::IndexBuffer
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
            gl::BindBuffer(self.buffer_type.binding_point(), self.handle);

            gl::BufferData(
                self.buffer_type.binding_point(),           // Target, in our case the currently active VBO
                data.raw_length(),          // The total length of the buffer data, in bytes
                data.to_buffer_raw_ptr(),   // Pointer to the data
                gl::STATIC_DRAW             // Usage hint for the driver
            );

            // Unbind buffer
            gl::BindBuffer(self.buffer_type.binding_point(), 0);
        }
    }

    /// Bind this buffer to the array buffer target.
    pub fn enable_buffer(&self) {
        unsafe {
            gl::BindBuffer(self.buffer_type.binding_point(), self.handle);
        }
    }

    /// Unbind this buffer fromthe array buffer target.
    pub fn disable_buffer(&self) {
        unsafe {
            gl::BindBuffer(self.buffer_type.binding_point(), 0);
        }
    }
}

impl<T> BufferBase for Buffer<T> where T: GPUType {
    fn enable(&self) {
        self.enable_buffer();
    }

    fn disable(&self) {
        self.disable_buffer();
    }
}

/// A struct encapsulating an OpenGL vertex array object (VAO)
pub struct VertexArray {
    /// The handle to the VAO
    handle: GlHandle
}

impl VertexArray {
    pub fn new() -> VertexArray {
        let mut handle: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut handle);
        }
        
        VertexArray{
            handle: handle
        }
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

    /// Activate a vertex attribute for the currently active VBO.
    /// NOTE: The VBO must already be bound!
    pub fn activate_attribute<T: GPUType>(&self, descriptor: &AttributeDescriptor) {
        self.enable_array();

        unsafe {
            gl::EnableVertexAttribArray(descriptor.index as _);
            gl::VertexAttribPointer(
                descriptor.index as _,
                T::NUM_COMPONENTS as _,
                T::ELEMENT_TYPE as _,
                gl::FALSE,
                0 as _,           // The stride, which is the length of one segment
                0 as _            // The offset into each segment
            );                 
        }

        self.disable_array();
    }
}
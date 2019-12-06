use crate::rendering::{ValueType, RenderParameters};
use gl::types::*;
use std::mem::*;
use nalgebra_glm::{Vec3};

/// A trait for objects that can be rendered to screen
pub trait Render {
    /// Render object to screen with given rendering parameters.
    fn render(&self, params: &mut RenderParameters);
}

/// A trait for types that can be uploaded to GPU buffers. In many cases, these are types that
/// contain multiple components of a base type, for example, a three-dimensional vector of GL_FLOAT
/// elements (or, in glm terms, a Vec3).
pub trait GPUType {
    /// The size of a single component, in bytes.
    const ELEMENT_SIZE: usize;

    /// The size of a single instance of the type, in bytes. This includes all the components!
    /// This is used to calculate the stride.
    const INSTANCE_SIZE: usize;

    /// How many components are in a single instance of the type. For example,
    /// a Vec3 contains 3 elements.
    const NUM_COMPONENTS: usize;

    /// The OpenGL value type of elements in type. For example, a Vec3 contains floats.
    const ELEMENT_TYPE: ValueType;

    /// Retrieve pointer to memory for an instance of this type.
    unsafe fn to_element_raw_ptr(&self) -> *const GLvoid;
}

/// A trait for sequences of GPU types that can be uploaded to a GPU buffer.
pub trait GPUBufferSource {
    /// Retrieve length of buffer contents, in bytes
    fn raw_length(&self) -> GLsizeiptr;

     /// Retrieve pointer to memory for a sequence of gpu types.
     unsafe fn to_buffer_raw_ptr(&self) -> *const GLvoid;
}

/// Implement GPUBufferSource trait for all slices of GPUTypes
impl<T> GPUBufferSource for &[T] where T: GPUType {
    unsafe fn to_buffer_raw_ptr(&self) -> *const GLvoid {
        (*self).as_ptr() as *const _
    }

    fn raw_length(&self) -> GLsizeiptr {
        (T::INSTANCE_SIZE * self.len()) as _
    }
}

impl GPUType for Vec3 {
    /// The size of a single component, in bytes.
    const ELEMENT_SIZE: usize = std::mem::size_of::<f32>();

    /// The size of a single instance of the type, in bytes. This includes all the components!
    /// This is used to calculate the stride.
    const INSTANCE_SIZE: usize = 3 * Self::ELEMENT_SIZE;

    /// How many components are in a single instance of the type. For example,
    /// a Vec3 contains 3 elements.
    const NUM_COMPONENTS: usize = 3;

    /// The OpenGL value type of elements in type. For example, a Vec3 contains floats.
    const ELEMENT_TYPE: ValueType = ValueType::Float;

    /// Retrieve pointer to memory for an instance of this type.
    unsafe fn to_element_raw_ptr(&self) -> *const GLvoid {
        self as *const Vec3 as *const _
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UniformBufferObject {
    pub time: f32,
    pub width: f32,
    pub height: f32,
    // pub resolution: nalgebra_glm::Vec2
}

// #[repr(C)]
// #[derive(Copy, Clone, Debug)]
// pub struct Vec2 {
//     x: f32,
//     y: f32
// }
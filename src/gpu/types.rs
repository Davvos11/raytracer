use crate::camera::Camera;
#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraData {
    screen_size: [u32; 2],
    center: [f32; 3],
    pixel00_loc: [f32; 3],
    pixel_delta_u: [f32; 3],
    pixel_delta_v: [f32; 3],
    padding: [u32; 6],
}

impl From<&Camera> for CameraData {
    fn from(cam: &Camera) -> Self {
        Self {
            screen_size: [cam.image_width, cam.image_height()],
            center: cam.center.into(), // TODO correct?
            pixel00_loc: cam.pixel00_loc.into(),
            pixel_delta_u: cam.pixel_delta_u.into(),
            pixel_delta_v: cam.pixel_delta_v.into(),
            padding: [0; 6],
        }
    }
}

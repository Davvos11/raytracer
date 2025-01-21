use crate::camera::Camera;
use crate::hittable::sphere::Sphere;
use crate::hittable::triangle::Triangle;

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
            padding: Default::default(),
        }
    }
}


#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TriangleData {
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
    padding: [u32; 3],
}


impl From<&Triangle> for TriangleData {
    fn from(value: &Triangle) -> Self {
        Self {
            v0: value.a().into(),
            v1: value.b().into(),
            v2: value.c().into(),
            padding: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SphereData {
    center: [f32; 3],
    radius: f32,
}

impl From<&Sphere> for SphereData {
    fn from(value: &Sphere) -> Self {
        Self {
            center: value.center().into(),
            radius: value.radius() as f32,
        }
    }
}

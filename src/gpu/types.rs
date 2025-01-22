use crate::camera::Camera;
use crate::hittable::sphere::Sphere;
use crate::hittable::triangle::Triangle;
use crate::hittable::Hittable;
use crate::value::color;
use crate::value::material::MaterialType;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraData {
    screen_size: [u32; 2],
    _0: [u32; 2], // Padding
    center: [f32; 3],
    _1: [u32; 1], // Padding
    pixel00_loc: [f32; 3],
    _2: [u32; 1], // Padding
    pixel_delta_u: [f32; 3],
    _3: [u32; 1], // Padding
    pixel_delta_v: [f32; 3],
    _4: [u32; 1], // Padding
}

impl From<&Camera> for CameraData {
    fn from(cam: &Camera) -> Self {
        Self {
            screen_size: [cam.image_width, cam.image_height()],
            center: cam.center.into(), // TODO correct?
            pixel00_loc: cam.pixel00_loc.into(),
            pixel_delta_u: cam.pixel_delta_u.into(),
            pixel_delta_v: cam.pixel_delta_v.into(),
            ..Default::default()
        }
    }
}




#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TriangleData {
    v0: [f32; 3],
    _0: [u32; 1], // Padding
    v1: [f32; 3],
    refraction_index: f32,
    v2: [f32; 3],
    material: u32,
    color: [f32; 3],
    fuzz: f32,
}


impl From<&Triangle> for TriangleData {
    fn from(value: &Triangle) -> Self {
        let color: [f32; 3];
        let material: u32;
        let fuzz: f32;
        let refraction_index;

        if let Some(material_type) = value.material_type() {
            match material_type {
                MaterialType::Lambertian => {
                    color = value.mat().albedo().into();
                    material = 0;
                    fuzz = 0.0;
                    refraction_index = f32::default();
                }
                MaterialType::Metal => {
                    color = value.mat().albedo().into();
                    material = 1;
                    fuzz = value.mat().fuzz() as f32;
                    refraction_index = f32::default();
                }
                MaterialType::Dielectric => {
                    color = color::Color::default().into();
                    material = 2;
                    fuzz = f32::default();
                    refraction_index = value.mat().refraction_index() as f32;
                }
            }
            
            return Self {
                v0: value.a().into(),
                v1: value.b().into(),
                v2: value.c().into(),
                color,
                material,
                fuzz,
                refraction_index,
                ..Default::default()
            }
        }

        Self {
            v0: value.a().into(),
            v1: value.b().into(),
            v2: value.c().into(),
            ..Default::default()
        }
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SphereData {
    center: [f32; 3],
    radius: f32,
    color: [f32; 3],
    material: u32,
    fuzz: f32,
    refraction_index: f32
}

impl From<&Sphere> for SphereData {
    
    fn from(value: &Sphere) -> Self {
        let color: [f32; 3];
        let material: u32;
        let fuzz: f32;
        let refraction_index: f32;
        
        if let Some(material_type) = value.material_type() {
            match material_type {
                MaterialType::Lambertian => {
                    color = value.mat().albedo().into();
                    material = 0;
                    fuzz = f32::default();
                    refraction_index = f32::default();
                }
                MaterialType::Metal => {
                    color = value.mat().albedo().into();
                    material = 1;
                    fuzz = value.mat().fuzz() as f32;
                    refraction_index = f32::default();
                }
                MaterialType::Dielectric => {
                    color = color::Color::default().into();
                    material = 2;
                    fuzz = f32::default();
                    refraction_index = f32::default();
                }
            }
            

            return Self {
                center: value.center().into(),
                radius: value.radius() as f32,
                color,
                material,
                fuzz,
                refraction_index
            }
        }

        Self {
            center: value.center().into(),
            radius: value.radius() as f32,
            ..Default::default()
        }
    }
}

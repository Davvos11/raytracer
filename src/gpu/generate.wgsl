struct CameraData {
    screenSize: vec2<f32>,
    center: vec3<f32>,
    pixel00_loc: vec3<f32>,
    pixel_delta_u: vec3<f32>,
    pixel_delta_v: vec3<f32>,
};

@group(0) @binding(0) var<uniform> cameraData: CameraData;

// output
struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
};

@group(0) @binding(1) var<storage, read_write> rayBuffer: array<Ray>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x < u32(cameraData.screenSize.x) && y < u32(cameraData.screenSize.y)) {
        let index = y * cameraData.screenSize.x + x;
        
        let origin = cameraData.center;
        let pixel_sample = cameraData.pixel00_loc + (x * cameraData.pixel_delta_u) + (y * cameraData.pixel_delta_v);
        
        let ray_direction = pixel_sample - origin;
        
        rayBuffer[index] = Ray(origin, ray_direction);
    }
}
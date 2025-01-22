struct ScreenData {
    x: u32,
    y: u32
}

@group(0) @binding(0) var<uniform> screenData: ScreenData;


struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
    t: f32,
    primIdx: u32,
    screenXy: vec2<u32>,
    accumulator: vec3<f32>,
    depth: u32
}

@group(0) @binding(1) var<storage, read_write> rayBuffer: array<Ray>;

@group(0) @binding(7) var<storage, read_write> pixelBuffer: array<vec4<f32>>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x < screenData.x && y < screenData.y) {
        let index = y * screenData.x + x;
        let ray = rayBuffer[index];
        
        let pixel_x = ray.screenXy.x;
        let pixel_y = ray.screenXy.y;
        let pixel_index = pixel_y * screenData.x + pixel_x;
        
        pixelBuffer[pixel_index] = vec4(ray.accumulator, 1.0);
    }
}

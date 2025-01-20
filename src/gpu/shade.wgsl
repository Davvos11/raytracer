struct ScreenData {
    x: f32,
    y: f32
}

struct Uniform {
    screenData: ScreenData
}

@group(0) @binding(0) var<uniform> uniformData: Uniform;

struct HitRecord {
    p: vec3<f32>,
    normal: vec3<f32>,
    t: f32,
    front_face: bool,
}

@group(0) @binding(2) var<storage, read_write> hitRecordBuffer: array<HitRecord>;

fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
        
    if (x < u32(&uniformData.screenData.x) && y < u32(&uniformData.screenData.y)) {
        let index = y * &uniformData.screenData.x + x;
        let hitRecord = &hitRecordBuffer[index];
        if (hit_anything(hitRecord.p)) {
            
        }
        
    }
}

fn hit_anything(p: vec3<f32>) -> bool {
    return p.x != 0.0 && p.y != 0.0 && p.z != 0.0;
}
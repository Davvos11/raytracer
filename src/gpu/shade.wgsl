struct TriangleData {
    v0: vec3<f32>,
    v1: vec3<f32>,
    v2: vec3<f32>,
    tCentroid: vec3<f32> // centroid is a reserved keyword
}

@group(0) @binding(2) var<storage, read> triangleData: array<TriangleData>;

struct SphereData {
    center: vec3<f32>,
    radius: f32
}

@group(0) @binding(3) var<storage, read> sphereData: array<SphereData>;

struct ScreenData {
    x: f32,
    y: f32
}

@group(0) @binding(0) var<uniform> screenData: ScreenData;

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
    t: f32,
    primIdx: u32,
    screenXy: vec2<u32>
}

@group(0) @binding(1) var<storage, read_write> rayBuffer: array<Ray>;
@group(0) @binding(4) var<storage, read_write> shadowRayBuffer: array<Ray>;
@group(0) @binding(5) var<storage, read_write> reflectionRayBuffer: array<Ray>;

var<workgroup> shadowRayIdx: atomic<u32>;
var<workgroup> extensionRayIdx: atomic<u32>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
        
    if (x < u32(screenData.x) && y < u32(screenData.y)) {
        let index = y * u32(screenData.x) + x;
        let ray = rayBuffer[index];

        if (ray.t <= 0.0) {
            return;
        }

        let intersectionPoint = ray_at(ray);
        let sphereLength = arrayLength(&sphereData);
        let primIdx = ray.primIdx;

        if (primIdx < sphereLength) {
            let currentSphere = sphereData[primIdx];

            if (true) { // todo: shadow or reflection
                let shadowIndex = atomicAdd(&shadowRayIdx, 1u) + 1u;
                
            } else {
                let extensionIndex = atomicAdd(&extensionRayIdx, 1u) + 1u;
            }
        } else {
            let currentTriangle = triangleData[primIdx - sphereLength];

            if (true) { // todo: shadow or reflection
                let shadowIndex = atomicAdd(&shadowRayIdx, 1u) + 1u;
            } else {
                let extensionIndex = atomicAdd(&extensionRayIdx, 1u) + 1u;
            }
        }
    }
}

fn shadow_ray(intersectionPoint: vec3<f32>) {

}

fn vec3_add(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(a.x + b.x, a.y + b.y, a.z + b.z);
}

fn vec3_multiply(a: vec3<f32>, b: f32) -> vec3<f32> {
    return vec3(a.x * b, a.y * b, a.z * b);
}

fn hit_anything(p: vec3<f32>) -> bool {
    return p.x != 0.0 && p.y != 0.0 && p.z != 0.0;
}

fn ray_at(r: Ray) -> vec3<f32> {
    return vec3_add(r.origin, vec3_multiply(r.direction, r.t));
}
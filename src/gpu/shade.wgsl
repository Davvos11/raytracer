struct TriangleData {
    v0: vec3<f32>,
    v1: vec3<f32>,
    v2: vec3<f32>,
}

@group(0) @binding(2) var<storage, read> triangleData: array<TriangleData>;

struct SphereData {
    center: vec3<f32>,
    radius: f32
}

@group(0) @binding(3) var<storage, read> sphereData: array<SphereData>;

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
    screenXy: vec2<u32>
}

@group(0) @binding(1) var<storage, read_write> rayBuffer: array<Ray>;
@group(0) @binding(4) var<storage, read_write> shadowRayBuffer: array<Ray>;
@group(0) @binding(5) var<storage, read_write> reflectionRayBuffer: array<Ray>;

@group(0) @binding(6) var<storage, read> randomUnit: array<vec3<f32>>;

@group(0) @binding(99) var<storage, read_write> debugData: array<Ray>;

var<workgroup> shadowRayIdx: atomic<u32>;
var<workgroup> extensionRayIdx: atomic<u32>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
        
    if (x < screenData.x && y < screenData.y) {
        let index = y * screenData.x + x;
        let ray = rayBuffer[index];
        
        if (ray.t <= 0.0) {
            return;
        }

        let intersectionPoint = ray_at(ray);
        let sphereLength = arrayLength(&sphereData);
        let primIdx = ray.primIdx;
        let random_unit = randomUnit[index];

        if (primIdx < sphereLength) {
            let currentSphere = sphereData[primIdx];
            let outward_normal = vec3_divide(vec3_subtract(intersectionPoint, currentSphere.center), currentSphere.radius);
            let front_face = vec3_dot(ray.direction, outward_normal) < 0.0;
            var normal = outward_normal;
            if (!front_face) {
                normal = vec3_multiply(normal, -1.0);
            }
            
            if (true) { // todo: shadow or reflection
                // lambertian - shadow ray
                let shadowIndex = atomicAdd(&shadowRayIdx, 1u);
                shadowRayBuffer[shadowIndex] = shadow_ray(normal, random_unit, intersectionPoint, ray.screenXy);
            } else {
                // metal - reflection ray
                let extensionIndex = atomicAdd(&extensionRayIdx, 1u);
                reflectionRayBuffer[extensionIndex] = reflect_ray(ray, intersectionPoint, random_unit, normal);
            }
        } else {
            let currentTriangle = triangleData[primIdx - sphereLength];
            
            let v0v1 = vec3_subtract(currentTriangle.v1, currentTriangle.v0);
            let v0v2 = vec3_subtract(currentTriangle.v2, currentTriangle.v0);
            let n = vec3_cross(v0v1, v0v2);
            
            let front_face = vec3_dot(ray.direction, n) < 0.0;
            var normal = n;
            if (!front_face) {
                normal = vec3_multiply(normal, -1.0);
            }

            if (true) { // todo: shadow or reflection
                // lambertian - shadow ray
                let shadowIndex = atomicAdd(&shadowRayIdx, 1u);
                shadowRayBuffer[shadowIndex] = shadow_ray(normal, random_unit, intersectionPoint, ray.screenXy);
            } else {
                // metal - reflection ray
                let extensionIndex = atomicAdd(&extensionRayIdx, 1u);
                reflectionRayBuffer[extensionIndex] = reflect_ray(ray, intersectionPoint, random_unit, normal);
            }
        }
    }
}

fn shadow_ray(normal: vec3<f32>, random_unit: vec3<f32>, p: vec3<f32>, screenXy: vec2<u32>) -> Ray {
    let scatter_direction = vec3_add(normal, random_unit);
    return Ray(p, scatter_direction, 0.0, 0u, screenXy);
}

fn reflect_ray(ray: Ray, p: vec3<f32>, random_unit: vec3<f32>, normal: vec3<f32>) -> Ray {
    var reflected = vec3_reflect(ray.direction, normal);
    reflected = vec3_add(vec3_unit(reflected), random_unit);
    
    return Ray(p, reflected, 0.0, 0u, ray.screenXy);
}

fn vec3_add(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(a.x + b.x, a.y + b.y, a.z + b.z);
}

fn vec3_multiply(a: vec3<f32>, b: f32) -> vec3<f32> {
    return vec3(a.x * b, a.y * b, a.z * b);
}

fn vec3_subtract(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(a.x - b.x, a.y - b.y, a.z - b.z);
}

fn vec3_cross(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x);
}

fn vec3_divide(a: vec3<f32>, b: f32) -> vec3<f32> {
    return vec3(a.x / b, a.y / b, a.z / b);
}

fn vec3_dot(a: vec3<f32>, b: vec3<f32>) -> f32 {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}

fn vec3_reflect(v: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    let dot = vec3_dot(v, n);
    return vec3_subtract(v, vec3_multiply(n, 2.0 * dot));
}

fn vec3_length(v: vec3<f32>) -> f32 {
    return sqrt(v.x * v.x + v.y * v.y + v.z * v.z);
}

fn vec3_unit(v: vec3<f32>) -> vec3<f32> {
    return vec3_divide(v, vec3_length(v));
}

fn hit_anything(p: vec3<f32>) -> bool {
    return p.x != 0.0 && p.y != 0.0 && p.z != 0.0;
}

fn ray_at(r: Ray) -> vec3<f32> {
    return vec3_add(r.origin, vec3_multiply(r.direction, r.t));
}
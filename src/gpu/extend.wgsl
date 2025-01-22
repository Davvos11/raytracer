struct TriangleData {
    v0: vec3<f32>,
    v1: vec3<f32>,
    refraction_index: f32,
    v2: vec3<f32>,
    material: u32,
    color: vec3<f32>,
    fuzz: f32,
}

@group(0) @binding(2) var<storage, read> triangleData: array<TriangleData>;

struct SphereData {
    center: vec3<f32>,
    radius: f32,
    color: vec3<f32>,
    material: u32,
    fuzz: f32,
    refraction_index: f32
}

@group(0) @binding(3) var<storage, read> sphereData: array<SphereData>;
@group(0) @binding(99) var<storage, read_write> debugData: array<u32>;

// Binding 0 is also used for camera data in generate.wgsl
// This works because both structs start with the screen size as two u32 values
struct ScreenData {
    x: u32,
    y: u32,
}

@group(0) @binding(0) var<uniform> screenData: ScreenData;

struct Interval {
    min: f32,
    max: f32
}

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

@group(0) @binding(9) var<storage, read_write> isFinished: u32;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;

    if (x < screenData.x && y < screenData.y) {
        let index = y * screenData.x + x;

        var ray = rayBuffer[index];
        
        if (ray.depth == 0u) {
            return;
        }
        // Set isFinished to false (is initialised at 1 every loop)
        isFinished = 0u;
        
        var ray_t = Interval(0.001, 10000000); // todo: find a way in wgsl to get max f32 value (0x1.fffffcp-127f ??)
        
        var hit_anything = false;
        
        for (var i = 0u; i < arrayLength(&triangleData); i++) {
            let currentTriangle = triangleData[i];
            if (hit_triangle(ray, index, i, currentTriangle, ray_t)) {
                hit_anything = true;
                ray = rayBuffer[index];
                ray_t = Interval(0.001, ray.t);
            }
        }
        
        for (var i = 0u; i < arrayLength(&sphereData); i++) {
            let currentSphere = sphereData[i];
            if (hit_sphere(ray, index, i, currentSphere, ray_t)) {
                hit_anything = true;
                ray = rayBuffer[index];
                ray_t = Interval(0.001, ray.t);
            }
        }
        
        if (!hit_anything) {
            rayBuffer[index] = Ray(ray.origin, ray.direction, ray.t, ray.primIdx, ray.screenXy, ray.accumulator, 0u);
        }
    }
}

fn hit_sphere(ray: Ray, ray_index: u32, primId: u32, sphere: SphereData, ray_t: Interval) -> bool {
    let oc = vec3_subtract(sphere.center, ray.origin);
    let a = ray.direction.x * ray.direction.x + ray.direction.y * ray.direction.y + ray.direction.z * ray.direction.z;
    let h = ray.direction.x * oc.x + ray.direction.y * oc.y + ray.direction.z * oc.z;
    let c = (oc.x * oc.x + oc.y * oc.y + oc.z * oc.z) - sphere.radius * sphere.radius;
    
    let discriminant = h * h - a * c;
    if discriminant < 0.0 {
        return false;
    }
    
    let sqrt_discriminant = sqrt(discriminant);
    
    var root = (h - sqrt_discriminant) / a;
    
    if (!interval_surrounds(ray_t, root)) {
        root = (h + sqrt_discriminant) / a;
        if (!interval_surrounds(ray_t, root)) {
            return false;
        }
    }
    
    let hit_t = root;
    // todo: mats?
    // seems like wgsl supports pointers but it might be easier to just use return types
    
    rayBuffer[ray_index] = Ray(ray.origin, ray.direction, hit_t, primId, ray.screenXy, ray.accumulator, ray.depth);


    return true;
}

fn hit_triangle (r: Ray, ray_index: u32, primId: u32, t: TriangleData, ray_t: Interval) -> bool {
    // Calculate the normal by the cross product of AB and AC
    let v0v1 = vec3_subtract(t.v1, t.v0); // AB
    let v0v2 = vec3_subtract(t.v2, t.v0); // AC
    let n = vec3_cross(v0v1, v0v2);
    
    // Check if the ray and plane are parallel
    let n_dot_dir = vec3_dot(n, r.direction);
    if (!interval_surrounds(ray_t, n_dot_dir)) {
        return false;
    }
    
    // Get the distance from the origin to the plane
    let d = -1 * vec3_dot(n, t.v0);
    // Get the distance along the ray
    let hit_t = -1 * ((vec3_dot(n, r.origin) + d) / n_dot_dir);
    
    // The triangle is not visible if it is behind the camera
    if (hit_t < 0.0) {
        return false;
    }
    
    // Get the intersection point
    let p = ray_at(r, hit_t);
    // Check if the plane intersection is inside the triangle
    // (inside-outside test)
    let v0p = vec3_subtract(p, t.v0);
    if (vec3_dot(n, vec3_cross(v0v1, v0p)) <= 0.0) {
        return false;
    }
    
    let v1v2 = vec3_subtract(t.v2, t.v1);
    let v1p = vec3_subtract(p, t.v1);
    if (vec3_dot(n, vec3_cross(v1v2, v1p)) <= 0.0) {
        return false;
    }
    
    let v2v0 = vec3_subtract(t.v0, t.v2);
    let v2p = vec3_subtract(p, t.v2);
    if (vec3_dot(n, vec3_cross(v2v0, v2p)) <= 0.0) {
        return false;
    }

    //todo: mat
    
    // Hit! Replace the ray in the buffer
    rayBuffer[ray_index] = Ray(r.origin, r.direction, hit_t, primId + arrayLength(&sphereData), r.screenXy, r.accumulator, r.depth);
    
    return true;
    
}

fn vec3_add(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(a.x + b.x, a.y + b.y, a.z + b.z);
}

fn vec3_subtract(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(a.x - b.x, a.y - b.y, a.z - b.z);
}

fn vec3_cross(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x);
}

fn vec3_dot(a: vec3<f32>, b: vec3<f32>) -> f32 {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}

fn vec3_multiply(a: vec3<f32>, b: f32) -> vec3<f32> {
    return vec3(a.x * b, a.y * b, a.z * b);
}



fn vec3_divide(a: vec3<f32>, b: f32) -> vec3<f32> {
    return vec3(a.x / b, a.y / b, a.z / b);
}

fn interval_surrounds(a: Interval, x: f32) -> bool {
    return a.min < x && x < a.max;
}

fn ray_at(r: Ray, t: f32) -> vec3<f32> {
    return vec3_add(r.origin, vec3_multiply(r.direction, t));
}

//fn hr_set_face_normal(h: HitRecord, r: Ray, outward_normal: vec3<f32>) {
//    *h.front_face = vec3_dot(r.direction, outward_normal) < 0.0;
//    if (*h.front_face) {
//        *h.normal = outward_normal;
//    } else {
//        *h.normal = -1 * outward_normal;
//    }
//}
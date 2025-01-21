struct TriangleData {
    v0: vec3<f32>,
    v1: vec3<f32>,
    v2: vec3<f32>,
    material: u32,
    color: vec3<f32>,
    fuzz: f32
}

@group(0) @binding(2) var<storage, read> triangleData: array<TriangleData>;

struct SphereData {
    center: vec3<f32>,
    radius: f32,
    color: vec3<f32>,
    material: u32,
    fuzz: f32
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

struct Interval {
    min: f32,
    max: f32
}

@group(0) @binding(4) var<storage, read_write> shadowRayBuffer: array<Ray>;

@group(0) @binding(7) var<storage, read_write> pixelBuffer: array<vec3<f32>>;

@compute @workgroup_size(16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    
    if (index < arrayLength(&shadowRayBuffer)) {
        let shadowRay = shadowRayBuffer[index];
        
        for (var i = 0u; i < arrayLength(&sphereData); i++) {
            // sphere
            let currentSphere = sphereData[i];
            if (hit_sphere(shadowRay, currentSphere, Interval(0.001, 100000000))) {
                // color
            }
        }
        for (var i = 0u; i < arrayLength(&triangleData); i++) {
            // triangle
            let currentTriangle = triangleData[i];
            if (hit_triangle(shadowRay, currentTriangle, Interval(0.001, 100000000))) {
               // color
            }
        }
    }
}

fn hit_sphere(ray: Ray, sphere: SphereData, ray_t: Interval) -> bool {
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
    return true;
}

fn hit_triangle (r: Ray, t: TriangleData, ray_t: Interval) -> bool {
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
struct TriangleData {
    v0: vec3<f32>,
    v1: vec3<f32>,
    v2: vec3<f32>,
    tCentroid: vec3<f32> // centroid is a reserved keyword
}

struct SphereData {
    center: vec3<f32>,
    radius: f32
}

struct Uniform {
    triangles: array<TriangleData>,
    spheres: array<SphereData>
}

@group(0) @binding(0) var<uniform> uniformData: Uniform;

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

@group(0) @binding(1) var<storage, read> rayBuffer: array<Ray>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    
}

fn hit_sphere(ray: Ray, sphere: SphereData) -> bool {
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
    // todo: find a way to do the intervals and to return the intersection points
    // seems like wgsl supports pointers but it might be easier to just use return types
    
    return true;
}

fn hit_triangle (r: Ray, t: TriangleData) -> bool {
    // Calculate the normal by the cross product of AB and AC
    let v0v1 = vec3_subtract(t.v1, t.v0); // AB
    let v0v2 = vec3_subtract(t.v2, t.v0); // AC
    let n = vec3_cross(v0v1, v0v2);
    
    let n_dot_dir = vec3_dot(n, r.direction);
    // todo: interval
    
    let d = -1 * vec3_dot(n, t.v0);
    // todo: rec
    
    
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
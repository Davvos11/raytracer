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

struct ScreenData {
    x: f32,
    y: f32
}

struct Uniform {
    triangles: array<TriangleData>,
    spheres: array<SphereData>,
    screenData: ScreenData
}

@group(0) @binding(0) var<uniform> uniformData: Uniform;

struct Interval {
    min: f32,
    max: f32
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
    originPixel: vec3<f32>
}

struct HitRecord {
    p: vec3<f32>,
    normal: vec3<f32>,
    t: f32,
    front_face: bool,
}

@group(0) @binding(1) var<storage, read> rayBuffer: array<Ray>;

@group(0) @binding(2) var<storage, read_write> hitRecordBUffer: array<HitRecord>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
        
    if (x < u32(&uniformData.screenData.x) && y < u32(&uniformData.screenData.y)) {
        let index = y * &uniformData.screenData.x + x;
        
        let rec = HitRecord();
        let ray_t = Interval(0.001, 10000000); // todo: find a way in wgsl to get max f32 value
        
        for (var i = 0u; i < arrayLength(&uniformData.triangles); i++) {
            
        }
    }
}

fn hit_sphere(ray: Ray, sphere: SphereData, rec: HitRecord, ray_t: Interval) -> bool {
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
    
    *rec.t = root;
    *rec.p = ray_at(ray, root);
    let outward_normal = (vec3_divide(vec3_subtract(*rec.p, sphere.center), sphere.radius));
    hr_set_face_normal(rec, ray, outward_normal);
    // todo: mats?
    // seems like wgsl supports pointers but it might be easier to just use return types
    
    return true;
}

fn hit_triangle (r: Ray, t: TriangleData, rec: HitRecord, ray_t: Interval) -> bool {
    // Calculate the normal by the cross product of AB and AC
    let v0v1 = vec3_subtract(t.v1, t.v0); // AB
    let v0v2 = vec3_subtract(t.v2, t.v0); // AC
    let n = vec3_cross(v0v1, v0v2);
    
    let n_dot_dir = vec3_dot(n, r.direction);
    if (!interval_surrounds(ray_t, n_dot_dir)) {
        return false;
    }
    
    let d = -1 * vec3_dot(n, t.v0);
    *rec.t = -1 * (vec3_divide(vec3_dot(n, vec3_add(r.origin, d)), n_dot_dir));
    
    if (*rec.t < 0.0) {
        return false;
    }
    
    *rec.p = ray_at(r, rec.t);
    
    let v0p = vec3_subtract(*rec.p, t.v0);
    if (vec3_dot(n, vec3_cross(v0v1, v0p)) <= 0.0) {
        return false;
    }
    
    let v1v2 = vec3_subtract(t.v2, t.v1);
    let v0p = vec3_subtract(*rec.p, t.v1);
    if (vec3_dot(n, vec3_cross(v1v2, v1p)) <= 0.0) {
        return false;
    }
    
    let v2v0 = vec3_subtract(t.v0, t.v2);
    let v2p = vec3_subtract(*rec.p, t.v2);
    if (vec3_dot(n, vec3_cross(v2v0, v2p)) <= 0.0) {
        return false;
    }
    
    hr_set_face_normal(rec, r, n);
    //todo: mat
    
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
    return vec3_add(r.origin, vec3_multiply(t, r.direction));
}

fn hr_set_face_normal(h: HitRecord, r: Ray, outward_normal: vec3<f32>) {
    *h.front_face = vec3_dot(r.direction, outward_normal) < 0.0;
    if (*h.front_face) {
        *h.normal = outward_normal;
    } else {
        *h.normal = -1 * outward_normal;
    }
}
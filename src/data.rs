#[derive(Default)]
pub struct Data {
    seconds: f64,
    primary_rays: i32,
    scatter_rays: i32,
    intersection_checks: i32
}

impl Data {
    pub fn new() -> Data {
        Data { seconds: 0.0, primary_rays: 0, scatter_rays: 0, intersection_checks: 0 }
    }

    pub fn seconds(&self) -> f64 {
        self.seconds
    }
    
    pub fn set_seconds(&mut self, seconds: f64) {
        self.seconds = seconds;
    }

    pub fn primary_rays(&self) -> i32 {
        self.primary_rays
    }

    pub fn add_primary_ray(&mut self) {
        self.primary_rays += 1;
    }

    pub fn scatter_rays(&self) -> i32 {
        self.scatter_rays
    }

    pub fn add_scatter_ray(&mut self) {
        self.scatter_rays += 1;
    }

    pub fn intersection_checks(&self) -> i32 {
        self.intersection_checks
    }

    pub fn add_intersection_check(&mut self) {
        self.intersection_checks += 1;
    }
}
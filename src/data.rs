#[derive(Default)]
pub struct Data {
    seconds: f64,
    primary_rays: usize,
    scatter_rays: usize,
    intersection_checks: usize,
    overlapping_aabb: usize,
    gridbox_intersection_checks: usize
}

impl Data {
    pub fn new() -> Data {
        Data::default()
    }

    pub fn seconds(&self) -> f64 {
        self.seconds
    }
    
    pub fn set_seconds(&mut self, seconds: f64) {
        self.seconds = seconds;
    }

    pub fn primary_rays(&self) -> usize {
        self.primary_rays
    }

    pub fn add_primary_ray(&mut self) {
        self.primary_rays += 1;
    }

    pub fn scatter_rays(&self) -> usize {
        self.scatter_rays
    }

    pub fn add_scatter_ray(&mut self) {
        self.scatter_rays += 1;
    }

    pub fn intersection_checks(&self) -> usize {
        self.intersection_checks
    }

    pub fn add_intersection_check(&mut self) {
        self.intersection_checks += 1;
    }

    pub fn overlapping_aabb(&self) -> usize {
        self.overlapping_aabb
    }

    pub fn add_overlapping_aabb(&mut self) {
        self.overlapping_aabb += 1;
    }
    
    pub fn gridbox_intersection_checks(&self) -> usize { self.intersection_checks }
    
    pub fn add_gridbox_intersection_check(&mut self) { self.gridbox_intersection_checks += 1; }
}
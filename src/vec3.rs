use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub};
use serde::{Deserialize, Serialize};
use crate::rtweekend::{random_double, random_double_range};

#[derive(Default, Copy, Clone, Serialize, Deserialize, Debug)]
pub struct Vec3 {
    e: [f64; 3],
}

impl Vec3 {
    pub fn new(e0: f64, e1: f64, e2: f64) -> Self {
        Self { e: [e0, e1, e2] }
    }

    pub fn unit(&self) -> Self {
        *self / self.length()
    }

    pub fn random_in_unit_disk() -> Self {
        loop {
            let p = Vec3::new(
                random_double_range(-1.0, 1.0),
                random_double_range(-1.0, 1.0),
                0.0,
            );
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn random_unit() -> Self {
        loop {
            let p = Self::random_range(-1.0, 1.0);
            let len_squared = p.length_squared();
            if 1e-160 < len_squared && len_squared <= 1.0 {
                return p / len_squared.sqrt();
            }
        }
    }

    pub fn random_on_hemisphere(normal: &Vec3) -> Self {
        let on_unit_sphere = Self::random_unit();
        if on_unit_sphere.dot(normal) > 0.0 {
            // The random vector is in the same hemisphere as the normal vector
            on_unit_sphere
        } else {
            -on_unit_sphere
        }
    }

    pub fn reflect(v: &Vec3, n: &Vec3) -> Self {
        *v - 2.0 * v.dot(n) * *n
    }

    pub fn refract(uv: &Vec3, n: &Vec3, etai_over_etat: f64) -> Self {
        let cos_theta = f64::min((-*uv).dot(n), 1.0);
        let r_out_perp = etai_over_etat * (*uv + cos_theta * *n);
        let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * *n;
        r_out_perp + r_out_parallel
    }

    pub fn random() -> Self {
        Self::new(random_double(), random_double(), random_double())
    }

    pub fn random_range(min: f64, max: f64) -> Self {
        Self::new(
            random_double_range(min, max),
            random_double_range(min, max),
            random_double_range(min, max),
        )
    }

    pub fn x(&self) -> f64 { self.e[0] }
    pub fn y(&self) -> f64 { self.e[1] }
    pub fn z(&self) -> f64 { self.e[2] }
    
    pub fn set_x(&mut self, x: f64) { self.e[0] = x; }
    pub fn set_y(&mut self, y: f64) { self.e[1] = y; }
    pub fn set_z(&mut self, z: f64) { self.e[2] = z; }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> f64 {
        self.x() * self.x() + self.y() * self.y() + self.z() * self.z()
    }

    pub fn near_zero(&self) -> bool {
        let s = 1e-8;
        self.x().abs() < s && self.y().abs() < s && self.z().abs() < s
    }

    pub fn dot(&self, rhs: &Self) -> f64 {
        self.x() * rhs.x() + self.y() * rhs.y() + self.z() * rhs.z()
    }

    pub fn cross(self, rhs: &Self) -> Self {
        Self::new(
            self.y() * rhs.z() - self.z() * rhs.y(),
            self.z() * rhs.x() - self.x() * rhs.z(),
            self.x() * rhs.y() - self.y() * rhs.x(),
        )
    }
    
    pub fn red() -> Self { Self::new(1.0, 0.0, 0.0) }
    pub fn green() -> Self { Self::new(0.0, 1.0, 0.0) }
    pub fn blue() -> Self { Self::new(0.0, 0.0, 1.0) }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vec3 { e: [-self.e[0], -self.e[1], -self.e[2]] }
    }
}

impl Index<u32> for Vec3 {
    type Output = f64;

    fn index(&self, index: u32) -> &Self::Output {
        &self.e[index as usize]
    }
}

impl IndexMut<u32> for Vec3 {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self.e[index as usize]
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, v: Self) {
        self.e[0] += v.e[0];
        self.e[1] += v.e[1];
        self.e[2] += v.e[2];
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, t: f64) {
        self.e[0] *= t;
        self.e[1] *= t;
        self.e[2] *= t;
    }
}

impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, t: f64) {
        *self *= 1.0 / t;
    }
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.e[0], self.e[1], self.e[2])
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x() + rhs.x(), self.y() + rhs.y(), self.z() + rhs.z())
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x() - rhs.x(), self.y() - rhs.y(), self.z() - rhs.z())
    }
}

impl Mul for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x() * rhs.x(), self.y() * rhs.y(), self.z() * rhs.z())
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, t: f64) -> Self::Output {
        Vec3::new(self.x() * t, self.y() * t, self.z() * t)
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, vec: Vec3) -> Self::Output {
        vec * self
    }
}

impl Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, t: f64) -> Self::Output {
        (1.0 / t) * self
    }
}

// Just an alias, but useful for clarity
pub type Point3 = Vec3;

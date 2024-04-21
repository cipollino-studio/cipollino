use std::ops::{Mul, Add, Sub};

use glam::{Vec2, vec2};
use roots::find_roots_cubic;

use super::geo::LineSegment;

pub struct BezierSegment<T> where T: Mul<f32, Output = T> + Add<T, Output = T> + Sub<T, Output = T> + Copy + Default {
    pub p0: T,
    pub b0: T,
    pub a1: T,
    pub p1: T 
}

impl<T> BezierSegment<T> where T: Mul<f32, Output = T> + Add<T, Output = T> + Sub<T, Output = T> + Copy + Default {

    pub fn sample(&self, t: f32) -> T {
        let a = (1.0 - t) * (1.0 - t) * (1.0 - t);
        let b = 3.0 * (1.0 - t) * (1.0 - t) * t;
        let c = 3.0 * (1.0 - t) * t * t;
        let d = t * t * t; 

        self.p0 * a + self.b0 * b + self.a1 * c + self.p1 * d
    }

    pub fn dsample(&self, t: f32) -> T {
        let a = 3.0 * (1.0 - t) * (1.0 - t);
        let b = 6.0 * (1.0 - t) * t;
        let c = 3.0 * t * t;
        (self.b0 - self.p0) * a + (self.a1 - self.b0) * b + (self.p1 - self.a1) * c
    }

    pub fn map<F, M>(&self, f: F) -> BezierSegment<M> where F: Fn(T) -> M, M: Mul<f32, Output = M> + Add<M, Output = M> + Sub<M, Output = M> + Copy + Default {
        BezierSegment {
            p0: f(self.p0),
            b0: f(self.b0),
            a1: f(self.a1),
            p1: f(self.p1),
        }
    }

    pub fn to_discrete<const N: usize>(&self) -> [T; N] {
        let mut res = [T::default(); N];
        for i in 0..N {
            let t = (i as f32) / ((N - 1) as f32);
            res[i] = self.sample(t);
        }
        res
    }

}

impl BezierSegment<f32> {

    pub fn bounds(&self) -> (f32, f32) {
        let mut min = self.p0.min(self.p1);
        let mut max = self.p0.max(self.p1);
        
        let x = self.b0 - self.p0;
        let y = self.a1 - self.b0;
        let z = self.p1 - self.a1;
        let a = 3.0 * x - 6.0 * y + 3.0 * z;
        let b = -6.0 * x + 6.0 * y;
        let c = 3.0 * x;
        let det = b * b - 4.0 * a * c;

        if det > 0.0 {
            let t1 = (-b + det.sqrt()) / (2.0 * a);
            if 0.0 <= t1 && t1 <= 1.0 {
                let val = self.sample(t1);
                min = min.min(val);
                max = max.max(val);
            } 
            let t2 = (-b - det.sqrt()) / (2.0 * a);
            if 0.0 <= t2 && t2 <= 1.0 {
                let val = self.sample(t2);
                min = min.min(val);
                max = max.max(val);
            } 
        }

        (min, max)
    }

}

impl BezierSegment<Vec2> {

    pub fn bounding_box(&self) -> (Vec2, Vec2) {
        let x_curve = self.map(|pt| pt.x);
        let (x_min, x_max) = x_curve.bounds();
        let y_curve = self.map(|pt| pt.y);
        let (y_min, y_max) = y_curve.bounds();
        (vec2(x_min, y_min), vec2(x_max, y_max))
    }

    pub fn intersect_segment_ts(&self, segment: &LineSegment) -> Vec<f32> {
        let (q, d) = segment.get_characteristic();

        let c0 = self.p0.dot(q); 
        let c1 = self.b0.dot(q);
        let c2 = self.a1.dot(q);
        let c3 = self.p1.dot(q);

        let poly_a = -c0 + 3.0 * c1 - 3.0 * c2 + c3;
        let poly_b = 3.0 * c0 - 6.0 * c1 + 3.0 * c2;
        let poly_c = -3.0 * c0 + 3.0 * c1;
        let poly_d = c0 - d;

        let mut res = match find_roots_cubic(poly_a, poly_b, poly_c, poly_d) {
            roots::Roots::No(_) => Vec::new(),
            roots::Roots::One([x]) => vec![x],
            roots::Roots::Two([x, y]) => vec![x, y],
            roots::Roots::Three([x, y, z]) => vec![x, y, z],
            roots::Roots::Four(_) => panic!("should be impossible"),
        };

        res.retain(|t| *t >= 0.0 && *t <= 1.0);
        res.retain(|t| {
            let t = *t;
            let pt = self.sample(t);
            if (segment.p1.x - segment.p0.x).abs() > (segment.p1.y - segment.p0.y).abs() {
                pt.x >= segment.p0.x.min(segment.p1.x) && pt.x <= segment.p0.x.max(segment.p1.x) 
            } else {
                pt.y >= segment.p0.y.min(segment.p1.y) && pt.y <= segment.p0.y.max(segment.p1.y) 
            }
        });

        res
    }

    pub fn intersect_segment(&self, segment: &LineSegment) -> Vec<Vec2> {
        self.intersect_segment_ts(segment).iter().map(|t| self.sample(*t)).collect()
    }

}

extern "C" {

    pub fn curve_fit_cubic_to_points_refit_fl(
        points: *const std::ffi::c_float,
        points_len: std::ffi::c_uint,
        dims: std::ffi::c_uint,
        error_threshold: std::ffi::c_float,
        calc_flag: std::ffi::c_uint,
        corners: *mut std::ffi::c_uint,
        corners_len: std::ffi::c_uint,
        corner_angle: std::ffi::c_float,
        
        r_cubic_array: *mut *mut std::ffi::c_float,
        r_cubic_array_len: *mut std::ffi::c_uint,
        r_cubic_orig_index: *mut *mut std::ffi::c_uint,
        r_corner_index_array: *mut *mut std::ffi::c_uint,
        r_corner_index_len: *mut std::ffi::c_uint
    ) -> std::ffi::c_int;

    pub fn free(ptr: *mut std::ffi::c_void);

}

pub fn fit_curve(dims: i32, points: &[f32], err: f32) -> Vec<f32> {
    unsafe {
        let mut r_cubic_array: *mut f32 = std::ptr::null_mut();
        let mut r_cubic_array_len: u32 = 0;
        curve_fit_cubic_to_points_refit_fl(
            points.as_ptr(),
            (points.len() / (dims as usize)) as u32,
            dims as u32,
            err, 
            0,
            std::ptr::null_mut(),
            0,
            std::f32::consts::PI * 5.0,
            &mut r_cubic_array as *mut *mut f32,
            &mut r_cubic_array_len as &mut u32,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut());
        
        let mut res = Vec::new();
        for i in 0..(r_cubic_array_len * 3) {
            for j in 0..dims {
                res.push(*r_cubic_array.add((i * (dims as u32) + (j as u32)) as usize));
            }
        }
        free(r_cubic_array as *mut std::ffi::c_void);
        res
    }
}

use std::ops::{Mul, Add, Sub};

use glam::Vec2;


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

pub fn bezier_sample<T>(t: f32, p0: T, b0: T, a1: T, p1: T) -> T
    where T: Mul<f32, Output = T> + Add<T, Output = T> + Copy {
    let a = (1.0 - t) * (1.0 - t) * (1.0 - t);
    let b = 3.0 * (1.0 - t) * (1.0 - t) * t;
    let c = 3.0 * (1.0 - t) * t * t;
    let d = t * t * t; 

    p0 * a + b0 * b + a1 * c + p1 * d
}

pub fn bezier_dsample<T>(t: f32, p0: T, b0: T, a1: T, p1: T) -> T
    where T: Mul<f32, Output = T> + Add<T, Output = T> + Sub<T, Output = T> + Copy {
    let a = 3.0 * (1.0 - t) * (1.0 - t);
    let b = 6.0 * (1.0 - t) * t;
    let c = 3.0 * t * t;
    (b0 - p0) * a + (a1 - b0) * b + (p1 - a1) * c
}

pub fn bezier_min_max(p0: f32, b0: f32, a1: f32, p1: f32) -> (f32, f32) {
    let mut min = p0.min(p1);
    let mut max = p0.max(p1);
    
    let x = b0 - p0;
    let y = a1 - b0;
    let z = p1 - a1;
    let a = 3.0 * x - 6.0 * y + 3.0 * z;
    let b = -6.0 * x + 6.0 * y;
    let c = 3.0 * x;
    let det = b * b - 4.0 * a * c;

    if det > 0.0 {
        let t1 = (-b + det.sqrt()) / (2.0 * a);
        if 0.0 <= t1 && t1 <= 1.0 {
            let val = bezier_sample(t1, p0, b0, a1, p1);
            min = min.min(val);
            max = max.max(val);
        } 
        let t2 = (-b - det.sqrt()) / (2.0 * a);
        if 0.0 <= t2 && t2 <= 1.0 {
            let val = bezier_sample(t2, p0, b0, a1, p1);
            min = min.min(val);
            max = max.max(val);
        } 
    }

    (min, max)
}

pub fn bezier_bounding_box(p0: Vec2, b0: Vec2, a1: Vec2, p1: Vec2) -> (Vec2, Vec2) {
    let (left, right) = bezier_min_max(p0.x, b0.x, a1.x, p1.x);
    let (bottom, top) = bezier_min_max(p0.y, b0.y, a1.y, p1.y);
    (Vec2::new(left, bottom), Vec2::new(right, top))
}

// TODO: replace this with something more sophisticated to maximize detail and minimize number of points
// Maybe use the curve's curvature for this?
// TODO: make this an iterator
pub fn bezier_to_discrete_t_vals(_p0: Vec2, _b0: Vec2, _a1: Vec2, _p1: Vec2, max_pts: i32, include_first: bool) -> Vec<f32> {
    let mut vals = Vec::new();
    for i in 0..max_pts {
        vals.push(if include_first { (i as f32) / ((max_pts - 1) as f32) } else { ((i + 1) as f32) / (max_pts as f32) });
    }
    vals
}

pub fn bezier_to_discrete(p0: Vec2, b0: Vec2, a1: Vec2, p1: Vec2, max_pts: i32, include_first: bool) -> Vec<Vec2> {
    bezier_to_discrete_t_vals(p0, b0, a1, p1, max_pts, include_first).iter().map(|t| bezier_sample(*t, p0, b0, a1, p1)).collect()
}

pub fn bezier_to_discrete_segments(p0: Vec2, b0: Vec2, a1: Vec2, p1: Vec2, max_pts: i32, include_first: bool) -> Vec<(Vec2, Vec2)> {
    bezier_to_discrete(p0, b0, a1, p1, max_pts, include_first).windows(2).map(|pts| (pts[0], pts[1])).collect()
}
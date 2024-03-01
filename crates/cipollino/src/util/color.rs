
use glam::{vec4, Vec4};

pub const VALUE_TOLERANCE: f32 = 0.00001;

pub fn rgba_to_hsva(rgba: Vec4) -> Vec4 {
    let r = rgba.x;
    let g = rgba.y;
    let b = rgba.z;
    let a = rgba.w;
    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let dif = max - min;
    let h = if dif == 0.0 {
        0.0
    } else if max == r {
        (g - b) / dif
    } else if max == g {
        (b - r) / dif + 2.0
    } else {
        (r - g) / dif + 4.0
    } / 6.0;
    let v = max;
    let s = if v == 0.0 {
        0.0
    } else {
        dif / v
    };
    vec4(h, s.max(VALUE_TOLERANCE), v.max(VALUE_TOLERANCE), a)
}

pub fn hsva_to_rgba(hsva: Vec4) -> Vec4 {
    let h = (hsva.x + hsva.x.floor().abs()).fract() * 6.0;
    let s = hsva.y.max(VALUE_TOLERANCE);
    let v = hsva.z.max(VALUE_TOLERANCE);
    let a = hsva.w;
    
    let alpha = v * (1.0 - s);
    let beta = v * (1.0 - (h - h.floor()) * s);
    let gamma = v * (1.0 - (1.0 - (h - h.floor())) * s);

    if 0.0 <= h && h < 1.0 {
        vec4(v, gamma, alpha, a)
    } else if 1.0 <= h && h < 2.0 {
        vec4(beta, v, alpha, a) 
    } else if 2.0 <= h && h < 3.0 {
        vec4(alpha, v, gamma, a)
    } else if 3.0 <= h && h < 4.0 {
        vec4(alpha, beta, v, a)
    } else if 4.0 <= h && h < 5.0 {
        vec4(gamma, alpha, v, a)
    } else {
        vec4(v, alpha, beta, a)
    }
}

pub fn contrast_color(col: Vec4) -> Vec4 {
    if (col.x + col.y + col.z) / 3.0 > 0.5 {
        vec4(0.0, 0.0, 0.0, 1.0) 
    } else {
        Vec4::ONE
    }
}
use glam::{Vec2, vec2};
use intersection_detection::{Intersection, point_like::Between};

pub fn segment_intersect(a0: Vec2, a1: Vec2, b0: Vec2, b1: Vec2) -> Option<Vec2> {

    // I was too lazy to code this myself...

    use intersection_detection::Line;
    let line1 = Line::new([a0.x, a0.y], [a1.x, a1.y]);
    let line2 = Line::new([b0.x, b0.y], [b1.x, b1.y]);

    if let Some(intersection) = line1
        .intersection(&line2)
        .try_into_intersection()
        .ok() {
            if let Intersection::Point(p) = intersection {
                return Some(Vec2::new(p[0], p[1]));
            }
    }
    None
}

pub fn segment_aabb_intersect(a0: Vec2, a1: Vec2, bb_min: Vec2, bb_max: Vec2) -> bool {
    // This is probably not the fastest way to do this.
    // However, its simple and it works, and this will still speed up things
    // when used to do AABB-based culling(like in the fill bucket)
    let minmax = vec2(bb_min.x, bb_max.y);
    let maxmin = vec2(bb_max.x, bb_min.y);
    if let Some(_) = segment_intersect(a0, a1, bb_min, minmax) {
        return true;
    }
    if let Some(_) = segment_intersect(a0, a1, minmax, bb_max) {
        return true;
    }
    if let Some(_) = segment_intersect(a0, a1, bb_max, maxmin) {
        return true;
    }
    if let Some(_) = segment_intersect(a0, a1, maxmin, bb_min) {
        return true;
    }
    if a0.x.is_between(bb_min.x, bb_max.x) && a0.y.is_between(bb_min.y, bb_max.y) {
        return true;
    }
    if a1.x.is_between(bb_min.x, bb_max.x) && a1.y.is_between(bb_min.y, bb_max.y) {
        return true;
    }
    false
}
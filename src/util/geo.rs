use glam::Vec2;
use intersection_detection::Intersection;

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

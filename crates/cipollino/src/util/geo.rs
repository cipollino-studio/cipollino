use glam::{vec2, vec3, Vec2, Vec3};
use intersection_detection::Intersection;

pub struct LineSegment {
    pub p0: Vec2,
    pub p1: Vec2 
}

impl LineSegment {
    
    pub fn new(p0: Vec2, p1: Vec2) -> Self {
        LineSegment {
            p0,
            p1
        }
    }

    pub fn intersect(&self, line: LineSegment) -> Option<Vec2> {
        // I was too lazy to code this myself...

        use intersection_detection::Line;
        let line1 = Line::new([self.p0.x, self.p0.y], [self.p1.x, self.p1.y]);
        let line2 = Line::new([line.p0.x, line.p0.y], [line.p1.x, line.p1.y]);

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

    // Returns the vector q and value d such that for all points p on the line, q dot p = d.
    pub fn get_characteristic(&self) -> (Vec2, f32) {
        // y = mx + b
        // y - mx = b
        // y - (y1 - y0) / (x1 - x0) * x = b
        // (x1 - x0) * y + (y0 - y1) * x = b * (x1 - x0)

        // Note that b = y0 - (y1 - y0) / (x1 - x0) * x0
        // b * (x1 - x0) = y0 * (x1 - x0) - x0 * (y1 - y0)

        (
            vec2(self.p0.y - self.p1.y, self.p1.x - self.p0.x),
            self.p0.y * (self.p1.x - self.p0.x) - self.p0.x * (self.p1.y - self.p0.y)
        )
    }

}

pub fn vec2_to_vec3(vec: Vec2) -> Vec3 {
    vec3(vec.x, vec.y, 0.0)
}
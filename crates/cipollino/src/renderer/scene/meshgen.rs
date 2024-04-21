
use std::{f32::consts, sync::Arc};

use crate::{project::{obj::ObjPtr, stroke::{iter_bezier_segments, Stroke}, Project}, renderer::mesh::Mesh};

use super::SceneRenderer;

fn unfilled_mesh(project: &Project, stroke_ptr: ObjPtr<Stroke>, gl: &Arc<glow::Context>) -> Option<Mesh> {
    let stroke = project.strokes.get(stroke_ptr)?;
    let mut mesh = Mesh::new(vec![2], gl);

    let mut top_pts = Vec::new();
    let mut btm_pts = Vec::new();
    let r = stroke.r;
    for segment in stroke.iter_bezier_segments() { 
        for i in 0..10 {
            let t = (i as f32) / 9.0; 
            let pt = segment.sample(t); 
            let tang = segment.dsample(t).normalize(); 
            let norm = glam::vec2(-tang.y, tang.x); 

            top_pts.push(pt + norm * r);
            btm_pts.push(pt - norm * r);
        }
    }

    let mut curr_idx = 0;
    let mut verts = Vec::new();
    let mut idxs = Vec::new();
    if top_pts.len() > 0 {
        for i in 0..(top_pts.len() - 1) {
            let t0 = top_pts[i];
            let t1 = top_pts[i + 1];
            let b0 = btm_pts[i];
            let b1 = btm_pts[i + 1];

            verts.push(t0.x);
            verts.push(t0.y);
            verts.push(t1.x);
            verts.push(t1.y);
            verts.push(b0.x);
            verts.push(b0.y);
            verts.push(b1.x);
            verts.push(b1.y);

            idxs.push(curr_idx + 0);
            idxs.push(curr_idx + 1);
            idxs.push(curr_idx + 2);

            idxs.push(curr_idx + 1);
            idxs.push(curr_idx + 2);
            idxs.push(curr_idx + 3);

            curr_idx += 4;
        }
    }

    // Stroke caps
    if top_pts.len() > 0 {

        let mut add_cap = |p0: glam::Vec2, p1: glam::Vec2| {
            let center = (p0 + p1) * 0.5;
            let r = (p0 - center).length();
            let up = (p0 - center).normalize();
            let left = glam::vec2(-up.y, up.x);

            verts.push(center.x);
            verts.push(center.y);
            curr_idx += 1;
            let n = 20;
            for i in 0..n {
                let a = consts::PI * (i as f32) / 19.0;
                let pt = center + r * (up * a.cos() + left * a.sin());
                verts.push(pt.x);
                verts.push(pt.y);
            }
            for i in 0..(n - 1) {
                idxs.push(curr_idx - 1);
                idxs.push(curr_idx + i);
                idxs.push(curr_idx + i + 1);
            }
            curr_idx += n;
        };

        add_cap(top_pts[0], btm_pts[0]);
        add_cap(*btm_pts.last().unwrap(), *top_pts.last().unwrap());
        
    }

    mesh.upload(&verts, &idxs, gl);
    Some(mesh)
}

fn filled_mesh(project: &Project, stroke_ptr: ObjPtr<Stroke>, gl: &Arc<glow::Context>) -> Option<Mesh> {
    let stroke = project.strokes.get(stroke_ptr)?;
    let mut mesh = Mesh::new(vec![2], gl);

    let mut verts = Vec::new();
    // Triangle fan source
    verts.push(0.0);
    verts.push(0.0);

    let mut idxs = Vec::new();

    for chain in &stroke.points {
        let mut polygon_pts = Vec::new();
        for segment in iter_bezier_segments(chain) { 
            polygon_pts.extend_from_slice(segment.to_discrete::<10>().as_slice()); 
        }
        
        let first_pt_idx = verts.len() / 2;
        
        for pt in &polygon_pts {
            verts.push(pt.x);
            verts.push(pt.y);
        }

        for i in 0..polygon_pts.len() {
            idxs.push(0);
            idxs.push((first_pt_idx + i) as u32);
            idxs.push(((first_pt_idx + (i + 1) % polygon_pts.len())) as u32);
        }
    }


    mesh.upload(&verts, &idxs, gl);
    
    Some(mesh)
}

impl SceneRenderer {

    pub fn get_mesh<'a>(&'a mut self, project: &Project, stroke_ptr: ObjPtr<Stroke>, gl: &Arc<glow::Context>) -> Option<&'a Mesh> {
        if !self.stroke_meshes.contains_key(&stroke_ptr) {
            if let Some(stroke) = project.strokes.get(stroke_ptr) {
                let stroke_filled = stroke.filled;
                let mesh = if stroke_filled { filled_mesh(project, stroke_ptr, gl) } else { unfilled_mesh(project, stroke_ptr, gl) };
                if let Some(mesh) = mesh {
                    if let Some(prev_mesh) = self.stroke_meshes.insert(stroke_ptr, mesh) {
                        prev_mesh.delete(gl);
                    }
                }
            }
        }
        self.stroke_meshes.get(&stroke_ptr)
    }

}
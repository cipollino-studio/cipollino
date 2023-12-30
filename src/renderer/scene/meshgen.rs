
use std::{sync::Arc, f32::consts};

use crate::{renderer::mesh::Mesh, project::Project, util::curve};

pub fn get_mesh<'a>(project: &'a mut Project, stroke_key: u64, gl: &Arc<glow::Context>) -> Option<&'a Mesh> {
    let stroke = project.strokes.get_mut(&stroke_key)?;
    if stroke.need_remesh {
        let mut mesh = Mesh::new(vec![2], gl);

        let mut top_pts = Vec::new();
        let mut btm_pts = Vec::new();
        let r = 0.05;
        let pt_keys = stroke.points.clone();
        if pt_keys.len() > 0 {
            for i in 0..(pt_keys.len() - 1) {
                let p0 = project.points.get(&pt_keys[i])?;
                let p1 = project.points.get(&pt_keys[i + 1])?;
                
                for j in 0..10 {
                    let t = (j as f32) / 9.0;
                    let pt = curve::bezier_sample(t, p0.data.pt, p0.data.b, p1.data.a, p1.data.pt);
                    let tang = curve::bezier_dsample(t, p0.data.pt, p0.data.b, p1.data.a, p1.data.pt).normalize();
                    let norm = glam::vec2(-tang.y, tang.x); 

                    top_pts.push(pt + norm * r);
                    btm_pts.push(pt - norm * r);
                }
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

        stroke.mesh = Some(mesh);
        stroke.need_remesh = false;
    }
    stroke.mesh.as_ref()
}


use std::{sync::Arc, collections::{VecDeque, HashSet, HashMap}};

use glam::{Vec2, vec2};
use crate::{editor::EditorState, panels::scene::ScenePanel, project::{action::Action, obj::child_obj::ChildObj, stroke::{Stroke, StrokeColor, StrokeMesh, StrokePoint}}, util::{curve::{bezier_bounding_box, bezier_dsample, bezier_sample, bezier_to_discrete_t_vals, fit_curve}, geo::{segment_aabb_intersect, segment_intersect}}};

use super::{Tool, active_frame};

pub struct Bucket {

}

impl Bucket {

    pub fn new() -> Self {
        Self {

        }
    }

}

impl Tool for Bucket {

    fn mouse_click(&mut self, mouse_pos: Vec2, state: &mut EditorState, _ui: &mut egui::Ui, scene: &mut ScenePanel, gl: &Arc<glow::Context>) {

        // If we click on an existing stroke, let's just change its color
        if let Some(stroke) = scene.sample_pick(mouse_pos, gl) {
            if let Some(act) = Stroke::set_color(&mut state.project, stroke, StrokeColor::Color(state.color)) {
                state.actions.add(Action::from_single(act));
                return;
            }
        }

        // Figure out the active frame in advance in case it doesn't exist
        let active_frame = active_frame(state);
        if active_frame.is_none() {
            return;
        }
        let (frame, mut acts) = active_frame.unwrap();
        
        // This uses a standard bitmap floodfill algorithmn adapted to work with vector art
        // Is this the best approach? Probably not.

        let grid_size = 1.5;

        let snap_coords = |pt: Vec2| {
            ((pt.x / grid_size).floor() as i32, (pt.y / grid_size).floor() as i32)
        };
        let unsnap_coords = |(x, y): (i32, i32)| {
            Vec2::new((x as f32) * grid_size, (y as f32) * grid_size)
        };

        // Step 1: Floodfill using BFS to find the boundary points
        let mut bfs = VecDeque::new();
        let mut vis = HashSet::new();
        let mouse_snap = snap_coords(mouse_pos);
        bfs.push_back(mouse_snap);
        vis.insert(mouse_snap);
        let offsets = [
            [ 1,  0],
            [ 0,  1],
            [-1,  0],
            [ 0, -1],
        ];
        let visible_strokes = state.visible_strokes();
        let mut boundary = HashMap::new(); 
        while let Some(curr) = bfs.pop_front() {
            let curr_unsnapped = unsnap_coords(curr);
            'offset: for [x_off, y_off] in offsets {
                let next = (curr.0 + x_off, curr.1 + y_off);
                if vis.contains(&next) {
                    continue;
                }
                let next_unsnapped = unsnap_coords(next);
                if next_unsnapped.y > scene.cam_pos.y + scene.cam_size || next_unsnapped.y < scene.cam_pos.y - scene.cam_size || next_unsnapped.x < scene.cam_pos.x - scene.cam_size * scene.cam_aspect || next_unsnapped.x > scene.cam_pos.x + scene.cam_size * scene.cam_aspect {
                    boundary.insert(next, next_unsnapped);
                    continue 'offset;
                }
                for stroke in &visible_strokes {
                    if let Some(stroke) = state.project.strokes.get(*stroke) {
                        let r = if !stroke.filled { (stroke.r - 0.4).max(0.0) } else { 0.0 };
                        'point_pairs: for (p0, p1) in stroke.iter_point_pairs() {

                            let (mut bb_min, mut bb_max) = bezier_bounding_box(p0.pt, p0.b, p1.a, p1.pt);
                            bb_min -= Vec2::splat(r * 2.0);
                            bb_max += Vec2::splat(r * 2.0);
                            if !segment_aabb_intersect(curr_unsnapped, next_unsnapped, bb_min, bb_max) {
                                continue 'point_pairs;
                            }

                            let mut top_pts = vec![];
                            let mut btm_pts = vec![];
                            for t in bezier_to_discrete_t_vals(p0.pt, p0.b, p1.a, p1.pt, 10, true) {
                                let pt = bezier_sample(t, p0.pt, p0.b, p1.a, p1.pt);
                                let tang = bezier_dsample(t, p0.pt, p0.b, p1.a, p1.pt).normalize();
                                let norm = vec2(tang.y, -tang.x);
                                top_pts.push(pt + norm * r); 
                                btm_pts.push(pt - norm * r); 
                            }

                            for pts in top_pts.windows(2) {
                                if let Some(intersect) = segment_intersect(pts[0], pts[1], curr_unsnapped, next_unsnapped) {
                                    boundary.insert(next, intersect);
                                    continue 'offset;
                                }
                            }
                            for pts in btm_pts.windows(2) {
                                if let Some(intersect) = segment_intersect(pts[0], pts[1], curr_unsnapped, next_unsnapped) {
                                    boundary.insert(next, intersect);
                                    continue 'offset;
                                }
                            }
                        }

                        if !stroke.filled {
                            let mut collide_with_end_cap = |p0: Vec2, p1: Vec2| {
                                let tang = (p1 - p0).normalize() * r;
                                let norm = vec2(-tang.y, tang.x);
                                for i in 0..10 {
                                    let a0 = i as f32 / 10.0 * std::f32::consts::PI;
                                    let a1 = (i + 1) as f32 / 10.0 * std::f32::consts::PI;
                                    let p0 = p1 + a0.cos() * norm - a0.sin() * tang;
                                    let p1 = p1 + a1.cos() * norm - a1.sin() * tang;
                                    if let Some(intersect) = segment_intersect(p0, p1, curr_unsnapped, next_unsnapped) {
                                        boundary.insert(next, intersect);
                                        return true;
                                    }
                                }
                                return false;
                            };

                            if !stroke.points.is_empty() {
                                let pt = stroke.points[0].first().unwrap(); 
                                if collide_with_end_cap(pt.a, pt.pt) {
                                    continue 'offset;
                                }
                                let pt = stroke.points[0].last().unwrap(); 
                                if collide_with_end_cap(pt.b, pt.pt) {
                                    continue 'offset;
                                }
                            }
                        }
                    }
                }
                vis.insert(next);
                bfs.push_back(next);
            }
        }


        // Step 2: Eliminate "Tails"
        // We only care about loops, not single lines in the middle of nowhere.
        // We launch a BFS from each point in the boundary with only 1 neighbour,
        // Eliminate every point with less than three neighbours that touch it.
        let offsets = [
            [ 1,  0],
            [ 1, -1],
            [ 0, -1],
            [-1, -1],
            [-1,  0],
            [-1,  1],
            [ 0,  1],
            [ 1,  1],
        ];

        let mut neighbour_cnt = HashMap::new();
        let mut tail_bfs = VecDeque::new();
        for (pt, _) in boundary.iter() {
            let (x, y) = *pt;
            let mut neighbours = 0;
            for [x_off, y_off] in offsets {
                let next = (x + x_off, y + y_off); 
                if boundary.contains_key(&next) {
                    neighbours += 1;
                }
            }
            neighbour_cnt.insert(*pt, neighbours);
            if neighbours == 1 {
                tail_bfs.push_back(*pt);
            }
        }

        let remove_pt = |pt: (i32, i32), boundary: &mut HashMap<(i32, i32), Vec2>, neighbour_cnt: &mut HashMap<(i32, i32), i32>, tail_bfs: &mut VecDeque<(i32, i32)>| {
            let val = boundary.remove(&pt);
            neighbour_cnt.remove(&pt);
            let (x, y) = pt;
            for [x_off, y_off] in offsets {
                let next = (x + x_off, y + y_off);
                if let Some(cnt) = neighbour_cnt.get_mut(&next) {
                    *cnt -= 1;
                    if *cnt == 1 {
                        tail_bfs.push_back(next);
                    }
                }
            }
            val
        };

        // Step 3: Find the boundary chain(s)
        // Uses the algorithm described here: https://en.wikipedia.org/wiki/Moore_neighborhood
        let mut chains = Vec::new();
        'find_chains: while !boundary.is_empty() { 

            // Remove tails
            while let Some(curr) = tail_bfs.pop_front() {
                remove_pt(curr, &mut boundary, &mut neighbour_cnt, &mut tail_bfs);
            }

            // Remove corners
            for pt in boundary.keys().map(|pt| *pt).collect::<Vec<(i32, i32)>>() {
                let up = boundary.contains_key(&(pt.0, pt.1 + 1));
                let down = boundary.contains_key(&(pt.0, pt.1 - 1));
                let right = boundary.contains_key(&(pt.0 + 1, pt.1));
                let left = boundary.contains_key(&(pt.0 - 1, pt.1));
                
                let vertical = up || down;
                let horizontal = left || right;
                if vertical && horizontal && *neighbour_cnt.get(&pt).unwrap() == 2 {
                    remove_pt(pt, &mut boundary, &mut neighbour_cnt, &mut tail_bfs);
                }
            }

            if boundary.is_empty() {
                break;
            }

            let mut chain = Vec::new();
            
            // We find the left and bottomost point instead of
            // the first in the hash map to make the algorithm
            // deterministic.
            let mut curr = (i32::MAX, i32::MAX);
            for pt in boundary.keys() {
                let pt = *pt;
                curr = curr.min(pt);
            }
            chain.push(remove_pt(curr, &mut boundary, &mut neighbour_cnt, &mut tail_bfs).unwrap());
            let first = curr;
            let mut dir = 0;
            let mut attempts = 0;
            loop {
                let next = (curr.0 + offsets[dir][0], curr.1 + offsets[dir][1]);
                if next == first {
                    break;
                }
                if let Some(pt) = remove_pt(next, &mut boundary, &mut neighbour_cnt, &mut tail_bfs) {
                    chain.push(pt);
                    dir = (dir + 5) % 8;
                    curr = next;
                    attempts = 0;
                } else {
                    dir += 1;
                    dir %= 8;

                    attempts += 1;
                    if attempts > 8 {
                        continue 'find_chains; 
                    }
                }
            }
            chains.push(chain);
        }

        // Nothing to fill
        if chains.is_empty() {
            return;
        }

        // Step 4: Convert to the final bezier form
        let mut all_pts = Vec::new();
        for (_chain_idx, chain) in chains.iter().enumerate() {
            let mut pts_data = Vec::new();
            for pt in chain {
                pts_data.push(pt.x);
                pts_data.push(pt.y);
            }
            let curve_pts = fit_curve(2, &pts_data.as_slice(), grid_size * 0.5);
            let mut pts = Vec::new();
            for i in 0..(curve_pts.len() / (2 * 3)) {
                let a = glam::vec2(curve_pts[i * 6 + 0], curve_pts[i * 6 + 1]);
                let p = glam::vec2(curve_pts[i * 6 + 2], curve_pts[i * 6 + 3]);
                let b = glam::vec2(curve_pts[i * 6 + 4], curve_pts[i * 6 + 5]);
                pts.push(StrokePoint {
                    a,
                    pt: p,
                    b
                }); 
            }
            all_pts.push(pts);
        }

        if let Some((_, act)) = Stroke::add_at_idx(&mut state.project, frame, Stroke {
            frame: frame,
            color: StrokeColor::Color(state.color),
            r: 0.05,
            filled: true,
            points: all_pts,
            mesh: StrokeMesh::new()
        }, 0) {
            acts.push(act);
        }

        state.actions.add(Action::from_list(acts));

    }

    fn get_icon(&self) -> &str {
        egui_phosphor::regular::PAINT_BUCKET
    }

    fn name(&self) -> &str {
        "Bucket"
    }

    fn shortcut(&self) -> egui::KeyboardShortcut {
        egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::B)
    }

}

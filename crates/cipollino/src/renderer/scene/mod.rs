
mod meshgen;
mod fb_manager;

use std::{collections::HashMap, sync::Arc};

use glam::vec4;
use glow::{Context, HasContext};

use crate::project::{graphic::Graphic, layer::{BlendingMode, Layer, LayerKind}, obj::{obj_list::ObjListTrait, ObjBox, ObjPtr}, stroke::Stroke, Project};

use self::fb_manager::FramebufferManager;

use super::{shader::Shader, fb::Framebuffer, mesh::Mesh};

impl BlendingMode {

    fn get_shader<'a>(&self, renderer: &'a mut SceneRenderer) -> &'a mut Shader {
        match self {
            BlendingMode::Normal => &mut renderer.blend_normal,
            BlendingMode::Add => &mut renderer.blend_add,
            BlendingMode::Screen => &mut renderer.blend_screen,
            BlendingMode::ColorDodge => &mut renderer.blend_color_dodge,
            BlendingMode::Multiply => &mut renderer.blend_multiply,
            BlendingMode::ColorBurn => &mut renderer.blend_color_burn,
            BlendingMode::Overlay => &mut renderer.blend_overlay,
            BlendingMode::SoftLight => &mut renderer.blend_soft_light,
            BlendingMode::HardLight => &mut renderer.blend_hard_light,
            BlendingMode::VividLight => &mut renderer.blend_vivid_light,
            BlendingMode::Color => &mut renderer.blend_color,
        }
    } 

}

pub struct SceneRenderer {
    pub flat_color_shader: Shader,
    pub circle_shader: Shader,
    pub quad: Mesh,

    pub clip_shadow_mesh: Mesh, 
    pub clip_shadow_shader: Shader,

    pub screen_quad: Mesh,
    pub screen_shader: Shader,

    stroke_meshes: HashMap<ObjPtr<Stroke>, Mesh>,

    framebuffers: FramebufferManager,

    blend_normal: Shader,
    blend_add: Shader,
    blend_screen: Shader,
    blend_color_dodge: Shader,
    blend_multiply: Shader,
    blend_color_burn: Shader,
    blend_overlay: Shader,
    blend_soft_light: Shader,
    blend_hard_light: Shader,
    blend_vivid_light: Shader,
    blend_color: Shader,
}

impl SceneRenderer {

    pub fn new(gl: &Arc<Context>) -> Self {
        let mut quad = Mesh::new(vec![2], gl);
        quad.upload(&vec![
            -0.5, -0.5,
             0.5, -0.5,
            -0.5,  0.5,
             0.5,  0.5
        ], &vec![
            0, 1, 2,
            1, 2, 3
        ], gl);

        let mut clip_shadow_mesh = Mesh::new(vec![2], gl);
        clip_shadow_mesh.upload(&vec![
            -1.0,  1.0,
             1.0,  1.0,
             1.0, -1.0,
            -1.0, -1.0,
            -0.5,  0.5,
             0.5,  0.5,
             0.5, -0.5,
            -0.5, -0.5
        ], &vec![
            0, 1, 4,
            1, 5, 4,
            1, 2, 5,
            2, 6, 5,
            2, 6, 3,
            3, 6, 7,
            3, 7, 0,
            0, 4, 7
        ], gl);

        let mut screen_quad = Mesh::new(vec![2, 2], gl);
        screen_quad.upload(&vec![
            -1.0, -1.0, 0.0, 0.0,
             1.0, -1.0, 1.0, 0.0,
            -1.0,  1.0, 0.0, 1.0,
             1.0,  1.0, 1.0, 1.0
        ], &vec![
            0, 1, 2,
            1, 2, 3
        ], gl);

        macro_rules! blend_mode_shader {
            ($mode:literal) => {
                Shader::new(
                    include_str!("shaders/screen_vs.glsl"),
                    concat!(include_str!("shaders/blending/blend_fs_template_begin.glsl"), include_str!(concat!("shaders/blending/modes/", $mode, ".glsl")), include_str!("shaders/blending/blend_fs_template_end.glsl")),
                    gl
                ) 
            };
        }

        Self {
            flat_color_shader: Shader::new(include_str!("shaders/flat_color_vs.glsl"), include_str!("shaders/flat_color_fs.glsl"), gl),
            circle_shader: Shader::new(include_str!("shaders/circle_vs.glsl"), include_str!("shaders/circle_fs.glsl"), gl),
            quad,

            clip_shadow_mesh,
            clip_shadow_shader: Shader::new(include_str!("shaders/clip_shadow_vs.glsl"), include_str!("shaders/clip_shadow_fs.glsl"), gl),

            screen_quad,
            screen_shader: Shader::new(include_str!("shaders/screen_vs.glsl"), include_str!("shaders/screen_fs.glsl"), gl),

            stroke_meshes: HashMap::new(),

            framebuffers: FramebufferManager::new(),

            blend_normal: blend_mode_shader!("normal"),
            blend_add: blend_mode_shader!("add"),
            blend_screen: blend_mode_shader!("screen"),
            blend_color_dodge: blend_mode_shader!("color_dodge"),
            blend_multiply: blend_mode_shader!("multiply"),
            blend_color_burn: blend_mode_shader!("color_burn"),
            blend_overlay: blend_mode_shader!("overlay"), 
            blend_soft_light: blend_mode_shader!("soft_light"),
            blend_hard_light: blend_mode_shader!("hard_light"), 
            blend_vivid_light: blend_mode_shader!("vivid_light"), 
            blend_color: blend_mode_shader!("color") 
        }
    }

    fn render_stroke_mesh(&mut self, project: &Project, stroke_ptr: ObjPtr<Stroke>, gl: &Arc<Context>) {
        let mesh = if let Some(mesh) = self.get_mesh(project, stroke_ptr, gl) {
            mesh
        } else {
            return;
        };
        mesh.render(gl);
    }

    fn render_stroke(&mut self, project: &Project, stroke_ptr: ObjPtr<Stroke>, color_override: Option<glam::Vec4>, gl: &Arc<Context>) {
        let stroke = if let Some(stroke) = project.strokes.get(stroke_ptr) {
            stroke
        } else {
            return;
        };
        let filled = stroke.filled;
        let color = color_override.unwrap_or(stroke.color.get_color(project));

        if !filled {
            unsafe {
                gl.clear(glow::DEPTH_BUFFER_BIT);
            }
            self.flat_color_shader.set_vec4("uColor", glam::vec4(color.x, color.y, color.z, color.w), gl);
            self.render_stroke_mesh(project, stroke_ptr, gl);
        } else {
            self.flat_color_shader.set_vec4("uColor", glam::vec4(color.x, color.y, color.z, 1.0), gl);
            unsafe {
                gl.enable(glow::STENCIL_TEST);
                gl.stencil_mask(0xFF);
                gl.clear(glow::STENCIL_BUFFER_BIT);
                gl.stencil_func(glow::NEVER, 1, 0xFF);
                gl.stencil_op(glow::INVERT, glow::INVERT, glow::INVERT);
            }
            self.render_stroke_mesh(project, stroke_ptr, gl);
            unsafe {
                gl.stencil_func(glow::EQUAL, 0xFF, 0xFF);
                gl.stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);
                gl.stencil_mask(0);
                gl.clear(glow::DEPTH_BUFFER_BIT);
            }
            self.flat_color_shader.set_vec4("uColor", glam::vec4(color.x, color.y, color.z, color.w), gl);
            self.render_stroke_mesh(project, stroke_ptr, gl);
            unsafe {
                gl.disable(glow::STENCIL_TEST);
            }
        }
    }

    fn render_onion_skin<'a, I>(&mut self, project: &Project, time: i32, onion_before: i32, onion_after: i32, layer_iter: I, gl: &Arc<Context>) where I: Iterator<Item = &'a &'a ObjBox<Layer>> {
        let mut onion_before_strokes = vec![Vec::new(); onion_before as usize];
        let mut onion_after_strokes = vec![Vec::new(); onion_before as usize];
        for layer in layer_iter { 
            let layer = layer.get(project);
            if let Some(frame) = layer.get_frame_at(project, time) {
                if onion_before != 0 {
                    let mut curr_time = frame.get(project).time;
                    for i in 0..onion_before {
                        if let Some(frame) = layer.get_frame_before(project, curr_time) {
                            for stroke in &frame.get(project).strokes {
                                if stroke.get(&project).filled {
                                    continue;
                                }
                                onion_before_strokes[i as usize].push(stroke.make_ptr());
                            }
                            
                            curr_time = frame.get(project).time;
                        }
                    }
                }
                if onion_after != 0 {
                    let mut curr_time = frame.get(project).time;
                    for i in 0..onion_after {
                        if let Some(frame) = layer.get_frame_after(project, curr_time) {
                            for stroke in &frame.get(project).strokes {
                                if stroke.get(project).filled {
                                    continue;
                                }
                                onion_after_strokes[i as usize].push(stroke.make_ptr());
                            }
                            curr_time = frame.get(project).time;
                        }
                    }
                }
            }
        }

        let initial_alpha = 0.75;
        let alpha_decay = 0.8 as f32;

        let mut alpha = initial_alpha * alpha_decay.powi(onion_before - 1);
        for strokes in onion_before_strokes.iter().rev() {
            for stroke in strokes {
                self.render_stroke(project, *stroke, Some(vec4(1.0, 0.3, 1.0, alpha)), gl);
            }
            alpha /= alpha_decay;
        }

        let mut alpha = initial_alpha * alpha_decay.powi(onion_before - 1);
        for strokes in onion_after_strokes.iter().rev() {
            for stroke in strokes {
                self.render_stroke(project, *stroke, Some(vec4(0.3, 1.0, 1.0, alpha)), gl);
            }
            alpha /= alpha_decay;
        }

    }

    fn render_picking<'a, I>(&mut self, project: &Project, time: i32, layer_iter: I, color_key_map: &mut Vec<ObjPtr<Stroke>>, gl: &Arc<Context>) where I: Iterator<Item = &'a &'a ObjBox<Layer>> {
        for layer in layer_iter { 
            let layer = layer.get(project);
            if let Some(frame) = layer.get_frame_at(project, time) {
                for stroke in &frame.get(project).strokes  {
                    let mut color = 0 as u32;
                    for i in 0..color_key_map.len() {
                        if color_key_map[i] == stroke.make_ptr() {
                            color = i as u32;
                            break;
                        } 
                    };
                    if color == 0 {
                        color_key_map.push(stroke.make_ptr());
                        color = color_key_map.len() as u32;
                    }
                    let bytes = color.to_le_bytes();
                    let r = (bytes[0] as f32) / 255.0;
                    let g = (bytes[1] as f32) / 255.0;
                    let b = (bytes[2] as f32) / 255.0;

                    self.render_stroke(project, stroke.make_ptr(), Some(glam::vec4(r, g, b, 1.0)), gl);
                }
            }
        }
    }

    fn get_shown_layers<'a>(&mut self, project: &'a Project, layers: &'a Vec<ObjBox<Layer>>, res_layers: &mut Vec<&'a ObjBox<Layer>>) {
        for layer_box in layers.iter().rev() {
            let layer = layer_box.get(project); 
            if !layer.show {
                continue;
            }
            if layer.kind == LayerKind::Animation {
                res_layers.push(layer_box);
            } else {
                self.get_shown_layers(project, &layer.layers, res_layers);
            }
        }
    }

    fn render_layer_contents_with_blending<F>(&mut self, project: &Project, w: u32, h: u32,  layer: &Layer, prev_fb: &Framebuffer, render_contents: F, gl: &Arc<Context>) where F: FnOnce(&mut SceneRenderer, &Project, &Layer, &Framebuffer) {
        if layer.requires_offscreen_render() {
            let mut bottom = self.framebuffers.alloc(w, h, gl);
            bottom.copy_from(prev_fb, gl);

            let top = self.framebuffers.alloc(w, h, gl);
            top.render_to(gl);
            unsafe {
                gl.clear_color(0.0, 0.0, 0.0, 0.0);
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            }

            render_contents(self, project, layer, &top);

            prev_fb.render_to(gl);
            let blend_shader = layer.blending.get_shader(self);
            unsafe {
                gl.active_texture(glow::TEXTURE0);
                gl.bind_texture(glow::TEXTURE_2D, Some(top.color));
                gl.active_texture(glow::TEXTURE1);
                gl.bind_texture(glow::TEXTURE_2D, Some(bottom.color));
            }
            blend_shader.enable(gl);
            blend_shader.set_int("uTopLayer", 0, gl);
            blend_shader.set_int("uBottomLayer", 1, gl);
            blend_shader.set_float("uLayerAlpha", layer.alpha, gl);
            unsafe {
                gl.disable(glow::DEPTH_TEST);
            }
            self.screen_quad.render(gl);
            self.flat_color_shader.enable(gl);
        } else {
            render_contents(self, project, layer, prev_fb);
        }
    }
    
    fn render_layer(&mut self, project: &Project, w: u32, h: u32, time: i32, layer: &Layer, prev_fb: &Framebuffer, gl: &Arc<Context>) {
        self.render_layer_contents_with_blending(project, w, h, layer, prev_fb, |renderer: &mut SceneRenderer, project: &Project, layer: &Layer, _: &Framebuffer| {
            if let Some(frame) = layer.get_frame_at(project, time) {
                for stroke in &frame.get(project).strokes {
                    renderer.render_stroke(project, stroke.make_ptr(), None, gl);
                }
            }
        }, gl);
    }

    fn render_layers(&mut self, project: &Project, w: u32, h: u32, time: i32, layers: &Vec<ObjBox<Layer>>, prev_fb: &Framebuffer, gl: &Arc<Context>) { 
        for layer in layers.iter().rev() {
            let layer = layer.get(project);
            if !layer.show {
                continue;
            }
            if layer.kind == LayerKind::Animation {
                self.render_layer(project, w, h, time, layer, prev_fb, gl); 
            } else if layer.kind == LayerKind::Group {
                self.render_layer_contents_with_blending(project, w, h, layer, prev_fb, |renderer: &mut SceneRenderer, project: &Project, layer: &Layer, fb: &Framebuffer| {
                    renderer.render_layers(project, w, h, time, &layer.layers, fb, gl);
                }, gl);
            }
        }
    }

    pub fn render(
        &mut self,

        fb: &mut Framebuffer,
        fb_pick: Option<(&mut Framebuffer, &mut Vec<ObjPtr<Stroke>>)>,
        w: u32,
        h: u32,

        cam_pos: glam::Vec2,
        cam_size: f32,

        project: &mut Project,
        gfx: ObjPtr<Graphic>,
        time: i32,

        onion_before: i32,
        onion_after: i32,

        gl: &Arc<Context>
    ) -> Option<glam::Mat4> {

        for stroke in &project.remeshes_needed {
            if let Some(mesh) = self.stroke_meshes.remove(stroke) {
                mesh.delete(gl);
            }
        }
        project.remeshes_needed.clear();

        fb.resize(w, h, gl);
        fb.render_to(gl);

        unsafe {
            gl.clear_color(1.0, 1.0, 1.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            gl.enable(glow::BLEND);
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
        }

        let aspect = (w as f32) / (h as f32);
        let proj = glam::Mat4::orthographic_rh_gl(-aspect * cam_size, aspect * cam_size, -cam_size, cam_size, -1.0, 1.0);
        let view = glam::Mat4::from_translation(-glam::vec3(cam_pos.x, cam_pos.y, 0.0));
        let proj_view = proj * view;
        self.flat_color_shader.enable(gl);
        self.flat_color_shader.set_mat4("uTrans", &proj_view, gl);

        let gfx = project.graphics.get(gfx)?;
        let mut layers = Vec::new();
        self.get_shown_layers(&project, &gfx.layers, &mut layers);
        let layer_iter = layers.iter();
        
        self.render_onion_skin(project, time, onion_before, onion_after, layer_iter.clone(), gl);

        self.render_layers(project, w, h, time, &gfx.layers, fb, gl);
       
        if let Some((fb_pick, color_key_map)) = fb_pick {
            fb_pick.resize(w, h, gl);
            fb_pick.render_to(gl);
            color_key_map.clear();

            unsafe {
                gl.clear_color(0.0, 0.0, 0.0, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT);
            }
            
            self.render_picking(project, time, layer_iter.clone(), color_key_map, gl);
        }
        fb.render_to(gl);

        Some(proj_view) 
        
    }

}

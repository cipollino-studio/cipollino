
use std::{sync::{Arc, Mutex}, cell::RefCell, rc::Rc};
use glow::HasContext;

use crate::{
    editor::EditorState,
    renderer::{fb::Framebuffer, mesh::Mesh, shader::Shader, scene::SceneRenderer}, project::Project, util::curve,
};

use super::tools::Tool;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ScenePanel {
    #[serde(skip)]
    fb: Arc<Mutex<Option<Framebuffer>>>,

    #[serde(skip)]
    cam_pos: glam::Vec2,
    #[serde(default)]
    cam_size: f32,
}

impl Default for ScenePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ScenePanel {
    pub fn new() -> Self {
        Self {
            fb: Arc::new(Mutex::new(None)),
            cam_pos: glam::vec2(0.0, 0.0),
            cam_size: 5.0,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        let gfx = state.open_graphic();
        let no_margin = egui::Frame {
            inner_margin: egui::Margin::same(0.0),
            ..Default::default()
        };
        if let Some(_gfx) = gfx {
            egui::SidePanel::new(egui::panel::Side::Left, "toolbar")
                .resizable(false)
                .exact_width(34.0)
                .frame(no_margin)
                .show_inside(ui, |ui| {
                    if ui
                        .button(
                            egui::RichText::new(format!("{}", egui_phosphor::regular::CURSOR))
                                .size(20.0),
                        )
                        .clicked()
                    {
                        state.curr_tool = state.select.clone();
                    }
                    if ui
                        .button(
                            egui::RichText::new(format!("{}", egui_phosphor::regular::PENCIL))
                                .size(20.0),
                        )
                        .clicked()
                    {
                        state.curr_tool = state.pencil.clone();
                    }
                });
            egui::CentralPanel::default()
                .frame(no_margin)
                .show_inside(ui, |ui| {
                    egui::Frame::canvas(ui.style()).show(ui, |ui| {
                        let (rect, response) = ui.allocate_exact_size(
                            ui.available_size(),
                            egui::Sense::click_and_drag(),
                        );

                        let gl_ctx_copy = state.renderer.gl_ctx.clone();
                        let fb_copy = self.fb.clone();

                        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
                            let gl = painter.gl();
                            *gl_ctx_copy.lock().unwrap() = Some(gl.clone());
                            if let Some(fb) = fb_copy.lock().unwrap().as_ref() {
                                // TODO: this is scuffed and inneficient
                                let mut shader = Shader::new(
                                    "
                                    #version 100\n 

                                    attribute vec2 aPos;

                                    varying vec2 uv;
                            
                                    void main() {
                                        gl_Position = vec4(aPos, 0.0, 1.0);                
                                        uv = aPos;
                                    } 
                                ",
                                    "
                                    #version 100\n 

                                    uniform sampler2D tex;
                                    varying mediump vec2 uv;

                                    void main() {
                                        gl_FragColor = texture2D(tex, uv / 2.0 + 0.5); 
                                    }
                                ",
                                    gl,
                                );

                                let mut quad = Mesh::new(vec![2], gl);
                                quad.upload(
                                    &vec![-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0],
                                    &vec![0, 1, 2, 1, 2, 3],
                                    gl,
                                );

                                shader.enable(gl);
                                shader.set_int("tex", 1, gl);
                                unsafe {
                                    gl.active_texture(glow::TEXTURE1);
                                    gl.bind_texture(glow::TEXTURE_2D, Some(fb.color));
                                }
                                quad.render(gl);

                                shader.delete(gl);
                                quad.delete(gl);
                            }
                        });

                        self.render_scene(state, &rect, &response, ui, state.open_graphic);

                        let callback = egui::PaintCallback {
                            rect,
                            callback: Arc::new(cb),
                        };
                        ui.painter().add(callback);
                    });
                });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No Graphic Open");
            });
        }
    }

    fn render_scene(&mut self, state: &mut EditorState, rect: &egui::Rect, response: &egui::Response, ui: &mut egui::Ui, gfx_key: u64) {
        if let Some(mouse_pos) = response.hover_pos() {
            let mouse_pos = self.cam_size * (mouse_pos - rect.center()) / (rect.height() * 0.5);
            let mouse_pos = glam::vec2(mouse_pos.x, -mouse_pos.y) + self.cam_pos;

            let zoom_fac =
                (1.05 as f32).powf(-ui.input(|i| i.scroll_delta.y.clamp(-6.0, 6.0) * 0.7));
            let next_cam_size = self.cam_size * zoom_fac;
            if next_cam_size > 0.02 && next_cam_size < 1000.0 {
                self.cam_pos -= (mouse_pos - self.cam_pos) * (zoom_fac - 1.0);
                self.cam_size = next_cam_size;
            }

            let tool = state.curr_tool.clone();
            if response.drag_started() {
                tool.borrow_mut().mouse_click(mouse_pos, state);
            }
            if response.dragged() {
                tool.borrow_mut().mouse_down(mouse_pos, state);
            }
            if response.drag_released() {
                tool.borrow_mut().mouse_release(mouse_pos, state, ui);
            }
            if response.hovered() {
                ui.ctx().output_mut(|o| {
                    let tool = state.curr_tool.clone();
                    o.cursor_icon = tool.borrow_mut().mouse_cursor(mouse_pos, state);
                });
            }
        }

        let frame = state.frame();
        state.renderer.use_renderer(|gl, renderer| {
            let mut fb = self.fb.lock().unwrap();
            if let None = fb.as_ref() {
                *fb = Some(Framebuffer::new(100, 100, gl));
            }
            let fb = fb.as_mut().unwrap();

            if let Some(proj_view) = renderer.render(
                fb,
                (rect.width() as u32) * 4,
                (rect.height() as u32) * 4,
                self.cam_pos,
                self.cam_size,
                &mut state.project,
                gfx_key,
                frame,
                if state.playing { 0 } else { state.onion_before },
                if state.playing { 0 } else { state.onion_after },
                gl,
            ) {
                self.render_overlays(&mut state.project, state.curr_tool.clone(), gfx_key, renderer, gl, proj_view, &state.selected_strokes);
            }

            Framebuffer::render_to_win(
                ui.ctx().screen_rect().width() as u32,
                ui.ctx().screen_rect().height() as u32,
                gl,
            );
        });
        
    }

    fn render_overlays(&self, project: &mut Project, curr_tool: Rc<RefCell<dyn Tool>>, gfx_key: u64, renderer: &mut SceneRenderer, gl: &Arc<glow::Context>, proj_view: glam::Mat4, selected_strokes: &Vec<u64>) {
        renderer.line_shader.enable(gl);
        renderer
            .line_shader
            .set_vec4("uColor", glam::vec4(0.0, 0.0, 0.0, 0.3), gl);
        let mut draw_quad = |left: f32, right: f32, top: f32, bottom: f32| {
            let center = glam::vec3((left + right) / 2.0, (top + bottom) / 2.0, 0.0);
            let scale = glam::vec3(right - left, top - bottom, 1.0);
            let model =
                glam::Mat4::from_translation(center) * glam::Mat4::from_scale(scale);
            let trans = proj_view * model;
            renderer.line_shader.set_mat4("uTrans", &trans, gl);
            renderer.quad.render(gl);
        };

        let gfx = project.graphics.get(&gfx_key).unwrap();
        if gfx.data.clip {
            let cam_top = 5.0;
            let cam_btm = -cam_top;
            let cam_right = cam_top * ((gfx.data.w as f32) / (gfx.data.h as f32));
            let cam_left = -cam_right;
            let huge = 100000.0;
            draw_quad(cam_left, cam_right, huge, cam_top);
            draw_quad(cam_left, cam_right, cam_btm, -huge);
            draw_quad(-huge, cam_left, cam_top, cam_btm);
            draw_quad(cam_right, huge, cam_top, cam_btm);
            draw_quad(-huge, cam_left, huge, cam_top);
            draw_quad(cam_right, huge, huge, cam_top);
            draw_quad(cam_right, huge, cam_btm, -huge);
            draw_quad(-huge, cam_left, cam_btm, -huge);
        }

        let mut overlay = OverlayRenderer::new(renderer, gl, proj_view, self.cam_size);
        curr_tool.borrow_mut().draw_overlay(&mut overlay); 
        for stroke in selected_strokes {
            if let Some(stroke) = project.strokes.get(stroke) {
                for (p0, p1) in stroke.iter_point_pairs() {
                    let p0 = project.points.get(&p0).unwrap(); 
                    let p1 = project.points.get(&p1).unwrap(); 
                    for i in 0..100 {
                        let t = (i as f32) / 100.0;
                        overlay.line(
                            curve::sample(t, p0.data.pt, p0.data.b, p1.data.a, p1.data.pt),
                            curve::sample(t + 0.01, p0.data.pt, p0.data.b, p1.data.a, p1.data.pt),
                            glam::vec4(0.0, 1.0, 1.0, 1.0) 
                        );
                    }
                }
            }
        }

    }

}

pub struct OverlayRenderer<'a> {
    renderer: &'a mut SceneRenderer,
    gl: &'a Arc<glow::Context>,
    proj_view: glam::Mat4,
    cam_size: f32
}

impl<'a> OverlayRenderer<'a> {

    pub fn new(renderer: &'a mut SceneRenderer, gl: &'a Arc<glow::Context>, proj_view: glam::Mat4, cam_size: f32) -> Self {
        Self {
            renderer,
            gl,
            proj_view,
            cam_size
        }
    }

    pub fn line(&mut self, p0: glam::Vec2, p1: glam::Vec2, color: glam::Vec4) {
        let center = (p0 + p1) / 2.0;
        let len = (p1 - p0).length();
        let scale = glam::vec3(len, 0.004 * self.cam_size, 1.0);
        let angle = glam::vec2(1.0, 0.0).angle_between(p1 - p0);
        let model = glam::Mat4::from_translation(glam::vec3(center.x, center.y, 0.0)) * glam::Mat4::from_axis_angle(glam::vec3(0.0, 0.0, 1.0), angle) * glam::Mat4::from_scale(scale);
        let trans = self.proj_view * model; 
        self.renderer.line_shader.set_mat4("uTrans", &trans, self.gl);
        self.renderer.line_shader.set_vec4("uColor", color, self.gl);
        self.renderer.quad.render(self.gl);
    }

}
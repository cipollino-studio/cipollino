
use std::sync::{Arc, Mutex};

use glow::HasContext;

use crate::{editor::EditorState, renderer::{mesh::Mesh, shader::Shader, fb::Framebuffer}};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ScenePanel {
    #[serde(skip)]
    fb: Arc<Mutex<Option<Framebuffer>>>,

    #[serde(skip)]
    cam_pos: glam::Vec2,
    #[serde(default)]
    cam_size: f32
}

impl Default for ScenePanel {

    fn default() -> Self {
        println!("HEY");
        Self::new() 
    }
    
}

impl ScenePanel {

    pub fn new() -> Self {
        Self {
            fb: Arc::new(Mutex::new(None)),
            cam_pos: glam::vec2(0.0, 0.0),
            cam_size: 5.0
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        let gfx = state.open_graphic.and_then(|key| state.project.graphics.get(&key));
        if let Some(_gfx) = gfx {
            let gfx_key = state.open_graphic.unwrap(); 
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
        
                let gl_ctx_copy = state.renderer.gl_ctx.clone();
                let fb_copy = self.fb.clone();

                let cb = egui_glow::CallbackFn::new(move |_info, painter| {
                    let gl = painter.gl();
                    *gl_ctx_copy.lock().unwrap() = Some(gl.clone());
                    if let Some(fb) = fb_copy.lock().unwrap().as_ref() {

                        // TODO: this is scuffed and inneficient
                        let mut shader = Shader::new("
                            #version 100\n 

                            attribute vec2 aPos;

                            varying vec2 uv;
                    
                            void main() {
                                gl_Position = vec4(aPos, 0.0, 1.0);                
                                uv = aPos;
                            } 
                        ", "
                            #version 100\n 

                            uniform sampler2D tex;
                            varying mediump vec2 uv;

                            void main() {
                                gl_FragColor = texture2D(tex, uv / 2.0 + 0.5); 
                            }
                        ", gl);

                        let mut quad = Mesh::new(vec![2], gl);
                        quad.upload(&vec![
                            -1.0, -1.0,
                             1.0, -1.0,
                            -1.0,  1.0,
                             1.0,  1.0
                        ], &vec![
                            0, 1, 2,
                            1, 2, 3
                        ], gl);

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

                if let Some(mouse_pos) = response.hover_pos() {
                    let mouse_pos = self.cam_size * (mouse_pos - rect.center()) / (rect.height() * 0.5);
                    let mouse_pos = glam::vec2(mouse_pos.x, -mouse_pos.y) + self.cam_pos; 

                    let zoom_fac = (1.05 as f32).powf(-ui.input(|i| i.scroll_delta.y * 0.7));
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
                        tool.borrow_mut().mouse_release(mouse_pos, state);
                    }
                }

                let frame = state.frame();
                state.renderer.use_renderer(|gl, renderer| {
                    let mut fb = self.fb.lock().unwrap(); 
                    if let None = fb.as_ref() {
                        *fb = Some(Framebuffer::new(100, 100, gl));
                    }
                    let fb = fb.as_mut().unwrap();

                    renderer.render(
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
                        gl
                    );

                    Framebuffer::render_to_win(ui.ctx().screen_rect().width() as u32, ui.ctx().screen_rect().height() as u32, gl);
                });
                
                let callback = egui::PaintCallback {
                    rect,
                    callback: Arc::new(cb)
                };
                ui.painter().add(callback);

            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No Graphic Open");
            });
        }
    }

}

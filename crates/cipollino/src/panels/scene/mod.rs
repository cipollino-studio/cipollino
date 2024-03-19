
use std::sync::{Arc, Mutex};
use glam::Vec2;
use glow::HasContext;

pub mod overlay;

use crate::{
    editor::{clipboard::Clipboard, selection::Selection, state::{EditorRenderer, EditorState}}, project::{action::Action, graphic::Graphic, obj::{child_obj::ChildObj, ObjPtr}, stroke::{Stroke, StrokeColor}}, renderer::fb::Framebuffer, util::ui::color::color_picker
};

use super::super::tools::active_frame_proj_layer_frame;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ScenePanel {
    #[serde(skip)]
    fb: Arc<Mutex<Option<Framebuffer>>>,
    #[serde(skip)]
    fb_pick: Arc<Mutex<Option<Framebuffer>>>,
    #[serde(skip)]
    color_key_map: Vec<ObjPtr<Stroke>>, 

    #[serde(skip)]
    prev_mouse_down: bool,

    #[serde(skip)]
    pub cam_pos: glam::Vec2,
    #[serde(skip)]
    pub cam_size: f32,
    #[serde(skip)]
    pub cam_aspect : f32
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
            fb_pick: Arc::new(Mutex::new(None)),
            color_key_map: Vec::new(),
            prev_mouse_down: false,
            cam_pos: glam::vec2(0.0, 0.0),
            cam_size: 600.0,
            cam_aspect: 1.0
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState, renderer: &mut EditorRenderer) {
        // Hack to get around serde's default stuff
        if self.cam_size == 0.0 {
            self.cam_size = 600.0; 
        }

        let no_margin = egui::Frame {
            inner_margin: egui::Margin::same(0.0),
            ..Default::default()
        };
        let no_gfx_open = state.project.graphics.get(state.open_graphic).is_none();
        if no_gfx_open {
            ui.centered_and_justified(|ui| {
                ui.label("No Graphic Open");
            });
            return;
        } else {
            egui::SidePanel::new(egui::panel::Side::Left, ui.next_auto_id())
                .resizable(false)
                .exact_width(35.0)
                .frame(no_margin)
                .show_inside(ui, |ui| {
                    self.toolbar(state, ui);
                });
            let response = egui::CentralPanel::default()
                .frame(no_margin)
                .show_inside(ui, |ui| {
                    egui::Frame::canvas(ui.style()).show(ui, |ui| {
                        self.draw_scene_to_ui(state, ui, renderer);
                    });
                }).response;

            if response.clicked_elsewhere() && state.selection.is_scene() {
                state.selection.clear();
                state.reset_tool();
            }
        }

        // Deleting strokes
        let delete_shortcut = state.delete_shortcut();
        if let Selection::Scene(strokes) = &mut state.selection {
            if ui.input_mut(|i| i.consume_shortcut(&delete_shortcut)) {
                let mut action = Action::new();
                for stroke_ptr in strokes {
                    if let Some(act) = Stroke::delete(&mut state.project, *stroke_ptr) {
                        action.add(act);
                    }
                }
                state.actions.add(action);
                state.selection.clear();
                state.reset_tool();
            }
        }

        // Pasting strokes
        if let Clipboard::Scene(strokes) = &state.clipboard {
            if state.just_pasted { 
                let frame = state.frame();
                if let Some((frame, act)) = active_frame_proj_layer_frame(&mut state.project, state.active_layer, frame) {
                    state.selection.clear();
                    let mut acts = Vec::new();
                    if let Some(act) = act {
                        acts.push(act);
                    }
                    for stroke in strokes {
                        Stroke::transform(&mut state.project, stroke.make_ptr(), glam::Mat4::from_translation(glam::vec3(20.0, 0.0, 0.0)));
                        if let Some(clone) = stroke.make_ptr().make_obj_clone(&mut state.project) {
                            if let Some((stroke, act)) = Stroke::add(&mut state.project, frame, clone) {
                                acts.push(act);
                                state.selection.select_stroke_inverting(stroke);
                            }
                        }
                    }
                    state.actions.add(Action::from_list(acts));
                    state.reset_tool();
                }
            }
        }

    }

    fn toolbar(&mut self, state: &mut EditorState, ui: &mut egui::Ui) {

        let toolbar_button_size = ui.available_width() - ui.spacing().window_margin.right;

        let mut new_tool = None; 
        for tool_rc in &state.tools {
            let tool = tool_rc.read().unwrap();
            let resp = ui.add_enabled(
                tool.name() != state.curr_tool.read().unwrap().name(),
                egui::Button::new(egui::RichText::new(format!("{}", tool.get_icon())).size(toolbar_button_size - 10.0)).min_size(egui::Vec2::splat(toolbar_button_size)));
            let shortcut = tool.shortcut();
            let resp = resp.on_hover_text(format!("{} ({})", tool.name(), ui.ctx().format_shortcut(&shortcut)));
            if resp.clicked() || ui.input_mut(|i| i.consume_shortcut(&shortcut)) {
                new_tool = Some(tool_rc.clone());
            }
        }
        if let Some(tool) = new_tool {
            state.reset_tool();
            state.curr_tool = tool;
        }

        let mut color = state.color.get_color(&state.project);
        let mut editing_color = false;
        let mut _set_color = false;
        color_picker(ui, &mut color, Some(egui::Vec2::splat(toolbar_button_size)), &mut editing_color, &mut _set_color);
        if editing_color {
            state.color = StrokeColor::Color(color);
        }
        
    }

    fn draw_scene_to_ui(&mut self, state: &mut EditorState, ui: &mut egui::Ui, renderer: &mut EditorRenderer) {
        let (rect, response) = ui.allocate_exact_size(
            ui.available_size(),
            egui::Sense::click_and_drag(),
        );

        let fb_copy = self.fb.clone();
        let screen_quad_copy = renderer.renderer.screen_quad.clone();
        let screen_shader_copy = renderer.renderer.screen_shader.clone();

        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
            let gl = painter.gl();
            let mut screen_shader_copy = screen_shader_copy.clone();
            if let Some(fb) = fb_copy.lock().unwrap().as_ref() {
                screen_shader_copy.enable(gl);
                screen_shader_copy.set_int("uTex", 1, gl);
                unsafe {
                    gl.active_texture(glow::TEXTURE1);
                    gl.bind_texture(glow::TEXTURE_2D, Some(fb.color));
                }
                screen_quad_copy.render(gl);
            }
        });

        self.render_scene(state, &rect, &response, ui, state.open_graphic, renderer);

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };

        ui.painter().add(callback);
    }

    fn render_scene(&mut self, state: &mut EditorState, rect: &egui::Rect, response: &egui::Response, ui: &mut egui::Ui, gfx: ObjPtr<Graphic>, renderer: &mut EditorRenderer) {

        // Tool interaction
        if let Some(mouse_pos) = response.hover_pos() {
            let mouse_pos = self.cam_size * (mouse_pos - rect.center()) / (rect.height() * 0.5);
            let mouse_pos = glam::vec2(mouse_pos.x, -mouse_pos.y) + self.cam_pos;
            self.cam_aspect = rect.aspect_ratio();

            let zoom_fac =
                (1.05 as f32).powf(-ui.input(|i| i.smooth_scroll_delta.y.clamp(-4.0, 4.0) * 0.7));
            let next_cam_size = (self.cam_size * zoom_fac).clamp(10.0, 5000.0);
            let zoom_fac = next_cam_size / self.cam_size;
            self.cam_pos -= (mouse_pos - self.cam_pos) * (zoom_fac - 1.0);
            self.cam_size = next_cam_size;

            let tool = state.curr_tool.clone();
            let mouse_down = response.is_pointer_button_down_on() || response.clicked();
            if mouse_down && !self.prev_mouse_down {
                tool.write().unwrap().mouse_click(mouse_pos, state, ui, self, renderer.gl);
            }
            if mouse_down && self.prev_mouse_down {
                tool.write().unwrap().mouse_down(mouse_pos, state, self);
            }
            if !mouse_down && self.prev_mouse_down {
                tool.write().unwrap().mouse_release(mouse_pos, state, ui, self, renderer.gl);
            }
            self.prev_mouse_down = mouse_down;
            if response.hovered() {
                ui.ctx().output_mut(|o| {
                    let tool = state.curr_tool.clone();
                    o.cursor_icon = tool.write().unwrap().mouse_cursor(mouse_pos, state, self, renderer.gl);
                });
            }
        }

        // Render scene to framebuffer
        let frame = state.frame();
        let mut fb = self.fb.lock().unwrap();
        let mut fb_pick = self.fb_pick.lock().unwrap();
        if let None = fb.as_ref() {
            *fb = Some(Framebuffer::new(100, 100, renderer.gl));
            *fb_pick = Some(Framebuffer::new(100, 100, renderer.gl));
        }
        let fb = fb.as_mut().unwrap();
        let fb_pick = fb_pick.as_mut().unwrap();

        if let Some(proj_view) = renderer.renderer.render(
            fb,
            Some((fb_pick, &mut self.color_key_map)),
            (rect.width() as u32) * 2,
            (rect.height() as u32) * 2,
            self.cam_pos,
            self.cam_size,
            &mut state.project,
            gfx,
            frame,
            if state.playing { 0 } else { state.onion_before },
            if state.playing { 0 } else { state.onion_after },
            renderer.gl,
        ) {
            self.render_overlays(gfx, renderer, proj_view, state);
        }

        Framebuffer::render_to_win(
            ui.ctx().screen_rect().width() as u32,
            ui.ctx().screen_rect().height() as u32,
            renderer.gl,
        );

    }

    pub fn sample_pick(&mut self, pos: Vec2, gl: &Arc<glow::Context>) -> Option<ObjPtr<Stroke>> {
        if let Some(fb_pick) = self.fb_pick.lock().unwrap().as_ref() {

            let h_cam_size = self.cam_size * (fb_pick.w as f32) / (fb_pick.h as f32);
            let left = self.cam_pos.x - h_cam_size;
            let right = self.cam_pos.x + h_cam_size;
            let top = self.cam_pos.y + self.cam_size;
            let bottom = self.cam_pos.y - self.cam_size;

            let x = (pos.x - left) / (right - left);
            let y = (pos.y - bottom) / (top - bottom);

            let px = (x * (fb_pick.w as f32)) as i32;
            let py = (y * (fb_pick.h as f32)) as i32;

            if px < 0 || py < 0 || px >= fb_pick.w as i32 || py >= fb_pick.h as i32 {
                return None;
            }

            let mut pixel_data = [0; 4];
            unsafe {
                gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(fb_pick.fbo));
                gl.read_pixels(px, py, 1, 1, glow::RGB, glow::UNSIGNED_BYTE, glow::PixelPackData::Slice(&mut pixel_data[0..3]));
            }
            let color = u32::from_le_bytes(pixel_data);
            if color > 0 && color <= self.color_key_map.len() as u32 {
                return Some(self.color_key_map[color as usize - 1]);
            }
        }
        None
    }

}

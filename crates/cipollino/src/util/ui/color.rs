
use egui::{Area, Response};
use glam::vec4;
use crate::util::color::{contrast_color, hsva_to_rgba, rgba_to_hsva, VALUE_TOLERANCE};

const COLOR_SLIDER_WIDTH: f32 = 275.0;
const COLOR_SLIDER_HEIGHT: f32 = 20.0;
const COLOR_SLIDER_RES: u32 = 36;

pub fn to_egui_color(color: glam::Vec4) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied((color.x * 255.0) as u8, (color.y * 255.0) as u8, (color.z * 255.0) as u8, (color.w * 255.0) as u8)
}

pub fn color_button(ui: &mut egui::Ui, color: glam::Vec4, size: egui::Vec2) -> Response {
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let visuals = ui.style().interact(&resp);
    let color = to_egui_color(color);
    ui.painter().rect(rect, visuals.rounding.at_most(2.0), color, (2.0, visuals.bg_fill));
    resp
}

pub fn color_sv_slider(ui: &mut egui::Ui, color: &mut glam::Vec4, editing_color: &mut bool, set_color: &mut bool) {

    let (rect, resp) = ui.allocate_at_least(egui::Vec2::splat(COLOR_SLIDER_WIDTH), egui::Sense::click_and_drag());

    let color_hsva = rgba_to_hsva(*color);
    let mut mesh = egui::Mesh::default();
    for xi in 0..=COLOR_SLIDER_RES {
        for yi in 0..=COLOR_SLIDER_RES {
            let xt = xi as f32 / (COLOR_SLIDER_RES as f32);
            let yt = yi as f32 / (COLOR_SLIDER_RES as f32);
            let color = hsva_to_rgba(vec4(color_hsva.x, xt, yt, 1.0));
            let x = egui::emath::lerp(rect.left()..=rect.right(), xt);
            let y = egui::emath::lerp(rect.bottom()..=rect.top(), yt);
            mesh.colored_vertex(egui::pos2(x, y), to_egui_color(color));
            if xi < COLOR_SLIDER_RES && yi < COLOR_SLIDER_RES {
                let x_offset = 1;
                let y_offset = COLOR_SLIDER_RES + 1;
                let tl = yi * y_offset + xi;
                mesh.add_triangle(tl, tl + x_offset, tl + y_offset);
                mesh.add_triangle(tl + x_offset, tl + y_offset, tl + y_offset + x_offset);
            }
        }
    }
    ui.painter().add(egui::Shape::mesh(mesh));

    if let Some(mouse_pos) = resp.interact_pointer_pos() {
        let x_val = egui::emath::remap_clamp(mouse_pos.x, rect.left()..=rect.right(), 0.0..=1.0); 
        let y_val = egui::emath::remap_clamp(mouse_pos.y, rect.bottom()..=rect.top(), 0.0..=1.0); 
        let new_col_hsva = vec4(color_hsva.x, x_val, y_val, color_hsva.w);
        *color = hsva_to_rgba(new_col_hsva);
        *editing_color = true;
    }
    if resp.drag_released() || resp.clicked() {
        *set_color = true;
    }

    let color_hsva = rgba_to_hsva(*color);
    let x = egui::emath::lerp(rect.left()..=rect.right(), color_hsva.y);
    let y = egui::emath::lerp(rect.bottom()..=rect.top(), color_hsva.z);
    ui.painter().circle(egui::pos2(x, y), 5.0, to_egui_color(*color), egui::Stroke::new(1.0, to_egui_color(contrast_color(*color))));

}

pub fn color_h_slider(ui: &mut egui::Ui, color: &mut glam::Vec4, editing_color: &mut bool, set_color: &mut bool) {
    
    let (rect, resp) = ui.allocate_at_least(egui::vec2(COLOR_SLIDER_WIDTH, COLOR_SLIDER_HEIGHT), egui::Sense::click_and_drag());

    let mut mesh = egui::Mesh::default();
    for i in 0..=COLOR_SLIDER_RES {
        let t = (i as f32) / (COLOR_SLIDER_RES as f32);
        let color = hsva_to_rgba(vec4(t, 1.0, 1.0, 1.0));
        let egui_color = to_egui_color(color);
        let x = egui::emath::lerp(rect.left()..=rect.right(), t);
        mesh.colored_vertex(egui::pos2(x, rect.top()), egui_color); 
        mesh.colored_vertex(egui::pos2(x, rect.bottom()), egui_color); 
        if i < COLOR_SLIDER_RES {
            mesh.add_triangle(2 * i + 0, 2 * i + 1,  2 * i + 2);
            mesh.add_triangle(2 * i + 1, 2 * i + 2,  2 * i + 3);
        }
    }
    ui.painter().add(egui::Shape::mesh(mesh));

    let color_hsva = rgba_to_hsva(*color);
    if let Some(mouse_pos) = resp.interact_pointer_pos() {
        let x_val = egui::emath::remap_clamp(mouse_pos.x, rect.left()..=rect.right(), 0.0..=0.999);
        *color = hsva_to_rgba(vec4(x_val, color_hsva.y, color_hsva.z, color_hsva.w));
        *editing_color = true;
    } 
    if resp.clicked() || resp.drag_released() {
        *set_color = true;
    }
    let color_hsva = rgba_to_hsva(*color);

    let slider_x = egui::emath::lerp(rect.left()..=rect.right(), (color_hsva.x + 2.0).fract());
    let r = rect.height() / 4.0;
    ui.painter().add(egui::Shape::convex_polygon(vec![
        egui::pos2(slider_x, rect.center().y),
        egui::pos2(slider_x + r, rect.bottom()),
        egui::pos2(slider_x - r, rect.bottom()),
    ], to_egui_color(*color), egui::Stroke::new(2.0, to_egui_color(contrast_color(*color)))));


}

pub fn color_slider(ui: &mut egui::Ui, color: &mut glam::Vec4, editing_color: &mut bool, set_color: &mut bool) {

    color_sv_slider(ui, color, editing_color, set_color);
    ui.add_space(5.0);
    color_h_slider(ui, color, editing_color, set_color);

}

pub fn color_picker(ui: &mut egui::Ui, color: &mut glam::Vec4, size: Option<egui::Vec2>, editing_color: &mut bool, set_color: &mut bool) {

    *editing_color = false;
    *set_color = false;

    // Clamp value and saturation to ensure that hue is preserved
    let color_hsva = rgba_to_hsva(*color);
    if color_hsva.y < VALUE_TOLERANCE || color_hsva.z < VALUE_TOLERANCE {
        let new_color = vec4(color_hsva.x, color_hsva.y.max(VALUE_TOLERANCE), color_hsva.z.max(VALUE_TOLERANCE), color_hsva.w);
        *color = hsva_to_rgba(new_color);
        *editing_color = true;
    }

    let size = size.unwrap_or(ui.spacing().interact_size);
    
    let popup_id = ui.next_auto_id();
    let button_resp = color_button(ui, *color, size);
    if button_resp.clicked() {
        ui.memory_mut(|mem| mem.toggle_popup(popup_id));
    }

    if ui.memory(|mem| mem.is_popup_open(popup_id)) {
        let area_resp = Area::new(popup_id)
            .order(egui::Order::Foreground)
            .fixed_pos(button_resp.rect.max)
            .constrain(true)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    color_slider(ui, color, editing_color, set_color);
                });
            }).response;
        
        if !button_resp.clicked() && (ui.input(|i| i.key_pressed(egui::Key::Escape)) || area_resp.clicked_elsewhere()) {
            ui.memory_mut(|mem| mem.close_popup());
        }
    }

}

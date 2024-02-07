
use editor::Editor;

pub mod editor;
pub mod panels;
pub mod project;
pub mod renderer;
pub mod util;
pub mod export;

fn main() -> Result<(), eframe::Error> {
    let (icon, w, h) = {
        let img = image::load_from_memory(include_bytes!("../../../res/icon256x256.png")).unwrap().into_rgba8();
        let (w, h) = img.dimensions();
        let rgba = img.into_raw();
        (rgba, w, h)
    };

    let options = eframe::NativeOptions {
        icon_data: Some(eframe::IconData { rgba: icon, width: w, height: h }),
        initial_window_size: Some(egui::vec2(1280.0, 720.0)),
        multisampling: 4,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    
    eframe::run_native(
        "Cipollino",
        options,
        Box::new(|cc| Box::new(Cipollino::new(cc))),
    )
}

struct Cipollino {
    editor: Editor
}

impl Cipollino {

    pub fn new(cc: &eframe::CreationContext) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

        cc.egui_ctx.set_fonts(fonts);
        
        Self {
            editor: Editor::new()
        }
    }

}

impl eframe::App for Cipollino {

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.style_mut(|style| {
            style.interaction.tooltip_delay = 0.75;
        });
        self.editor.render(ctx, frame);
    }

}

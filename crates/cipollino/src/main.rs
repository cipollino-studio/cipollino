
use editor::Editor;
use renderer::scene::SceneRenderer;

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
    let icon_data = egui::IconData {
        rgba: icon,
        width: w,
        height: h,
    };

    let options = eframe::NativeOptions {
        multisampling: 4,
        renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default().with_title("Cipollino").with_maximized(true).with_icon(icon_data),
        ..Default::default()
    };
    
    eframe::run_native(
        "Cipollino",
        options,
        Box::new(|cc| Box::new(Cipollino::new(cc))),
    )
}

struct Cipollino {
    editor: Editor,
    scene_renderer: Option<SceneRenderer>,
}

impl Cipollino {

    pub fn new(cc: &eframe::CreationContext) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        let mut phosphor_font_data = egui::FontData::from_static(include_bytes!("../../../res/Phosphor.ttf"));
        phosphor_font_data.tweak.y_offset_factor = 0.1;

        fonts
            .font_data
            .insert("phosphor".into(),phosphor_font_data);

        if let Some(font_keys) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            font_keys.push("phosphor".into());
        }

        cc.egui_ctx.set_fonts(fonts);
        
        Self {
            editor: Editor::new(),
            scene_renderer: None
        }
    }

}

impl eframe::App for Cipollino {

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.style_mut(|style| {
            style.interaction.tooltip_delay = 0.75;
        });
        self.editor.render(ctx, frame, &mut self.scene_renderer);
    }

}

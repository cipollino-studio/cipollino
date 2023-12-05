
use editor::Editor;

pub mod editor;
pub mod panels;
pub mod project;
pub mod renderer;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1280.0, 720.0)),
        multisampling: 4,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Cipollino",
        options,
        Box::new(|_cc| Box::new(Cipollino::new())),
    )
}

struct Cipollino {
    editor: Editor
}

impl Cipollino {

    pub fn new() -> Self {
        Self {
            editor: Editor::new()
        }
    }

}

impl eframe::App for Cipollino {

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.editor.render(ctx, frame);
    }

}
use egui::WidgetText;


pub struct Toasts {
    toasts: egui_toast::Toasts,
}

impl Toasts {

    pub fn new() -> Self {
        Self {
            toasts: egui_toast::Toasts::new()
        }
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        self.toasts.show(ctx);
    }

    pub fn error_toast<T>(&mut self, message: T) where T: Into<WidgetText> {
        self.toasts.add(egui_toast::Toast {
            kind: egui_toast::ToastKind::Error,
            text: message.into(),
            options: egui_toast::ToastOptions::default().show_progress(false) 
        });
    }

}


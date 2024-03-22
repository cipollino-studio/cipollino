
use egui::{vec2, Align, Layout, Pos2, Rect, Ui};

pub struct ColumnLayout<'a> {
    top_left: Pos2,
    curr_pos: Pos2,
    max_height: f32,
    ui: &'a mut Ui
}

impl<'a> ColumnLayout<'a> {

    #[must_use = "must call .finish() on ColumnLayout."]
    pub fn new(ui: &'a mut Ui) -> Self {
        Self {
            top_left: ui.cursor().min,
            curr_pos: ui.cursor().min,
            max_height: 0.0,
            ui
        }
    }

    #[must_use = "must call .finish() on ColumnLayout."]
    pub fn add_column<F>(mut self, width: f32, add_contents: F) -> Self where F: FnOnce(&mut Ui) {
        let mut sub_ui = self.ui.child_ui(Rect {
            min: self.curr_pos,
            max: self.curr_pos + vec2(width, 0.0) 
        }, Layout::top_down_justified(Align::LEFT));
        sub_ui.set_width(width);
        add_contents(&mut sub_ui);
        self.curr_pos.x += sub_ui.min_rect().width() + self.ui.spacing().item_spacing.x;
        self.max_height = self.max_height.max(sub_ui.min_rect().height());

        self
    } 

    pub fn finish(mut self) {
        self.curr_pos.x -= self.ui.spacing().item_spacing.x;
        let size = vec2(self.curr_pos.x - self.top_left.x, self.max_height); 
        self.ui.advance_cursor_after_rect(Rect::from_min_size(self.top_left, size));
    }

}

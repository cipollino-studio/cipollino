use crate::editor::{EditorSystems, state::EditorState};

pub mod assets;
pub mod timeline;
pub mod scene;
pub mod tool;
pub mod colors;

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Panel {
    Assets(assets::AssetsPanel),
    Timeline(timeline::TimelinePanel),
    Scene(scene::ScenePanel),
    Tool(tool::ToolPanel),
    Color(colors::ColorPanel)
}

pub struct PanelViewer<'a, 'b> {
    state: &'a mut EditorState,
    systems: &'a mut EditorSystems<'b>,
    enable: bool
}

impl egui_dock::TabViewer for PanelViewer<'_, '_> {
    type Tab = (u64, Panel);

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab.1 {
            Panel::Assets(..) => "Assets",
            Panel::Timeline(..) => "Timeline",
            Panel::Scene(..) => "Scene",
            Panel::Tool(..) => "Tool Options",
            Panel::Color(..) => "Color"
        }.into()
    }

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(tab.0)
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.set_enabled(self.enable);
        match &mut tab.1 {
            Panel::Assets(assets) => assets.render(ui, &mut self.state, self.systems),
            Panel::Timeline(timeline) => timeline.render(ui, &mut self.state, self.systems), 
            Panel::Scene(scene) => scene.render(ui, &mut self.state, &mut self.systems),
            Panel::Tool(tool) => tool.render(ui, &mut self.state),
            Panel::Color(color) => color.render(ui, &mut self.state)
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PanelManager {
    tree: egui_dock::DockState<(u64, Panel)>,
    curr_id: u64,
}

impl PanelManager {

    pub fn new() -> Self {
        let tree = egui_dock::DockState::new(vec![]);
        Self {
            tree: tree,
            curr_id: 0
        }
    }

    pub fn add_panel(&mut self, panel: Panel) {
        self.tree.add_window(vec![(self.curr_id, panel)]);
        self.curr_id += 1;
    }

    pub fn render(&mut self, ctx: &egui::Context, enable: bool, state: &mut EditorState, systems: &mut EditorSystems) {
        egui_dock::DockArea::new(&mut self.tree)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut PanelViewer {
                state,
                systems,
                enable,
            });
    }

}

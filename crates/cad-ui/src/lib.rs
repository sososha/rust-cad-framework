use egui::Context;

pub trait UIComponent {
    fn show(&mut self, ctx: &Context);
}

pub struct ToolPalette {
    pub selected_tool: String,
}

impl ToolPalette {
    pub fn new() -> Self {
        Self {
            selected_tool: "Line".to_string(),
        }
    }
}

impl UIComponent for ToolPalette {
    fn show(&mut self, ctx: &Context) {
        egui::Window::new("Tools").show(ctx, |ui| {
            if ui.button("Line").clicked() {
                self.selected_tool = "Line".to_string();
            }
            if ui.button("Circle").clicked() {
                self.selected_tool = "Circle".to_string();
            }
        });
    }
}

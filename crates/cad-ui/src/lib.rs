use egui::Context;


pub enum UIAction {
    SelectTool(String),
    Save,
    Load,
    Undo,
    Redo,
    None,
}

pub struct LayerManager;

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

impl ToolPalette {
    pub fn show(&mut self, ctx: &Context) -> UIAction {
        let mut action = UIAction::None;
        egui::Window::new("Tools").show(ctx, |ui| {
            if ui.button("Line").clicked() {
                self.selected_tool = "Line".to_string();
                action = UIAction::SelectTool("Line".to_string());
            }
            if ui.button("Circle").clicked() {
                self.selected_tool = "Circle".to_string();
                action = UIAction::SelectTool("Circle".to_string());
            }
            ui.separator();
            if ui.button("Save (drawing.json)").clicked() {
                action = UIAction::Save;
            }
            if ui.button("Load (drawing.json)").clicked() {
                action = UIAction::Load;
            }
            ui.separator();
            if ui.button("Undo").clicked() {
                action = UIAction::Undo;
            }
            if ui.button("Redo").clicked() {
                action = UIAction::Redo;
            }
        });
        action
    }
}

impl LayerManager {
    pub fn show(ctx: &Context, doc: &mut cad_core::Document) {
        egui::Window::new("Layers").show(ctx, |ui| {
            if ui.button("Add Layer").clicked() {
                let new_layer_name = format!("Layer {}", doc.layers.len());
                doc.layers.push(cad_core::Layer::new(new_layer_name));
            }
            ui.separator();
            
            for (i, layer) in doc.layers.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    let is_active = i == doc.active_layer_index;
                    if ui.radio(is_active, "").clicked() {
                        doc.active_layer_index = i;
                    }
                    ui.checkbox(&mut layer.visible, "");
                    ui.label(&layer.name);
                });
            }
        });
    }
}

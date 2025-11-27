use cad_core::{Command, Document, Entity};

pub struct AddEntityCommand {
    entity: Option<Entity>,
}

impl AddEntityCommand {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity: Some(entity),
        }
    }
}

impl Command for AddEntityCommand {
    fn execute(&mut self, doc: &mut Document) {
        if let Some(entity) = self.entity.take() {
            doc.add_entity(entity);
        }
    }

    fn undo(&mut self, doc: &mut Document) {
        // Simple undo for add: pop the last entity from the active layer
        // Assumption: The entity added by this command is the last one in the active layer.
        // This is true for a simple stack if no other operations happen.
        // For more robustness, we might need IDs.
        if let Some(layer) = doc.layers.get_mut(doc.active_layer_index) {
            if let Some(entity) = layer.entities.pop() {
                self.entity = Some(entity);
            }
        }
    }
}

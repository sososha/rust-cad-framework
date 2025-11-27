use crate::primitives::Point;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct EntityId;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Entity {
    Line {
        p1: Point,
        p2: Point,
    },
    Circle {
        center: Point,
        radius: f32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layer {
    /// Name of the layer
    pub name: String,
    /// List of entities in this layer
    pub entities: Vec<Entity>,
    /// Whether the layer is visible
    pub visible: bool,
    /// Whether the layer is locked (cannot be modified)
    pub locked: bool,
}

impl Layer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            entities: Vec::new(),
            visible: true,
            locked: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Document {
    /// List of layers in the document. Order determines rendering order (bottom to top).
    pub layers: Vec<Layer>,
    /// Index of the currently active layer where new entities are added.
    pub active_layer_index: usize,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    pub fn new() -> Self {
        // Always start with a default layer
        Self {
            layers: vec![Layer::new("Layer 0".to_string())],
            active_layer_index: 0,
        }
    }

    /// Adds an entity to the currently active layer.
    /// Returns an error if no layer is active (should not happen in valid state).
    pub fn add_entity(&mut self, entity: Entity) {
        if let Some(layer) = self.layers.get_mut(self.active_layer_index) {
            layer.entities.push(entity);
        } else {
            // Fallback: Add to the last layer or create one if empty
            if self.layers.is_empty() {
                self.layers.push(Layer::new("Default".to_string()));
            }
            self.layers.last_mut().unwrap().entities.push(entity);
        }
    }
    
    /// Returns a flat list of all visible entities for rendering.
    /// TODO: This clones entities, which is not efficient for large documents.
    /// Consider returning an iterator or using a render-specific data structure.
    pub fn get_visible_entities(&self) -> Vec<Entity> {
        self.layers.iter()
            .filter(|l| l.visible)
            .flat_map(|l| l.entities.clone())
            .collect()
    }
}

pub trait Command {
    fn execute(&mut self, doc: &mut Document);
    fn undo(&mut self, doc: &mut Document);
}

pub struct CommandManager {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn execute(&mut self, mut command: Box<dyn Command>, doc: &mut Document) {
        command.execute(doc);
        self.undo_stack.push(command);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, doc: &mut Document) {
        if let Some(mut command) = self.undo_stack.pop() {
            command.undo(doc);
            self.redo_stack.push(command);
        }
    }

    pub fn redo(&mut self, doc: &mut Document) {
        if let Some(mut command) = self.redo_stack.pop() {
            command.execute(doc);
            self.undo_stack.push(command);
        }
    }
}

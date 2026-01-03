//! # Struct Editor Plugin
//!
//! This plugin provides a professional multi-panel editor for creating struct definitions.
//! It supports .struct files (folder-based) that contain struct metadata and fields.
//!
//! ## File Types
//!
//! - **Struct Definition** (.struct folder)
//!   - Contains `struct.json` with the struct definition
//!   - Appears as a single file in the file drawer
//!
//! ## Editors
//!
//! - **Struct Editor**: Multi-panel editor with properties, fields, and code preview

use plugin_editor_api::*;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use gpui::*;
use ui::dock::PanelView;

// Struct Editor modules
mod editor;
mod field_editor;
mod workspace_panels;

// Re-export main types
pub use editor::StructEditor;
pub use field_editor::{FieldEditorView, FieldEditorEvent};
pub use workspace_panels::{PropertiesPanel, FieldsPanel, CodePreviewPanel};

/// Storage for editor instances owned by the plugin
struct EditorStorage {
    panel: Arc<dyn ui::dock::PanelView>,
    wrapper: Box<StructEditorWrapper>,
}

/// The Struct Editor Plugin
pub struct StructEditorPlugin {
    /// CRITICAL: Plugin owns ALL editor instances to prevent memory leaks!
    /// The main app only gets raw pointers - it NEVER owns the Arc or Box.
    editors: Arc<Mutex<HashMap<usize, EditorStorage>>>,
    next_editor_id: Arc<Mutex<usize>>,
}

impl Default for StructEditorPlugin {
    fn default() -> Self {
        Self {
            editors: Arc::new(Mutex::new(HashMap::new())),
            next_editor_id: Arc::new(Mutex::new(0)),
        }
    }
}

impl EditorPlugin for StructEditorPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: PluginId::new("com.pulsar.struct-editor"),
            name: "Struct Editor".into(),
            version: "0.1.0".into(),
            author: "Pulsar Team".into(),
            description: "Professional multi-panel editor for creating struct definitions".into(),
        }
    }

    fn file_types(&self) -> Vec<FileTypeDefinition> {
        vec![
            FileTypeDefinition {
                id: FileTypeId::new("struct"),
                extension: "struct".to_string(),
                display_name: "Struct Definition".to_string(),
                icon: ui::IconName::Box,
                color: gpui::rgb(0x00BCD4).into(),
                structure: FileStructure::FolderBased {
                    marker_file: "struct.json".to_string(),
                    template_structure: vec![],
                },
                default_content: json!({
                    "name": "NewStruct",
                    "fields": []
                }),
                categories: vec!["Types".to_string()],
            }
        ]
    }

    fn editors(&self) -> Vec<EditorMetadata> {
        vec![EditorMetadata {
            id: EditorId::new("struct-editor"),
            display_name: "Struct Editor".into(),
            supported_file_types: vec![FileTypeId::new("struct")],
        }]
    }

    fn create_editor(
        &self,
        editor_id: EditorId,
        file_path: PathBuf,
        window: &mut Window,
        cx: &mut App,
        logger: &plugin_editor_api::EditorLogger,
    ) -> Result<(Arc<dyn PanelView>, Box<dyn EditorInstance>), PluginError> {

        logger.info("STRUCT EDITOR LOADED!!");

        logger.info(&format!("Creating editor with ID: {}", editor_id.as_str()));
        if editor_id.as_str() == "struct-editor" {
            let actual_path = if file_path.is_dir() {
                file_path.join("struct.json")
            } else {
                file_path.clone()
            };

            // Create a view context for the panel
            let panel = cx.new(|cx| {
                StructEditor::new_with_file(actual_path.clone(), window, cx)
            });

            // Wrap the panel in Arc - will be shared with main app
            let panel_arc: Arc<dyn ui::dock::PanelView> = Arc::new(panel.clone());

            // Clone file_path for logging
            let file_path_for_log = file_path.clone();

            // Create the wrapper for EditorInstance
            let wrapper = Box::new(StructEditorWrapper {
                panel: panel.into(),
                file_path,
            });

            // Generate unique ID for this editor
            let id = {
                let mut next_id = self.next_editor_id.lock().unwrap();
                let id = *next_id;
                *next_id += 1;
                id
            };

            // CRITICAL: Store Arc and Box in plugin's HashMap to keep them alive!
            self.editors.lock().unwrap().insert(id, EditorStorage {
                panel: panel_arc.clone(),
                wrapper: wrapper.clone(),
            });

            log::info!("Created struct editor instance {} for {:?}", id, file_path_for_log);

            // Return Arc (main app will clone it) and Box for EditorInstance
            Ok((panel_arc, wrapper))
        } else {
            Err(PluginError::EditorNotFound { editor_id })
        }
    }

    fn on_load(&mut self) {
        log::info!("Struct Editor Plugin loaded");
    }

    fn on_unload(&mut self) {
        // Clear all editors when plugin unloads
        let mut editors = self.editors.lock().unwrap();
        let count = editors.len();
        editors.clear();
        log::info!("Struct Editor Plugin unloaded (cleaned up {} editors)", count);
    }
}

/// Wrapper to bridge Entity<StructEditor> to EditorInstance trait
#[derive(Clone)]
pub struct StructEditorWrapper {
    panel: Entity<StructEditor>,
    file_path: std::path::PathBuf,
}

impl plugin_editor_api::EditorInstance for StructEditorWrapper {
    fn file_path(&self) -> &std::path::PathBuf {
        &self.file_path
    }

    fn save(&mut self, window: &mut Window, cx: &mut App) -> Result<(), PluginError> {
        self.panel.update(cx, |panel, cx| {
            panel.plugin_save(window, cx)
        })
    }

    fn reload(&mut self, window: &mut Window, cx: &mut App) -> Result<(), PluginError> {
        self.panel.update(cx, |panel, cx| {
            panel.plugin_reload(window, cx)
        })
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Export the plugin using the provided macro
export_plugin!(StructEditorPlugin);

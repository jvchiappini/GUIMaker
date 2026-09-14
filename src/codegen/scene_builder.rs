use crate::scene::{SceneState, WidgetKind, PlacedWidget};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Representation of a FUI Node for serialization
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuiNode {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_click: Option<String>,
    pub layout: FuiLayout,
    pub props: HashMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<FuiNode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuiLayout {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Generates the .fui XML/JSON structure representing the static view.
pub fn generate_fui_json(scene: &SceneState) -> String {
    let mut children_map: HashMap<Option<u32>, Vec<&PlacedWidget>> = HashMap::new();
    for w in &scene.widgets {
        children_map.entry(w.parent_id).or_default().push(w);
    }

    fn build_node(widget: &PlacedWidget, children_map: &HashMap<Option<u32>, Vec<&PlacedWidget>>) -> FuiNode {
        let mut props = HashMap::new();
        props.insert("label".to_string(), widget.props.label.clone());
        props.insert("bg_color".to_string(), widget.props.bg_color_hex.clone());
        // Custom macro ID string based on widget type and ID
        let id_str = format!("{}_{}", widget.kind.to_string().to_lowercase(), widget.id);

        let mut node = FuiNode {
            kind: widget.kind.to_string(),
            id: Some(id_str.clone()),
            on_click: if widget.kind == WidgetKind::Button { Some(format!("on_{}_click", id_str)) } else { None },
            layout: FuiLayout {
                x: widget.x,
                y: widget.y,
                w: widget.w,
                h: widget.h,
            },
            props,
            children: Vec::new(),
        };

        if let Some(children) = children_map.get(&Some(widget.id)) {
            for child in children {
                node.children.push(build_node(child, children_map));
            }
        }
        node
    }

    let mut root_nodes = Vec::new();
    if let Some(top_levels) = children_map.get(&None) {
        for w in top_levels {
            root_nodes.push(build_node(w, &children_map));
        }
    }

    serde_json::to_string_pretty(&root_nodes).unwrap_or_else(|_| "[]".to_string())
}

/// Generates the rust struct controller that will act as the macro injected backend.
pub fn generate_controller_macro_skeleton(scene: &SceneState) -> String {
    let mut code = String::new();
    code.push_str("//! APPLICATION CONTROLLER\n");
    code.push_str("//! Bindings to the generated .fui view.\n\n");
    code.push_str("use ferrous_ui_macros::FerrousController;\n");
    code.push_str("use ferrous_ui_core::{EventContext, NodeId};\n\n");
    code.push_str("#[derive(FerrousController, Default)]\n");
    code.push_str("#[fui_view(path = \"ui/main.fui\")]\n");
    code.push_str("pub struct AppController {\n");
    
    // Inject #[fui_id] for all buttons/inputs so they can be referenced inside the controller
    for w in &scene.widgets {
        let id_str = format!("{}_{}", w.kind.to_string().to_lowercase(), w.id);
        if w.kind == WidgetKind::Button || w.kind == WidgetKind::TextInput {
            code.push_str(&format!("    #[fui_id]\n"));
            code.push_str(&format!("    pub {}: Option<NodeId>,\n", id_str));
        }
    }
    
    code.push_str("}\n\n");
    code.push_str("impl AppController {\n");
    for w in &scene.widgets {
        if w.kind == WidgetKind::Button {
            let id_str = format!("{}_{}", w.kind.to_string().to_lowercase(), w.id);
            code.push_str(&format!("    #[fui_action]\n"));
            code.push_str(&format!("    pub fn on_{}_click(&mut self, _ctx: &mut EventContext<Self>) {{\n", id_str));
            code.push_str(&format!("        println!(\"Action triggered: {} ({})\");\n", id_str, w.props.label));
            code.push_str("    }\n\n");
        }
    }
    code.push_str("}\n");
    code
}

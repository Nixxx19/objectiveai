use std::collections::HashSet;
use std::sync::Arc;

use crate::functions::inventions::{Tool, ToolArgsType};

mod schema_lookup {
    include!(concat!(env!("OUT_DIR"), "/schema_lookup.rs"));
}

pub fn schema_tools(schemas: &[&str]) -> Vec<Tool> {
    let mut seen = HashSet::new();
    let mut tools = Vec::new();
    let mut stack: Vec<&str> = schemas.iter().copied().collect();

    while let Some(name) = stack.pop() {
        if !seen.insert(name.to_string()) {
            continue;
        }

        if let Some(content) = schema_lookup::schema_content(name) {
            let tool_name = schema_lookup::tool_name(name).unwrap();
            let tool_desc = schema_lookup::tool_description(name).unwrap();

            tools.push(Tool {
                name: tool_name,
                description: tool_desc,
                args_type: ToolArgsType::None,
                call: Arc::new(move |_| Ok(content.to_string())),
            });

            for dep in schema_lookup::schema_refs(name) {
                if !seen.contains(*dep) {
                    stack.push(dep);
                }
            }
        }
    }

    tools
}

//! Tool registration and LLM tool calling framework.
//!
//! Apps can register tools that the agent can invoke. The agent uses
//! JSON Schema to understand tool signatures and calls them via IPC.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use crate::capabilities::CapabilityType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: ToolParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameters {
    #[serde(rename = "type")]
    pub param_type: String, // "object"
    pub properties: HashMap<String, ParameterProperty>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterProperty {
    #[serde(rename = "type")]
    pub prop_type: String, // "string", "number", "boolean", "array", "object"
    pub description: String,
    pub enum_values: Option<Vec<Value>>,
    pub default: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
}

/// Tool handler trait - apps implement this for their tools
pub trait ToolHandler: Send + Sync {
    fn schema(&self) -> &ToolSchema;
    fn call(&self, args: Value) -> Result<Value, String>;
}

/// Simple function-based tool handler
pub struct FunctionTool<F> {
    schema: ToolSchema,
    handler: F,
}

impl<F> FunctionTool<F>
where
    F: Fn(Value) -> Result<Value, String> + Send + Sync + 'static,
{
    pub fn new(schema: ToolSchema, handler: F) -> Self {
        Self { schema, handler }
    }
}

impl<F> ToolHandler for FunctionTool<F>
where
    F: Fn(Value) -> Result<Value, String> + Send + Sync + 'static,
{
    fn schema(&self) -> &ToolSchema {
        &self.schema
    }

    fn call(&self, args: Value) -> Result<Value, String> {
        (self.handler)(args)
    }
}

/// Tool registry for an app
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn ToolHandler>>,
    required_capability: CapabilityType,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            required_capability: CapabilityType::RegisterTool,
        }
    }

    pub fn register<H: ToolHandler + 'static>(&mut self, tool: H) {
        let schema = tool.schema();
        self.tools.insert(schema.name.clone(), Box::new(tool));
    }

    pub fn register_function(
        &mut self,
        name: &str,
        description: &str,
        parameters: ToolParameters,
        handler: impl Fn(Value) -> Result<Value, String> + Send + Sync + 'static,
    ) {
        let schema = ToolSchema {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
        };
        self.register(FunctionTool::new(schema, handler));
    }

    pub fn get_schema(&self, name: &str) -> Option<&ToolSchema> {
        self.tools.get(name).map(|t| t.schema())
    }

    pub fn list_schemas(&self) -> Vec<ToolSchema> {
        self.tools.values().map(|t| t.schema().clone()).collect()
    }

    pub fn call(&self, call: ToolCall) -> ToolResult {
        if let Some(tool) = self.tools.get(&call.name) {
            match tool.call(call.arguments) {
                Ok(output) => ToolResult {
                    call_id: call.id,
                    success: true,
                    output,
                    error: None,
                },
                Err(e) => ToolResult {
                    call_id: call.id,
                    success: false,
                    output: Value::Null,
                    error: Some(e),
                },
            }
        } else {
            ToolResult {
                call_id: call.id,
                success: false,
                output: Value::Null,
                error: Some(format!("Tool not found: {}", call.name)),
            }
        }
    }

    pub fn call_async(&self, call: ToolCall) -> ToolResult {
        self.call(call)
    }
}

/// Helper macros for defining tool parameters
#[macro_export]
macro_rules! tool_params {
    ( $( $name:expr => { type: $t:expr, desc: $d:expr $(, required: $req:expr)? $(, enum: $e:expr)? $(, default: $def:expr)? } ),* $(,)? ) => {
        {
            let mut props = std::collections::HashMap::new();
            let mut required = Vec::new();
            $(
                let mut prop = $crate::tools::ParameterProperty {
                    prop_type: $t.to_string(),
                    description: $d.to_string(),
                    enum_values: None,
                    default: None,
                };
                $(
                    prop.enum_values = Some($e);
                )?
                $(
                    prop.default = Some($def);
                )?
                props.insert($name.to_string(), prop);
                $(
                    required.push($name.to_string());
                )?
            )*
            $crate::tools::ToolParameters {
                param_type: "object".to_string(),
                properties: props,
                required,
            }
        }
    };
}

/// Helper to create string parameter
pub fn string_param(desc: &str) -> ParameterProperty {
    ParameterProperty {
        prop_type: "string".to_string(),
        description: desc.to_string(),
        enum_values: None,
        default: None,
    }
}

/// Helper to create number parameter
pub fn number_param(desc: &str) -> ParameterProperty {
    ParameterProperty {
        prop_type: "number".to_string(),
        description: desc.to_string(),
        enum_values: None,
        default: None,
    }
}

/// Helper to create boolean parameter
pub fn bool_param(desc: &str) -> ParameterProperty {
    ParameterProperty {
        prop_type: "boolean".to_string(),
        description: desc.to_string(),
        enum_values: None,
        default: None,
    }
}

/// Helper to create array parameter
pub fn array_param(desc: &str, _items: ParameterProperty) -> ParameterProperty {
    ParameterProperty {
        prop_type: "array".to_string(),
        description: desc.to_string(),
        enum_values: None,
        default: None,
    }
}
//! MCP Protocol implementation

use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};

/// MCP Server
pub struct McpServer {
    tools: Vec<ToolDefinition>,
    resources: Vec<ResourceDefinition>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            tools: crate::tools::get_tools(),
            resources: crate::resources::get_resources(),
        }
    }

    /// Run the server, reading from stdin and writing to stdout
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();

        for line in stdin.lock().lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    let error_response = JsonRpcResponse::error(
                        None,
                        -32700,
                        format!("Parse error: {}", e),
                    );
                    writeln!(stdout_lock, "{}", serde_json::to_string(&error_response)?)?;
                    stdout_lock.flush()?;
                    continue;
                }
            };

            let response = self.handle_request(request).await;
            writeln!(stdout_lock, "{}", serde_json::to_string(&response)?)?;
            stdout_lock.flush()?;
        }

        Ok(())
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id),
            "tools/list" => self.handle_list_tools(request.id),
            "tools/call" => self.handle_call_tool(request.id, request.params).await,
            "resources/list" => self.handle_list_resources(request.id),
            "resources/read" => self.handle_read_resource(request.id, request.params).await,
            _ => JsonRpcResponse::error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        }
    }

    fn handle_initialize(&self, id: Option<serde_json::Value>) -> JsonRpcResponse {
        JsonRpcResponse::success(id, serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "serverInfo": {
                "name": "cadi-mcp-server",
                "version": env!("CARGO_PKG_VERSION")
            }
        }))
    }

    fn handle_list_tools(&self, id: Option<serde_json::Value>) -> JsonRpcResponse {
        let tools: Vec<_> = self.tools.iter()
            .map(|t| serde_json::json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.input_schema
            }))
            .collect();

        JsonRpcResponse::success(id, serde_json::json!({
            "tools": tools
        }))
    }

    async fn handle_call_tool(
        &self,
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => return JsonRpcResponse::error(id, -32602, "Missing params".to_string()),
        };

        let tool_name = params.get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("");

        let arguments = params.get("arguments")
            .cloned()
            .unwrap_or(serde_json::Value::Object(Default::default()));

        let result = crate::tools::call_tool(tool_name, arguments).await;

        match result {
            Ok(content) => JsonRpcResponse::success(id, serde_json::json!({
                "content": content
            })),
            Err(e) => JsonRpcResponse::success(id, serde_json::json!({
                "content": [{
                    "type": "text",
                    "text": format!("Error: {}", e)
                }],
                "isError": true
            })),
        }
    }

    fn handle_list_resources(&self, id: Option<serde_json::Value>) -> JsonRpcResponse {
        let resources: Vec<_> = self.resources.iter()
            .map(|r| serde_json::json!({
                "uri": r.uri,
                "name": r.name,
                "description": r.description,
                "mimeType": r.mime_type
            }))
            .collect();

        JsonRpcResponse::success(id, serde_json::json!({
            "resources": resources
        }))
    }

    async fn handle_read_resource(
        &self,
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => return JsonRpcResponse::error(id, -32602, "Missing params".to_string()),
        };

        let uri = params.get("uri")
            .and_then(|u| u.as_str())
            .unwrap_or("");

        let result = crate::resources::read_resource(uri).await;

        match result {
            Ok(contents) => JsonRpcResponse::success(id, serde_json::json!({
                "contents": contents
            })),
            Err(e) => JsonRpcResponse::error(id, -32603, e.to_string()),
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON-RPC 2.0 Request
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

impl JsonRpcResponse {
    pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<serde_json::Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError { code, message }),
        }
    }
}

/// Tool definition for MCP
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Resource definition for MCP
pub struct ResourceDefinition {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
}

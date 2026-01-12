//! MCP Protocol implementation

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::prompts::PromptDefinition;

/// MCP Server
pub struct McpServer {
    tools: Vec<ToolDefinition>,
    resources: Vec<ResourceDefinition>,
    prompts: Vec<PromptDefinition>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            tools: crate::tools::get_tools(),
            resources: crate::resources::get_resources(),
            prompts: crate::prompts::get_prompts(),
        }
    }

    /// Run the server in stdio mode (for local MCP clients like Claude Desktop)
    pub async fn run_stdio(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();

        tracing::info!("Running in stdio mode - reading from stdin");

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

    /// Run the server in HTTP mode (for Docker/container deployment)
    pub async fn run_http(self, bind_address: &str) -> Result<(), Box<dyn std::error::Error>> {
        let shared_state = Arc::new(self);

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = Router::new()
            .route("/health", get(health_check))
            .route("/", post(handle_jsonrpc))
            .route("/mcp", post(handle_jsonrpc))
            .route("/jsonrpc", post(handle_jsonrpc))
            .with_state(shared_state)
            .layer(cors)
            .layer(TraceLayer::new_for_http());

        let listener = tokio::net::TcpListener::bind(bind_address).await?;
        tracing::info!("MCP HTTP server listening on {}", bind_address);

        axum::serve(listener, app).await?;
        Ok(())
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id),
            "tools/list" => self.handle_list_tools(request.id),
            "tools/call" => self.handle_call_tool(request.id, request.params).await,
            "resources/list" => self.handle_list_resources(request.id),
            "resources/read" => self.handle_read_resource(request.id, request.params).await,
            "prompts/list" => self.handle_list_prompts(request.id),
            "prompts/get" => self.handle_get_prompt(request.id, request.params),
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
                "resources": {},
                "prompts": {},
                "streaming": {}
            },
            "serverInfo": {
                "name": "cadi-mcp-server",
                "version": env!("CARGO_PKG_VERSION")
            },
            "instructions": "⚡ CADI: The Build System for the Agentic Era\n\nWORKFLOW:\n1. cadi_search - Find existing atoms (~50 tokens vs 500+ to write)\n2. cadi_view_context - Get assembled code with ghost imports\n3. Only write new code if nothing found\n4. cadi_import after writing to save for reuse\n\nGhost imports automatically include type definitions so LLMs don't hallucinate!"
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

    fn handle_list_prompts(&self, id: Option<serde_json::Value>) -> JsonRpcResponse {
        let prompts: Vec<_> = self.prompts.iter()
            .map(|p| serde_json::json!({
                "name": p.name,
                "description": p.description,
                "arguments": p.arguments
            }))
            .collect();

        JsonRpcResponse::success(id, serde_json::json!({
            "prompts": prompts
        }))
    }

    fn handle_get_prompt(
        &self,
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => return JsonRpcResponse::error(id, -32602, "Missing params".to_string()),
        };

        let name = params.get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("");

        let arguments = params.get("arguments")
            .cloned()
            .unwrap_or(serde_json::Value::Object(Default::default()));

        match crate::prompts::get_prompt(name, &arguments) {
            Ok(messages) => JsonRpcResponse::success(id, serde_json::json!({
                "messages": messages
            })),
            Err(e) => JsonRpcResponse::error(id, -32602, e),
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
#[allow(dead_code)]
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

// =============================================================================
// HTTP Handlers for Docker/container deployment
// =============================================================================

/// Health check endpoint
async fn health_check() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}

/// JSON-RPC handler for HTTP transport
async fn handle_jsonrpc(
    State(server): State<Arc<McpServer>>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    let response = server.handle_request(request).await;
    Json(response)
}

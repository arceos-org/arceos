// LLM client — OpenAI-compatible API with tool_calls and vision support.

use std::time::{Duration, Instant};

use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

/// A tool definition sent in the request.
#[derive(Serialize, Clone)]
pub struct ToolDef {
    #[serde(rename = "type")]
    pub type_: String,
    pub function: FunctionDef,
}

#[derive(Serialize, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

// ---------------------------------------------------------------------------
// Message types — supports text, multimodal content, and tool roles
// ---------------------------------------------------------------------------

/// A chat message. `content` can be a plain string or a multimodal array.
/// We serialize it as an untagged enum via serde_json::Value.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub role: String,

    /// For regular messages: string content.
    /// For multimodal (vision): array of content parts.
    /// For tool_calls assistant messages: may be empty string or null.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Value>,

    /// Present when the assistant wants to call tools.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    /// Present in tool-result messages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    /// Create a simple text message.
    pub fn text(role: &str, content: &str) -> Self {
        Self {
            role: role.to_string(),
            content: Some(Value::String(content.to_string())),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a multimodal message with image + text.
    pub fn vision(role: &str, image_base64: &str, mime: &str, text: &str) -> Self {
        let parts = serde_json::json!([
            {
                "type": "image_url",
                "image_url": {
                    "url": format!("data:{};base64,{}", mime, image_base64)
                }
            },
            {
                "type": "text",
                "text": text
            }
        ]);
        Self {
            role: role.to_string(),
            content: Some(parts),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a tool-result message.
    pub fn tool_result(tool_call_id: &str, result: &str) -> Self {
        Self {
            role: "tool".to_string(),
            content: Some(Value::String(result.to_string())),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.to_string()),
        }
    }

    /// Create an assistant message that contains tool_calls (for context history).
    pub fn assistant_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: Some(Value::String(String::new())),
            tool_calls: Some(tool_calls),
            tool_call_id: None,
        }
    }

    /// Get text content as &str, if it is a plain string.
    pub fn text_content(&self) -> Option<&str> {
        self.content.as_ref().and_then(|v| v.as_str())
    }
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    pub message: ResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ResponseMessage {
    pub role: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub reasoning_content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub function: FunctionCall,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

// ---------------------------------------------------------------------------
// LLM Client
// ---------------------------------------------------------------------------

pub struct LlmClient {
    api_url: String,
    model: String,
}

impl LlmClient {
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            api_url: format!("{}/chat/completions", base_url),
            model: model.to_string(),
        }
    }

    /// Send a chat completion request with optional tools.
    /// Returns the full ResponseMessage (may contain content, tool_calls, reasoning).
    pub fn chat(
        &self,
        messages: &[ChatMessage],
        tools: Option<&[ToolDef]>,
    ) -> Result<ResponseMessage, String> {
        let request_body = ChatRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            temperature: 0.3,
            tools: tools.map(|t| t.to_vec()),
            tool_choice: tools.map(|_| "auto".to_string()),
            max_tokens: Some(2048),
        };

        let body_json =
            serde_json::to_string(&request_body).map_err(|e| format!("Serialize error: {}", e))?;

        let mut retry_delay = Duration::from_secs(1);
        let max_retries = 3;

        for attempt in 0..max_retries {
            if attempt > 0 {
                eprintln!(
                    "[llm] Retry {}/{} after {:?}...",
                    attempt, max_retries, retry_delay
                );
                std::thread::sleep(retry_delay);
                retry_delay *= 2;
            }

            let start = Instant::now();
            let result = minreq::post(&self.api_url)
                .with_header("Content-Type", "application/json")
                .with_body(body_json.clone())
                .with_timeout(120)
                .send();

            let elapsed = start.elapsed();

            match result {
                Ok(resp) => {
                    debug!(
                        "[llm] HTTP {} ({:.0}ms)",
                        resp.status_code,
                        elapsed.as_millis()
                    );

                    if resp.status_code != 200 {
                        let error_body = resp.as_str().unwrap_or("<unreadable>");
                        error!("[llm] Error: {}", error_body);
                        if attempt < max_retries - 1 {
                            continue;
                        }
                        return Err(format!("HTTP {}: {}", resp.status_code, error_body));
                    }

                    let body_str = resp
                        .as_str()
                        .map_err(|e| format!("Response decode error: {}", e))?;
                    let chat_resp: ChatResponse = serde_json::from_str(body_str)
                        .map_err(|e| format!("JSON parse error: {} (body: {})", e, body_str))?;

                    if chat_resp.choices.is_empty() {
                        return Err("Empty choices in response".to_string());
                    }
                    return Ok(chat_resp.choices.into_iter().next().unwrap().message);
                }
                Err(e) => {
                    error!("[llm] Request failed ({:.0}ms): {}", elapsed.as_millis(), e);
                    if attempt < max_retries - 1 {
                        continue;
                    }
                    return Err(format!(
                        "Request failed after {} retries: {}",
                        max_retries, e
                    ));
                }
            }
        }

        Err("Exhausted retries".to_string())
    }
}

// Tool definitions and dispatch for ArceAgent.
//
// Each tool is an OpenAI-compatible function definition. The dispatch layer
// executes the tool and returns a string result to feed back to the LLM.

use log::info;
use serde_json::{Value, json};

use crate::{
    hal::HalInterface,
    llm::{FunctionDef, ToolDef},
    memory::Memory,
};

// ---------------------------------------------------------------------------
// Tool catalog — returns the full list of tools for the LLM
// ---------------------------------------------------------------------------

pub fn tool_catalog() -> Vec<ToolDef> {
    vec![
        make_tool(
            "move_forward",
            "控制机器人向前移动指定距离（厘米）",
            json!({
                "type": "object",
                "properties": {
                    "distance_cm": {"type": "number", "description": "前进距离，单位厘米"}
                },
                "required": ["distance_cm"]
            }),
        ),
        make_tool(
            "move_backward",
            "控制机器人向后移动指定距离（厘米）",
            json!({
                "type": "object",
                "properties": {
                    "distance_cm": {"type": "number", "description": "后退距离，单位厘米"}
                },
                "required": ["distance_cm"]
            }),
        ),
        make_tool(
            "turn",
            "控制机器人原地转向。direction 为 left 或 right，angle_deg 为角度。",
            json!({
                "type": "object",
                "properties": {
                    "direction": {"type": "string", "enum": ["left", "right"], "description": "转向方向"},
                    "angle_deg": {"type": "number", "description": "转动角度"}
                },
                "required": ["direction", "angle_deg"]
            }),
        ),
        make_tool(
            "stop",
            "立即停止机器人所有运动",
            json!({"type": "object", "properties": {}}),
        ),
        make_tool(
            "arm_lower",
            "放下机械臂到工作位置",
            json!({"type": "object", "properties": {}}),
        ),
        make_tool(
            "arm_raise",
            "抬起机械臂到收纳位置",
            json!({"type": "object", "properties": {}}),
        ),
        make_tool(
            "gripper_close",
            "收紧机械臂末端的夹子（抓取物体）",
            json!({"type": "object", "properties": {}}),
        ),
        make_tool(
            "gripper_open",
            "张开机械臂末端的夹子（释放物体）",
            json!({"type": "object", "properties": {}}),
        ),
        make_tool(
            "take_photo",
            "调用摄像头拍摄一张照片并返回图像描述。照片会被自动附加到下一轮对话中供你查看。",
            json!({"type": "object", "properties": {}}),
        ),
        make_tool(
            "read_memory",
            "读取持久记忆文件的全部内容。记忆中保存了用户的偏好和长期指令。",
            json!({"type": "object", "properties": {}}),
        ),
        make_tool(
            "write_memory",
            "向持久记忆文件写入一条记录。下次启动时会自动加载。用于保存用户偏好或长期指令。",
            json!({
                "type": "object",
                "properties": {
                    "key": {"type": "string", "description": "记忆键名，如 user_name, wake_word"},
                    "value": {"type": "string", "description": "记忆内容"}
                },
                "required": ["key", "value"]
            }),
        ),
        make_tool(
            "delete_memory",
            "从持久记忆中删除一条记录",
            json!({
                "type": "object",
                "properties": {
                    "key": {"type": "string", "description": "要删除的记忆键名"}
                },
                "required": ["key"]
            }),
        ),
        make_tool(
            "get_time",
            "获取当前系统时间",
            json!({"type": "object", "properties": {}}),
        ),
        make_tool(
            "get_status",
            "获取机器人当前状态（电池电量、位置、机械臂状态等）",
            json!({"type": "object", "properties": {}}),
        ),
    ]
}

fn make_tool(name: &str, description: &str, parameters: Value) -> ToolDef {
    ToolDef {
        type_: "function".to_string(),
        function: FunctionDef {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
        },
    }
}

// ---------------------------------------------------------------------------
// Tool dispatch — execute a tool call and return (result_string, optional_image)
// ---------------------------------------------------------------------------

/// Execute a tool call. Returns (result_text, optional_image_base64).
/// The optional image is used by take_photo — the caller should attach it
/// as a vision message in the next LLM round.
pub fn dispatch(
    name: &str,
    args: &Value,
    hal: &mut HalInterface,
    memory: &mut Memory,
) -> (String, Option<String>) {
    info!("[tool] {}({})", name, args);

    let result = match name {
        "move_forward" => {
            let dist = args["distance_cm"].as_f64().unwrap_or(0.0);
            hal.move_forward(dist)
        }
        "move_backward" => {
            let dist = args["distance_cm"].as_f64().unwrap_or(0.0);
            hal.move_backward(dist)
        }
        "turn" => {
            let dir = args["direction"].as_str().unwrap_or("left");
            let angle = args["angle_deg"].as_f64().unwrap_or(0.0);
            hal.turn(dir, angle)
        }
        "stop" => hal.stop(),
        "arm_lower" => hal.arm_lower(),
        "arm_raise" => hal.arm_raise(),
        "gripper_close" => hal.gripper_close(),
        "gripper_open" => hal.gripper_open(),
        "take_photo" => {
            let (description, img_b64) = hal.take_photo();
            return (description, Some(img_b64));
        }
        "read_memory" => memory.read_all(),
        "write_memory" => {
            let key = args["key"].as_str().unwrap_or("");
            let value = args["value"].as_str().unwrap_or("");
            memory.write(key, value)
        }
        "delete_memory" => {
            let key = args["key"].as_str().unwrap_or("");
            memory.delete(key)
        }
        "get_time" => hal.get_time(),
        "get_status" => hal.get_status(),
        _ => format!("Unknown tool: {}", name),
    };

    (result, None)
}

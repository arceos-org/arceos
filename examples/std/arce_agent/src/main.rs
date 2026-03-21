// ArceAgent v0.2: Interactive AI Agent on ArceOS Unikernel
//
// Architecture:
//   User input / autonomous loop
//     → LLM reasoning (qwen3.5-plus-thinking, with vision + tool_call)
//     → Tool execution (movement, arm, camera, memory, etc.)
//     → Output to user
//
// Uses minreq for HTTP (no TLS — requires a local HTTP proxy).
// Compiles identically on Linux (for development) and ArceOS (for deployment).


#[cfg(target_os = "hermit")]
use arceos_rust as _;

mod base64;
mod context;
mod hal;
mod lineedit;
mod llm;
mod memory;
mod tools;


use context::ContextManager;
use hal::HalInterface;
use lineedit::LineEditor;
use llm::{ChatMessage, LlmClient};
use log::{debug, error, info};
use memory::Memory;
use tools::{dispatch, tool_catalog};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

#[cfg(target_os = "hermit")]
const API_BASE: &str = "http://10.0.2.2:8080/v1";
#[cfg(not(target_os = "hermit"))]
const API_BASE: &str = "http://localhost:8080/v1";
const MODEL: &str = "qwen3.5-plus-thinking";
const MAX_CONTEXT_MESSAGES: usize = 40;
/// Max tool-call rounds per user turn (prevents infinite loops).
const MAX_TOOL_ROUNDS: usize = 10;

const SYSTEM_PROMPT: &str = r#"你是 ArceAgent，一个运行在 ArceOS Unikernel 上的智能机器人助手。

## 你的身体
你搭载在一个小型移动机器人平台上，具备：
- **底盘**：可以前进、后退、左转、右转
- **机械臂**：可以升降，末端有夹子可以抓取和释放物体
- **摄像头**：可以拍照获取当前视野中的图像
- **文件系统**：可以通过持久记忆保存和读取信息

## 你的能力
- 使用提供的工具来控制机器人执行物理操作
- 通过 take_photo 工具拍照，照片会自动附加到对话中供你查看
- 通过 read_memory / write_memory / delete_memory 来管理持久记忆
- 用户告诉你需要长期记住的事情时，主动使用 write_memory 保存
- 通过 get_status 查看机器人当前状态
- 通过 get_time 获取系统运行时间

## 交互方式
- 用户会通过命令行与你对话
- 你可以自由地与用户聊天，回答问题
- 当需要执行物理操作时，调用相应的工具
- 如果需要了解周围环境，先拍照再做判断
- 你的文字回复会直接显示在命令行上，请使用简洁自然的语言

## 重要注意事项
- 你运行在一个极简的 Unikernel 操作系统上，没有 Linux，启动时间不到 1 秒，内存占用仅 12MB
- 这体现了"云端大脑 + 裸机身体"的架构：你的推理能力来自云端 LLM，执行能力来自本地硬件
- 请记住你是一个有物理实体的机器人，不是纯软件助手"#;

const STARTUP_PROMPT: &str = "你刚刚启动。请先用 read_memory \
                              工具检查持久记忆中是否有需要注意的内容，然后根据记忆内容和当前状态，\
                              向用户输出一句简短的欢迎/问候信息。如果记忆中有用户的名字或偏好，\
                              请在问候中体现出来。";

// ---------------------------------------------------------------------------
// LLM interaction loop — shared by startup greeting and user turns.
// Sends context to LLM, executes any tool calls, prints final text response.
// ---------------------------------------------------------------------------

fn run_llm_turn(
    llm: &LlmClient,
    ctx: &mut ContextManager,
    tools: &[llm::ToolDef],
    hal: &mut HalInterface,
    mem: &mut Memory,
) {
    let mut pending_image: Option<String> = None;

    for round in 0..MAX_TOOL_ROUNDS {
        let messages = ctx.build_messages();

        debug!(
            "[main] Calling LLM (round {}, {} messages)...",
            round + 1,
            messages.len()
        );

        let response = match llm.chat(&messages, Some(tools)) {
            Ok(r) => r,
            Err(e) => {
                error!("[main] LLM error: {}", e);
                println!("（LLM 通信失败: {}）", e);
                break;
            }
        };

        // Print reasoning (thinking) as debug log
        if let Some(ref reasoning) = response.reasoning_content {
            if !reasoning.is_empty() {
                info!("[thinking] {}", reasoning);
            }
        }

        // Check if LLM wants to call tools
        if let Some(ref tool_calls) = response.tool_calls {
            if !tool_calls.is_empty() {
                // Record the assistant's tool_calls message in context
                ctx.push(ChatMessage::assistant_tool_calls(tool_calls.clone()));

                // Execute each tool call
                for tc in tool_calls {
                    let args: serde_json::Value =
                        serde_json::from_str(&tc.function.arguments).unwrap_or_default();
                    let (result, image) = dispatch(&tc.function.name, &args, &mut *hal, &mut *mem);

                    info!("[tool] {} -> {}", tc.function.name, result);

                    // Record tool result in context
                    ctx.push(ChatMessage::tool_result(&tc.id, &result));

                    // If tool returned an image, save it for attachment
                    if let Some(img_b64) = image {
                        if !img_b64.is_empty() {
                            pending_image = Some(img_b64);
                        }
                    }
                }

                // If a photo was taken, attach it as a vision message
                if let Some(img_b64) = pending_image.take() {
                    let vision_msg = ChatMessage::vision(
                        "user",
                        &img_b64,
                        "image/png",
                        "这是刚才摄像头拍到的照片，请描述你看到了什么。",
                    );
                    ctx.push(vision_msg);
                }

                // Continue the loop — LLM needs to see tool results
                continue;
            }
        }

        // No tool calls — LLM produced a text response
        if let Some(ref text) = response.content {
            if !text.is_empty() {
                println!();
                println!("{}", text);
                println!();
                // Record assistant response in context
                ctx.push(ChatMessage::text("assistant", text));
            }
        }

        // Done with this turn
        break;
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    // 启用自动换行
    println!("\x1b[?7h");
    println!("========================================");
    println!("  ArceAgent v0.2 — AI Robot Assistant");
    println!("  Running on ArceOS Unikernel");
    println!("========================================");
    println!();

    // init logging (colors, log level)
    let mut builder = env_logger::Builder::new();
    builder.format_timestamp(None);
    builder.format_target(false);
    builder.filter_level(log::LevelFilter::Debug);
    builder.write_style(env_logger::WriteStyle::Always);

    builder.init();

    // Initialize subsystems
    let llm = LlmClient::new(API_BASE, MODEL);
    let mut hal = HalInterface::init();
    let mut mem = Memory::load();

    // Build system prompt with loaded memory
    let full_prompt = format!("{}{}", SYSTEM_PROMPT, mem.summary());
    let mut ctx = ContextManager::new(&full_prompt, MAX_CONTEXT_MESSAGES);
    let tools = tool_catalog();

    info!("[main] Model: {}", MODEL);
    info!("[main] API:   {}", API_BASE);
    info!("[main] Tools: {} registered", tools.len());

    // --- Startup: LLM checks memory and generates a greeting ---
    println!();
    println!("正在初始化...");
    ctx.push(ChatMessage::text("user", STARTUP_PROMPT));
    run_llm_turn(&llm, &mut ctx, &tools, &mut hal, &mut mem);

    println!("  支持: 左右箭头移动光标, 上下箭头浏览历史, Ctrl-C 取消, Ctrl-D 退出");
    println!();

    let mut editor = LineEditor::new(100);

    loop {
        let input = match editor.read_line("> ") {
            Some(line) => line,
            None => break, // EOF / Ctrl-D
        };

        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        if input == "quit" || input == "exit" {
            println!("再见！");
            break;
        }

        // Add user message to context
        ctx.push(ChatMessage::text("user", input));
        run_llm_turn(&llm, &mut ctx, &tools, &mut hal, &mut mem);
    }
}

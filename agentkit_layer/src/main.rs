mod llm_interface;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, SampleRate,
};
use llm_interface::{ChatCompletionRequest, ChatMessage, LanguageModel, OpenAICompatibleModel};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    env,
    error::Error,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::sleep;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// ACP 消息结构体
#[derive(Debug, Serialize, Deserialize)]
struct AcpMessage {
    #[serde(rename = "type")]
    message_type: String,
    seq_id: u64,
    payload: Value,
}

/// 执行动作载荷结构体
#[derive(Debug, Serialize, Deserialize)]
struct PerformActionPayload {
    action: String,
    command_name: Option<String>,
    element_id: Option<String>,
    target_query: Option<String>,
    params: Option<Value>,
}

/// ACP 响应载荷结构体
#[derive(Debug, Serialize, Deserialize)]
struct AcpResponsePayload {
    success: bool,
    message: String,
}

/// 初始化 Whisper 上下文
fn initialize_whisper(model_path: &str) -> Result<WhisperContext, Box<dyn Error>> {
    let path = Path::new(model_path);
    if !path.exists() {
        return Err(format!("Whisper 模型文件不存在: {}", model_path).into());
    }

    let ctx = WhisperContext::new_with_params(
        model_path,
        WhisperContextParameters::default(),
    )?;
    Ok(ctx)
}

/// 捕获音频并转录为文本
fn capture_and_transcribe(whisper_ctx: &WhisperContext) -> Result<String, Box<dyn Error>> {
    println!("开始录音 (5 秒)...");

    // 初始化音频设备
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("无法找到默认输入设备")?;

    println!("使用音频设备: {}", device.name()?);

    // 尝试获取16kHz单声道的配置
    let mut config = device.default_input_config()?.config();
    let sample_format = device.default_input_config()?.sample_format();
    
    // 查询是否支持16kHz采样率的配置
    let supported_configs = device.supported_input_configs()
        .map_err(|e| format!("查询支持的音频配置错误: {}", e))?;
    
    // 尝试找到支持16kHz的配置
    for range in supported_configs {
        if range.min_sample_rate().0 <= 16000 && range.max_sample_rate().0 >= 16000 {
            config.sample_rate = cpal::SampleRate(16000);
            break;
        }
    }
    
    // 我们需要将音频重采样为 16kHz 单声道以供 Whisper 使用
    println!("音频配置: 采样率 {}Hz, 声道 {}, 格式 {:?}", 
             config.sample_rate.0, config.channels, sample_format);
    
    // 创建音频缓冲区，以 f32 格式存储采样
    let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
    let buffer_clone = buffer.clone();
    
    // 调整配置以适应 Whisper 的需求
    // 确保采样率为16kHz
    config.sample_rate = SampleRate(16000);
    
    // 录制时间 (5 秒)
    let recording_duration = Duration::from_secs(5);
    
    // 设置音频流
    let err_fn = move |err| {
        eprintln!("音频流错误: {}", err);
    };

    let stream = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _: &_| {
                let mut buffer = buffer_clone.lock().unwrap();
                buffer.extend_from_slice(data);
            },
            err_fn,
            None,
        )?,
        SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data: &[i16], _: &_| {
                let mut buffer = buffer_clone.lock().unwrap();
                // 转换 i16 为 f32
                buffer.extend(data.iter().map(|&s| s as f32 / i16::MAX as f32));
            },
            err_fn,
            None,
        )?,
        SampleFormat::U16 => device.build_input_stream(
            &config,
            move |data: &[u16], _: &_| {
                let mut buffer = buffer_clone.lock().unwrap();
                // 转换 u16 为 f32
                buffer.extend(data.iter().map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0));
            },
            err_fn,
            None,
        )?,
        _ => return Err("不支持的采样格式".into()),
    };

    // 开始录音
    stream.play()?;
    std::thread::sleep(recording_duration);
    drop(stream);

    println!("录音结束，开始转录...");

    // 获取音频数据并转录
    let audio_data = buffer.lock().unwrap().clone();
    
    if audio_data.is_empty() {
        return Err("没有捕获到音频数据".into());
    }

    // 使用whisper-rs 0.10.0版本的API
    // 创建whisper状态
    let mut state = whisper_ctx.create_state()?;
    
    // 设置参数
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    // 设置语言为中文（可选）
    params.set_language(Some("zh"));
    // 设置其他可能的参数
    // params.set_translate(false);
    // params.set_single_segment(true);
    
    // 执行转录
    state.full(params, &audio_data)?;
    
    // 获取结果
    let num_segments = state.full_n_segments()?;
    let mut transcription = String::new();
    
    for i in 0..num_segments {
        if let Ok(segment) = state.full_get_segment_text(i) {
            transcription.push_str(&segment);
        }
    }
    
    // 如果转录失败，允许用户手动输入
    if transcription.is_empty() {
        println!("语音转录结果为空，请手动输入转录结果:");
        let mut manual_input = String::new();
        std::io::stdin().read_line(&mut manual_input)?;
        transcription = manual_input.trim().to_string();
    }

    println!("转录结果: {}", transcription);
    Ok(transcription)
}

/// 使用 LLM 解释命令
async fn interpret_command_with_llm(
    llm: Arc<dyn LanguageModel>,
    transcription: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    println!("使用 LLM 解释命令: {}", transcription);

    // 系统提示
    let system_prompt = "您是一个 AI 助手，正在帮助用户控制一个桌面应用程序。
该应用程序有一个按钮，点击它可以循环改变背景颜色。
如果用户的语音命令（转录文本）表明了想要改变背景颜色或点击此按钮的意图，
请仅输出命令字符串 'CYCLE_COLOR_COMMAND'。
不要添加任何其他文本、解释或客套话。只需要这个命令字符串。
如果用户的意图不明确或不相关，请输出 'UNKNOWN_COMMAND'。";

    // 构建请求
    let request = ChatCompletionRequest {
        model: "gpt-3.5-turbo".to_string(), // 可以配置为其他模型
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: transcription.to_string(),
            },
        ],
    };

    // 发送请求给 LLM
    let response = llm.chat_completions(request).await?;

    // 解析响应
    if let Some(choice) = response.choices.first() {
        let command = choice.message.content.trim();
        
        if command == "CYCLE_COLOR_COMMAND" || command == "UNKNOWN_COMMAND" {
            return Ok(command.to_string());
        } else {
            println!("LLM 返回了意外的响应: {}", command);
            return Ok("UNKNOWN_COMMAND".to_string());
        }
    } else {
        println!("LLM 没有返回选择");
        return Ok("UNKNOWN_COMMAND".to_string());
    }
}

/// 发送 ACP 请求
fn send_acp_request(
    stream: &mut TcpStream,
    command_name: &str,
) -> Result<AcpResponsePayload, Box<dyn Error>> {
    // 生成随机序列 ID
    let seq_id = rand::thread_rng().gen::<u64>();

    // 构建执行动作载荷
    let payload = PerformActionPayload {
        action: "custom_command".to_string(),
        command_name: Some(command_name.to_string()),
        element_id: None,
        target_query: None,
        params: None,
    };

    // 构建 ACP 消息
    let acp_message = AcpMessage {
        message_type: "request".to_string(),
        seq_id,
        payload: serde_json::to_value(payload).map_err(|e| format!("序列化载荷失败: {}", e))?,
    };

    // 将 ACP 消息转换为 JSON 字符串，并添加换行符
    let message_json = serde_json::to_string(&acp_message).map_err(|e| format!("序列化 ACP 消息失败: {}", e))?;
    let message_with_newline = format!("{}\n", message_json);

    println!("发送 ACP 请求: {}", message_with_newline);

    // 发送 ACP 请求
    stream
        .write_all(message_with_newline.as_bytes())
        .map_err(|e| format!("发送 ACP 请求失败: {}", e))?;

    // 读取 ACP 响应
    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .map_err(|e| format!("读取 ACP 响应失败: {}", e))?;

    println!("收到 ACP 响应: {}", response_line);

    // 解析 ACP 响应
    let acp_response: AcpMessage = serde_json::from_str(&response_line)
        .map_err(|e| format!("解析 ACP 响应失败: {}", e))?;

    // 检查序列 ID 是否匹配
    if acp_response.seq_id != seq_id {
        return Err(format!("ACP 响应序列 ID 不匹配: 预期 {}, 实际 {}", seq_id, acp_response.seq_id).into());
    }

    // 解析响应载荷
    let response_payload: AcpResponsePayload = serde_json::from_value(acp_response.payload)
        .map_err(|e| format!("解析响应载荷失败: {}", e))?;

    Ok(response_payload)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== AgentKit Layer 启动 ===");

    // 检查 OpenAI API 密钥是否存在
    let api_key = env::var("OPENAI_API_KEY").ok();
    if api_key.is_none() {
        println!("警告: 未设置 OPENAI_API_KEY 环境变量");
        println!("您可以设置 OPENAI_API_KEY 环境变量来使用 OpenAI 的 API");
        println!("例如: export OPENAI_API_KEY=your_api_key_here");
        println!("\n继续执行，但 LLM 功能可能受限...");
    } else {
        println!("已检测到 OPENAI_API_KEY 环境变量");
    }

    // 初始化 Whisper 上下文（语音识别）
    let model_path = "./ggml-base.en.bin"; // 可以配置为命令行参数或环境变量
    let use_voice = Path::new(model_path).exists();
    
    let whisper_ctx = if use_voice {
        match initialize_whisper(model_path) {
            Ok(ctx) => {
                println!("Whisper 模型初始化成功，将使用语音识别");
                Some(ctx)
            },
            Err(e) => {
                println!("警告: 无法初始化 Whisper 模型: {}。将使用手动输入代替。", e);
                None
            }
        }
    } else {
        println!("未找到 Whisper 模型文件: {}。将使用手动输入代替语音识别。", model_path);
        println!("如需使用语音识别，请下载 Whisper GGML 模型文件，例如从:");
        println!("https://huggingface.co/ggerganov/whisper.cpp/tree/main");
        None
    };

    // 初始化 LLM
    let base_url = env::var("OPENAI_API_BASE").ok();
    let model_name = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string());

    let llm = Arc::new(OpenAICompatibleModel::new(api_key, base_url, model_name));
    println!("LLM 初始化完成");

    // 连接到 target_gpui_app
    let mut tcp_stream = match TcpStream::connect("127.0.0.1:7880") {
        Ok(stream) => {
            println!("已连接到 target_gpui_app");
            stream
        }
        Err(e) => {
            return Err(format!("无法连接到 target_gpui_app: {}", e).into());
        }
    };

    // 主循环
    loop {
        if whisper_ctx.is_some() {
            println!("\n按 Enter 开始语音识别，或输入 'quit' 退出，或直接输入命令:");
        } else {
            println!("\n输入命令或 'quit' 退出:");
        }
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        let input = input.trim();
        
        if input.eq_ignore_ascii_case("quit") {
            break;
        }

        // 获取转录文本，要么通过语音识别，要么通过手动输入
        let transcription = if input.is_empty() && whisper_ctx.is_some() {
            // 只有当Whisper模型可用且用户按下Enter时才尝试语音识别
            match capture_and_transcribe(whisper_ctx.as_ref().unwrap()) {
                Ok(text) => text,
                Err(e) => {
                    println!("转录音频失败: {}。请手动输入命令:", e);
                    let mut manual_input = String::new();
                    std::io::stdin().read_line(&mut manual_input)?;
                    manual_input.trim().to_string()
                }
            }
        } else if input.is_empty() {
            // 当没有Whisper模型但用户按下Enter时，提醒用户
            println!("语音识别不可用。请手动输入命令:");
            let mut manual_input = String::new();
            std::io::stdin().read_line(&mut manual_input)?;
            manual_input.trim().to_string()
        } else {
            // 用户直接输入了文本命令
            input.to_string()
        };

        if transcription.is_empty() {
            println!("未收到有效输入，请重试");
            continue;
        }

        // 使用 LLM 解释命令
        let command = match interpret_command_with_llm(llm.clone(), &transcription).await {
            Ok(cmd) => cmd,
            Err(e) => {
                println!("LLM 解释失败: {}", e);
                continue;
            }
        };

        // 处理命令
        if command == "CYCLE_COLOR_COMMAND" {
            println!("发送 CYCLE_COLOR 命令到 target_gpui_app");
            
            match send_acp_request(&mut tcp_stream, "CYCLE_COLOR") {
                Ok(response) => {
                    println!(
                        "命令执行 {}: {}",
                        if response.success { "成功" } else { "失败" },
                        response.message
                    );
                }
                Err(e) => {
                    println!("发送 ACP 请求失败: {}", e);
                    // 尝试重新连接
                    println!("尝试重新连接到 target_gpui_app...");
                    match TcpStream::connect("127.0.0.1:7880") {
                        Ok(stream) => {
                            println!("重新连接成功");
                            tcp_stream = stream;
                        }
                        Err(e) => {
                            println!("重新连接失败: {}", e);
                        }
                    }
                }
            }
        } else {
            println!("未知命令或意图不明确");
        }

        // 添加短暂延迟以避免 CPU 使用率过高
        sleep(Duration::from_millis(100)).await;
    }

    println!("=== AgentKit Layer 已退出 ===");
    Ok(())
}
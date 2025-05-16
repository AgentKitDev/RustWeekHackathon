use gpui::{
    self, div, 
    DismissEvent, EventEmitter, Render, Styled, IntoElement, Context,
    ParentElement, Window,
    Hsla, black, white, AppContext,
};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 应用程序的背景颜色枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundColor {
    White,
    LightBlue,
    LightGreen,
}

impl BackgroundColor {
    /// 循环到下一个颜色
    pub fn next(&self) -> Self {
        match self {
            BackgroundColor::White => BackgroundColor::LightBlue,
            BackgroundColor::LightBlue => BackgroundColor::LightGreen,
            BackgroundColor::LightGreen => BackgroundColor::White,
        }
    }

    /// 获取颜色名称
    pub fn name(&self) -> &'static str {
        match self {
            BackgroundColor::White => "White",
            BackgroundColor::LightBlue => "Light Blue",
            BackgroundColor::LightGreen => "Light Green",
        }
    }
    
    /// 转换为GPUI Hsla颜色
    pub fn to_rgb(&self) -> Hsla {
        match self {
            BackgroundColor::White => Hsla { h: 0.0, s: 0.0, l: 1.0, a: 1.0 },
            BackgroundColor::LightBlue => Hsla { h: 210.0, s: 0.5, l: 0.8, a: 1.0 },
            BackgroundColor::LightGreen => Hsla { h: 120.0, s: 0.5, l: 0.8, a: 1.0 },
        }
    }
}

/// 应用程序状态
pub struct AppState {
    current_bg_color: Arc<Mutex<BackgroundColor>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_bg_color: Arc::new(Mutex::new(BackgroundColor::White)),
        }
    }

    /// 循环背景颜色
    pub fn cycle_bg_color(&self) -> BackgroundColor {
        let mut color = self.current_bg_color.lock().unwrap();
        *color = color.next();
        *color
    }

    /// 获取当前背景颜色
    pub fn get_bg_color(&self) -> BackgroundColor {
        *self.current_bg_color.lock().unwrap()
    }
}

/// 自定义操作类型，用于应用程序内部通信
#[derive(Debug, Clone)]
pub enum AppAction {
    CycleColor,
}

/// 应用根视图
pub struct RootView {
    app_state: Arc<AppState>,
}

impl RootView {
    pub fn new(app_state: Arc<AppState>, _cx: &mut Window) -> Self {
        Self { app_state }
    }
}

impl Render for RootView {
    fn render(&mut self, _cx: &mut Window, _view_cx: &mut Context<Self>) -> impl IntoElement {
        let bg_color = self.app_state.get_bg_color();
        let color_name = bg_color.name();

        // 使用Hsla创建颜色
        let white_bg = white(); 
        let black_text = black();
        let black_border = black();
        
        // 获取背景颜色的Hsla值
        let bg_color_value = bg_color.to_rgb();

        div()
            .size_full()
            .bg(bg_color_value)
            .children(vec![
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_4()
                    .children(vec![
                        div()
                            .text_xl()
                            .pb_4()
                            .child(format!("当前背景: {}", color_name)),
                        div() // 简化的按钮
                            .bg(white_bg)
                            .text_color(black_text)
                            .border_1()
                            .border_color(black_border)
                            .rounded_md()
                            .px_4()
                            .py_2()
                            .child("点击按钮或通过ACP改变颜色"),
                    ]),
            ])
    }
}

// 注释掉 actions! 宏 - 已经手动实现了 Action trait
// actions!(root_view, [CycleColor]);

impl EventEmitter<DismissEvent> for RootView {}


/// ACP 消息类型
#[derive(Debug, Serialize, Deserialize)]
pub struct AcpMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub seq_id: u64,
    pub payload: Value,
}

/// ACP 响应载荷
#[derive(Debug, Serialize, Deserialize)]
pub struct AcpResponsePayload {
    pub success: bool,
    pub message: String,
}

/// 处理 ACP 连接
pub fn handle_acp_connection(
    stream: TcpStream,
    app_state: Arc<AppState>,
    _app_context: impl AppContext // 使用trait约束
) {
    let mut reader = BufReader::new(stream.try_clone().expect("无法克隆 TCP 流"));
    let mut writer = stream;

    let mut line = String::new();
    if let Err(e) = reader.read_line(&mut line) {
        eprintln!("读取 ACP 请求时出错: {}", e);
        return;
    }

    println!("收到 ACP 请求: {}", line);

    // 解析 ACP 消息
    let acp_message: AcpMessage = match serde_json::from_str(&line) {
        Ok(msg) => msg,
        Err(e) => {
            eprintln!("解析 ACP 消息时出错: {}", e);
            return;
        }
    };

    // 获取序列 ID 用于响应
    let seq_id = acp_message.seq_id;

    // 检查是否是请求类型
    if acp_message.message_type != "request" {
        send_error_response(&mut writer, seq_id, "不支持的消息类型");
        return;
    }

    // 解析 payload 中的 action 和 command_name
    let action = match acp_message.payload.get("action") {
        Some(Value::String(action)) => action.as_str(),
        _ => {
            send_error_response(&mut writer, seq_id, "缺少 action 字段");
            return;
        }
    };

    // 检查 action 类型
    if action != "custom_command" {
        send_error_response(&mut writer, seq_id, "不支持的 action 类型");
        return;
    }

    // 获取 command_name
    let command_name = match acp_message.payload.get("command_name") {
        Some(Value::String(cmd)) => cmd.as_str(),
        _ => {
            send_error_response(&mut writer, seq_id, "缺少 command_name 字段");
            return;
        }
    };

    // 处理 CYCLE_COLOR 命令
    if command_name == "CYCLE_COLOR" {
        // 直接更改应用状态，不再派发操作
        app_state.cycle_bg_color();

        // 发送成功响应
        let response = AcpMessage {
            message_type: "response".to_string(),
            seq_id,
            payload: serde_json::to_value(AcpResponsePayload {
                success: true,
                message: "颜色已通过 ACP 循环".to_string(),
            })
            .unwrap(),
        };

        if let Err(e) = writeln!(writer, "{}", serde_json::to_string(&response).unwrap()) {
            eprintln!("发送 ACP 响应时出错: {}", e);
        }
    } else {
        send_error_response(&mut writer, seq_id, "未知命令");
    }
}

/// 发送错误响应
pub fn send_error_response(writer: &mut TcpStream, seq_id: u64, error_message: &str) {
    let response = AcpMessage {
        message_type: "response".to_string(),
        seq_id,
        payload: serde_json::to_value(AcpResponsePayload {
            success: false,
            message: format!("错误: {}", error_message),
        })
        .unwrap(),
    };

    if let Err(e) = writeln!(writer, "{}", serde_json::to_string(&response).unwrap()) {
        eprintln!("发送错误响应时出错: {}", e);
    }
}
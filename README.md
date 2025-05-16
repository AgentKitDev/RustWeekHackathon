# AgentKit - AI 语音控制桌面应用框架

AgentKit 是一个概念验证框架 (MVP)，旨在实现对 macOS 桌面应用程序的语音控制。该项目包含两个主要组件：

1. **target_gpui_app**：一个使用 GPUI 框架构建的简单桌面应用程序，可通过 ACP 协议接收命令来改变背景颜色。
2. **agentkit_layer**：作为"大脑"的应用程序，能够捕获音频，将语音转换为文本，通过 LLM 理解用户意图，并向目标应用发送相应的命令。

## 功能特点

- 基于 **GPUI** 构建的简单桌面应用程序
- 使用 **whisper-rs** 进行语音转文本 (STT)
- 使用 **OpenAI API** 或兼容接口进行语言理解
- 通过自定义 **应用控制协议 (ACP)** 实现应用程序间的通信

## 先决条件

- Rust (最新稳定版)
- OpenAI API 密钥 (推荐，但可选 - 设置环境变量 `OPENAI_API_KEY`)
- Whisper GGML 模型文件 (可选，用于语音识别，例如 `ggml-base.en.bin`)

## 安装与使用

1. 克隆仓库：

   ```
   git clone https://github.com/yourusername/agentkit.git
   cd agentkit
   ```

2. （可选）设置 OpenAI API 密钥环境变量（推荐以获得最佳体验）：

   ```
   export OPENAI_API_KEY=your_api_key
   ```

3. （可选）如果希望使用语音识别，下载 Whisper GGML 模型文件并放置在 `agentkit_layer` 目录下：

   ```
   # 可以从以下网址下载模型文件
   # https://huggingface.co/ggerganov/whisper.cpp/tree/main
   ```

4. （可选）如果您希望使用本地 LLM 服务而不是 OpenAI API：

   ```
   export OPENAI_API_BASE=http://localhost:11434/v1
   export OPENAI_MODEL=your_model_name
   ```

5. 构建并安装：

   ```
   cargo build --release
   # 可选：将bin目录添加到PATH
   export PATH=$PATH:$(pwd)/bin
   ```

6. 启动 AgentKit：

   ```
   # 使用命令行界面
   agentkit

   # 或者直接使用启动脚本
   ./start_agentkit.sh
   ```

7. 跟随提示操作:
   - 如果安装了 Whisper 模型文件，可以按 Enter 键开始语音录制，说出类似"改变背景颜色"的命令
   - 如果没有 Whisper 模型文件，可以直接输入文本命令，例如"改变背景颜色"

## 命令行界面

AgentKit 提供了一个简单的命令行界面以便于使用：

```
用法: agentkit [选项]

选项:
  --build      首先构建项目再运行
  --separate   显示如何单独启动组件的说明
  --help, -h   显示此帮助信息

示例:
  agentkit           直接启动应用
  agentkit --build   先构建再启动应用
```

## 手动启动组件

如果您希望单独启动每个组件，可以使用以下命令：

1. 首先启动 `target_gpui_app`：

   ```
   cd target_gpui_app
   cargo run
   ```

2. 然后在另一个终端中启动 `agentkit_layer`：
   ```
   cd agentkit_layer
   cargo run
   ```

## 应用控制协议 (ACP)

ACP 是一个简单的基于 JSON 的协议，用于应用程序间的通信：

**请求格式：**

```json
{
  "type": "request",
  "seq_id": <u64>,
  "payload": {
    "action": "custom_command",
    "command_name": "CYCLE_COLOR"
  }
}
```

**响应格式：**

```json
{
  "type": "response",
  "seq_id": <u64>,
  "payload": {
    "success": <boolean>,
    "message": <string>
  }
}
```

## 项目扩展

虽然当前 MVP 仅实现了改变背景颜色的基本功能，但 ACP 协议的设计已考虑未来扩展，例如：

- 集成 AccessKit 实现无障碍控制
- 支持更多 UI 框架
- 实现更复杂的应用程序控制命令

## 许可

[MIT 许可证](LICENSE)

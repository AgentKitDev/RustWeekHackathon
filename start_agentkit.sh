#!/bin/bash

# AgentKit 启动脚本
# 该脚本用于启动 target_gpui_app 和 agentkit_layer

set -e  # 遇到错误立即退出

# 显示彩色输出
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # 无颜色

# 打印标题
print_title() {
    echo -e "${BLUE}====================================================${NC}"
    echo -e "${BLUE}           AgentKit 语音控制应用框架              ${NC}"
    echo -e "${BLUE}====================================================${NC}"
    echo
}

# 显示帮助信息
show_help() {
    echo -e "用法: $0 [选项]"
    echo
    echo -e "选项:"
    echo -e "  --build      首先构建项目再运行"
    echo -e "  --help       显示此帮助信息"
    echo
    echo -e "示例:"
    echo -e "  $0            直接启动应用"
    echo -e "  $0 --build    先构建再启动应用"
    echo
}

# 检查命令行参数
BUILD=false

while [ "$#" -gt 0 ]; do
    case "$1" in
        --build)
            BUILD=true
            shift
            ;;
        --help)
            print_title
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}错误: 未知参数 $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# 打印标题
print_title

# 项目根目录
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

# 检查配置
echo -e "${BLUE}正在检查配置...${NC}"

# 检查 OpenAI API 密钥
if [ -z "$OPENAI_API_KEY" ]; then
    echo -e "${BLUE}OpenAI API 密钥未设置。${NC}"
    echo -e "您可以使用以下命令设置 OpenAI API 密钥:"
    echo -e "export OPENAI_API_KEY=your_api_key_here"
    echo -e "${BLUE}将继续运行，但 LLM 功能可能受限。${NC}"
    echo
else
    echo -e "${GREEN}检测到 OpenAI API 密钥。${NC}"
fi

# 检查 Whisper 模型文件（可选）
WHISPER_MODEL="./agentkit_layer/ggml-base.en.bin"
if [ ! -f "$WHISPER_MODEL" ]; then
    echo -e "${BLUE}Whisper 模型文件不存在: $WHISPER_MODEL${NC}"
    echo -e "将使用文本输入代替语音识别。"
    echo -e "如需启用语音识别功能，您可以下载 Whisper GGML 模型文件:"
    echo -e "https://huggingface.co/ggerganov/whisper.cpp/tree/main"
    echo -e "并将其放置在 agentkit_layer 目录下。"
    echo
else
    echo -e "${GREEN}检测到 Whisper 模型文件，将启用语音识别。${NC}"
fi

# 构建项目
if [ "$BUILD" = true ]; then
    echo -e "${GREEN}正在构建项目...${NC}"
    cargo build
    echo -e "${GREEN}构建完成!${NC}"
    echo
fi

# 启动 target_gpui_app (后台运行)
echo -e "${GREEN}正在启动 target_gpui_app...${NC}"
cd "$ROOT_DIR/target_gpui_app"
cargo run &
TARGET_APP_PID=$!

# 等待 target_gpui_app 启动
sleep 3
echo -e "${GREEN}target_gpui_app 已启动 (PID: $TARGET_APP_PID)${NC}"
echo

# 启动 agentkit_layer
echo -e "${GREEN}正在启动 agentkit_layer...${NC}"
cd "$ROOT_DIR/agentkit_layer"
cargo run

# 当 agentkit_layer 退出时，清理 target_gpui_app
echo -e "${GREEN}正在关闭 target_gpui_app...${NC}"
kill $TARGET_APP_PID 2>/dev/null || true
echo -e "${GREEN}已关闭所有组件。${NC}"
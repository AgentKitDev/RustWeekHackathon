use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage,
        CreateChatCompletionRequestArgs, Role,
    },
    Client,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 聊天消息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// 聊天完成请求结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
}

/// 聊天选择结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessage,
}

/// 聊天完成响应结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub choices: Vec<ChatChoice>,
}

/// 语言模型 trait
#[async_trait]
pub trait LanguageModel: Send + Sync {
    /// 发送聊天完成请求
    async fn chat_completions(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, Box<dyn std::error::Error + Send + Sync>>;
}

/// OpenAI 兼容模型
pub struct OpenAICompatibleModel {
    client: Client<OpenAIConfig>,
    model: String,
}

impl OpenAICompatibleModel {
    /// 创建新的 OpenAI 兼容模型
    pub fn new(api_key: Option<String>, base_url: Option<String>, model: String) -> Self {
        let mut config = OpenAIConfig::default();
        
        if let Some(key) = api_key {
            config = config.with_api_key(key);
        }
        
        if let Some(url) = base_url {
            config = config.with_api_base(url);
        }
        
        let client = Client::with_config(config);
        
        Self { client, model }
    }
}

#[async_trait]
impl LanguageModel for OpenAICompatibleModel {
    async fn chat_completions(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, Box<dyn std::error::Error + Send + Sync>> {
        // 将我们的请求格式转换为 async-openai 格式
        let mut messages = Vec::new();
        
        for msg in &request.messages {
            match msg.role.as_str() {
                "system" => {
                    let message = ChatCompletionRequestSystemMessage {
                        role: Role::System,
                        content: msg.content.clone(),
                        name: None,
                    };
                    messages.push(message.into());
                },
                "user" => {
                    let message = ChatCompletionRequestUserMessage {
                        role: Role::User,
                        content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(msg.content.clone()),
                        name: None,
                    };
                    messages.push(message.into());
                },
                "assistant" => {
                    // 使用正确的格式创建助手消息
                    let content = msg.content.clone();
                    let assistant_msg = async_openai::types::ChatCompletionRequestAssistantMessage {
                        role: Role::Assistant,
                        content: Some(content),
                        name: None,
                        tool_calls: None,
                        #[allow(deprecated)]
                        function_call: None,
                    };
                    messages.push(async_openai::types::ChatCompletionRequestMessage::Assistant(assistant_msg));
                },
                _ => continue, // 跳过未知角色
            }
        }
        
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages)
            .build()?;
        
        // 发送请求到 OpenAI API
        let response = self.client.chat().create(request).await?;
        
        // 将 async-openai 响应格式转换为我们的格式
        let choices = response
            .choices
            .into_iter()
            .map(|choice| {
                let message = ChatMessage {
                    role: choice.message.role.to_string(),
                    content: choice.message.content.unwrap_or_default(),
                };
                
                ChatChoice { message }
            })
            .collect();
        
        Ok(ChatCompletionResponse { choices })
    }
}
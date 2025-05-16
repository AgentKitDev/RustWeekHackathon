## AgentKit: A Technical Vision for Building Universal AI-Automated Human-Computer Interaction Based on Rust (MVP Exploration)

> I actually came up with this project idea back in April. Here’s the full blog post: [https://dev.to/zhanghandong/agentkit-a-technical-vision-for-building-universal-ai-automation-for-human-computer-interaction-2523](https://dev.to/zhanghandong/agentkit-a-technical-vision-for-building-universal-ai-automation-for-human-computer-interaction-2523)

**Project Vision:** To create AgentKit, a universal AI automation suite built with Rust, enabling AI to understand and operate various application interfaces. ultimately achieving cross-platform, cross-device intelligent automation, and enhancing the efficiency and convenience of human-computer interaction.

**This Hackathon's MVP Goal and Current Status:**

The goal of this Hackathon is to preliminarily explore and validate the core concept of AgentKit—achieving automated control of desktop GUI applications driven by AI (simplified). We chose `gpui` as the target UI framework for this attempt.

**Currently, we have set up the basic framework of the project and implemented the core workflow in code. However, due to the ongoing process of familiarizing ourselves with the `gpui` framework's API (particularly its view system, context management, and event handling mechanisms), some compilation errors in the `target_gpui_app` component have not yet been fully resolved. Therefore, a complete end-to-end demonstration is not yet possible.**

Despite this, this exploration has clearly outlined AgentKit's technical path and potential.

**MVP Core Components and Envisioned Workflow:**

1.  **`target_gpui_app` (Target Application - Based on GPUI):**

    - **Concept:** A simple macOS desktop application built with `gpui`. The interface includes a button that, when clicked or upon receiving an external command, cycles the application's background color.
    - **Interaction Interface (ACP - Simplified Application Control Protocol):** The application embeds a lightweight TCP server, listening on a specific port (e.g., `127.0.0.1:7880`) to receive and parse simple JSON commands from the `agentkit_layer`.

2.  **`agentkit_layer` (AgentKit Core Control Layer - Rust):**
    - **Voice Input Processing:**
      - Uses the `cpal` crate to capture microphone audio on macOS.
      - Utilizes the `whisper-rs` crate and a local Whisper GGML model for **local, free Speech-to-Text (STT)**.
    - **Intent Understanding and Command Generation (LLM-driven):**
      - Transcribed text is sent to a Large Language Model (LLM) compatible with the OpenAI Chat Completions API (e.g., OpenAI's GPT models or a local Ollama instance).
      - The LLM parses natural language instructions (e.g., "change background color") into predefined ACP commands (e.g., `{"action":"custom_command", "command_name":"CYCLE_COLOR"}`).
    - **Application Control Protocol (ACP) Communication:**
      - Encapsulates LLM-generated commands into simple JSON messages.
      - Sends these JSON commands to `target_gpui_app` via a TCP connection.

**Envisioned Demonstration Flow:**

User issues a voice command -> `agentkit_layer` performs STT and LLM intent understanding -> Generates ACP command -> ACP command is sent via TCP to `target_gpui_app` -> `target_gpui_app` executes the command and changes its background color.

**Technical Vision and Value:**

- **Potential of Rust:** Validates the feasibility of building such automation agents using Rust, leveraging its performance and safety advantages.
- **Modular Design:** STT, LLM, and application control modules are relatively independent, facilitating easy replacement and upgrades.
- **Local-First and Privacy:** The adoption of local STT reflects consideration for user privacy and offline capabilities.
- **LLM-Powered Interaction:** Demonstrates the powerful capabilities of LLMs in natural language understanding and task decomposition, key to future intelligent interaction.
- **Application Control Protocol (ACP):** A preliminary design for an inter-application control protocol, which is a crucial foundation for achieving cross-application, cross-framework automation. **We believe that a standardized ACP, combined with robust UI understanding capabilities, is core to AgentKit's universality.**

**Expectations for GPUI and Future Outlook:**

During this exploration, we have deeply experienced the complexity of directly manipulating UI elements. To enable AgentKit (and other similar automation tools) to interact more deeply and reliably with `gpui` applications, we **eagerly anticipate that the `gpui` framework will natively integrate or provide support for accessibility services similar to AccessKit in the future.**

- **Value of AccessKit:** AccessKit can provide a standardized, cross-platform interface allowing external tools (like AgentKit) to structurally "understand" UI elements' hierarchy, roles, states, and available actions, and to "execute" these actions. This would be far more reliable and powerful than methods relying on screen coordinate clicks or image recognition.
- **Future of AgentKit:** If `gpui` and more UI frameworks support AccessKit, AgentKit's ACP protocol can evolve to directly leverage AccessKit's capabilities. This would enable fine-grained, semantic automation control over any AccessKit-integrated application, truly realizing the vision of "AI operating a computer like a human."

**Conclusion:** Although this MVP could not be fully demonstrated due to time constraints and unfamiliarity with the `gpui` API, we have successfully outlined AgentKit's technical blueprint and validated the design concepts of its core components (local STT, LLM intent understanding, inter-application control protocol). We firmly believe that with further support for accessibility technologies (like AccessKit) from modern UI frameworks such as `gpui`, AgentKit has the potential to become a powerful, universal AI automation solution.

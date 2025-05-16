use std::env;
use std::process::{Command, exit};
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // 获取当前可执行文件的路径
    let current_exe = env::current_exe().expect("无法获取当前可执行文件路径");
    let exe_dir = current_exe.parent().expect("无法获取父目录");
    let project_root = Path::new(exe_dir).join("..");
    
    // 帮助信息
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        print_help();
        return;
    }
    
    // 检查启动方式
    if args.len() > 1 && args[1] == "--separate" {
        println!("AgentKit - AI 语音控制桌面应用框架");
        println!("\n请先启动 target_gpui_app:");
        println!("cd target_gpui_app && cargo run");
        println!("\n然后在另一个终端启动 agentkit_layer:");
        println!("cd agentkit_layer && cargo run");
        return;
    }
    
    // 查找并运行启动脚本
    let script_path = project_root.join("start_agentkit.sh");
    
    if !script_path.exists() {
        eprintln!("错误: 找不到启动脚本 {}", script_path.display());
        eprintln!("请确保您在正确的目录中运行此命令，或者使用 --separate 参数手动启动组件。");
        exit(1);
    }
    
    // 构建参数
    let mut script_args = vec![];
    if args.len() > 1 {
        for arg in &args[1..] {
            script_args.push(arg);
        }
    }
    
    // 运行启动脚本
    match Command::new("bash")
        .arg(&script_path)
        .args(script_args)
        .status() 
    {
        Ok(status) => {
            if !status.success() {
                eprintln!("启动脚本执行失败，退出代码: {:?}", status.code());
                exit(status.code().unwrap_or(1));
            }
        },
        Err(e) => {
            eprintln!("无法执行启动脚本: {}", e);
            exit(1);
        }
    }
}

fn print_help() {
    println!("AgentKit - AI 语音控制桌面应用框架");
    println!("\n用法: agentkit [选项]");
    println!("\n选项:");
    println!("  --build      首先构建项目再运行");
    println!("  --separate   显示如何单独启动组件的说明");
    println!("  --help, -h   显示此帮助信息");
    println!("\n示例:");
    println!("  agentkit           直接启动应用");
    println!("  agentkit --build   先构建再启动应用");
}

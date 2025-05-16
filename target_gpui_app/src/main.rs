use gpui::{Application, App, Window};
use std::{sync::Arc, thread, net::TcpListener};
use target_gpui_app::{AppState, RootView, handle_acp_connection};

fn main() {
    Application::new().run(|cx| {
        let app_state = Arc::new(AppState::new());
        let app_state_clone_for_tcp = app_state.clone();

        // 创建窗口和根视图
        cx.open_window(
            Default::default(),
            |cx_window, _| {
                RootView::new(app_state.clone(), cx_window)
            },
        );

        // 启动 TCP 服务器线程处理 ACP 请求
        let cx_handle = cx.focus_handle();

        thread::spawn(move || {
            let listener = TcpListener::bind("127.0.0.1:7880").expect("无法绑定到端口 7880");
            println!("ACP 服务器已启动在 127.0.0.1:7880");

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        println!("新的 ACP 连接已建立");
                        handle_acp_connection(stream, app_state_clone_for_tcp.clone(), cx_handle.clone());
                    }
                    Err(e) => {
                        eprintln!("接受连接时出错: {}", e);
                    }
                }
            }
        });
    });
}

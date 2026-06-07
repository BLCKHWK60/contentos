use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager,
};

mod stt;
mod publish;

// Show (and create focus on) a window by label.
fn show_window(app: &tauri::AppHandle, label: &str) {
    if let Some(win) = app.get_webview_window(label) {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            stt::transcribe,
            stt::list_models,
            stt::download_model,
            stt::delete_model,
            publish::http_request,
            publish::oauth_listen,
            publish::oauth_await
        ])
        .setup(|app| {
            // ----- Menu-bar (tray) menu -----
            let capture = MenuItem::with_id(app, "capture", "Quick Capture…", true, Some("CmdOrCtrl+Shift+C"))?;
            let open = MenuItem::with_id(app, "open", "Open ContentOS", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit = MenuItem::with_id(app, "quit", "Quit ContentOS", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&capture, &open, &sep, &quit])?;

            let _tray = TrayIconBuilder::with_id("contentos-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "capture" => show_window(app, "capture"),
                    "open" => show_window(app, "main"),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // Left-click the menu-bar icon -> pop the quick-capture window.
                    if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                        show_window(tray.app_handle(), "capture");
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ContentOS");
}

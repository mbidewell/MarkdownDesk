use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    AppHandle, Manager, WebviewWindow, WindowEvent,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

pub struct AppState {
    pub current_file: Mutex<Option<PathBuf>>,
    pub dirty: Mutex<bool>,
    pub pending_file: Mutex<Option<PathBuf>>,
}

fn js_string(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}

fn update_window_title(window: &WebviewWindow, file: Option<&PathBuf>, dirty: bool) {
    let name = file
        .and_then(|f| f.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("Untitled.md");
    let title = if dirty {
        format!("* {}", name)
    } else {
        name.to_string()
    };
    let _ = window.set_title(&title);
}

fn open_file_at_path(window: &WebviewWindow, path: PathBuf, app: &AppHandle) {
    let state = app.state::<AppState>();
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled.md");
            let base = path
                .parent()
                .map(|p| format!("file://{}/", p.display()))
                .unwrap_or_else(|| "file:///".to_string());

            *state.current_file.lock().unwrap() = Some(path.clone());
            *state.dirty.lock().unwrap() = false;
            update_window_title(window, Some(&path), false);

            let js = format!(
                "editorAPI.setMarkdown({}, {}, {});",
                js_string(&text),
                js_string(name),
                js_string(&base)
            );
            let _ = window.eval(&js);
        }
        Err(err) => {
            let msg = format!("Could not open file: {}", err);
            window
                .dialog()
                .message(msg)
                .kind(MessageDialogKind::Error)
                .title("Error Opening File")
                .show(|_| {});
        }
    }
}

fn prompt_new_document(window: &WebviewWindow, app: &AppHandle) {
    let state = app.state::<AppState>();
    let is_dirty = *state.dirty.lock().unwrap();
    if is_dirty {
        let title_name = state
            .current_file
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|f| f.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled.md")
            .to_string();

        let window_clone = window.clone();
        window
            .dialog()
            .message(format!("Save changes to {}?", title_name))
            .title("Unsaved Changes")
            .kind(MessageDialogKind::Warning)
            .buttons(MessageDialogButtons::OkCancelCustom(
                "Save".to_string(),
                "Discard".to_string(),
            ))
            .show(move |result| {
                if result {
                    let _ = window_clone.eval("postNativeMessage('command', 'save');");
                } else {
                    let state = window_clone.app_handle().state::<AppState>();
                    *state.current_file.lock().unwrap() = None;
                    *state.dirty.lock().unwrap() = false;
                    update_window_title(&window_clone, None, false);
                    let _ = window_clone.eval(
                        "editorAPI.setMarkdown('', 'Untitled.md', 'file:///', true);",
                    );
                }
            });
    } else {
        *state.current_file.lock().unwrap() = None;
        *state.dirty.lock().unwrap() = false;
        update_window_title(window, None, false);
        let _ = window.eval("editorAPI.setMarkdown('', 'Untitled.md', 'file:///', true);");
    }
}

fn prompt_open_dialog(window: &WebviewWindow, app: &AppHandle) {
    let app_clone = app.clone();
    let window_clone = window.clone();
    window
        .dialog()
        .file()
        .add_filter("Markdown Files", &["md", "markdown", "txt"])
        .pick_file(move |file_path| {
            if let Some(path) = file_path {
                let p = match path {
                    tauri_plugin_dialog::FilePath::Path(buf) => buf,
                    tauri_plugin_dialog::FilePath::Url(url) => {
                        PathBuf::from(url.to_file_path().unwrap_or_default())
                    }
                };
                open_file_at_path(&window_clone, p, &app_clone);
            }
        });
}

fn prompt_save_as(window: &WebviewWindow, app: &AppHandle) {
    let window_clone = window.clone();
    let state = app.state::<AppState>();
    let default_name = state
        .current_file
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("Untitled.md")
        .to_string();

    window
        .dialog()
        .file()
        .add_filter("Markdown Files", &["md", "markdown"])
        .set_file_name(&default_name)
        .save_file(move |file_path| {
            if let Some(path) = file_path {
                let p = match path {
                    tauri_plugin_dialog::FilePath::Path(buf) => buf,
                    tauri_plugin_dialog::FilePath::Url(url) => {
                        PathBuf::from(url.to_file_path().unwrap_or_default())
                    }
                };
                let escaped_path = js_string(&p.to_string_lossy());
                let js = format!(
                    "window.__TAURI_INTERNALS__.invoke('save_content', {{ path: {}, content: editorAPI.getMarkdown() }});",
                    escaped_path
                );
                let _ = window_clone.eval(&js);
            }
        });
}

#[tauri::command]
fn save_content(
    path: Option<String>,
    content: String,
    window: WebviewWindow,
    app: AppHandle,
) -> Result<String, String> {
    let state = app.state::<AppState>();
    let target = if let Some(p) = path {
        PathBuf::from(p)
    } else if let Some(cur) = state.current_file.lock().unwrap().clone() {
        cur
    } else {
        return Err("No destination path specified".to_string());
    };

    if let Err(e) = std::fs::write(&target, &content) {
        let msg = format!("Could not save {}: {}", target.display(), e);
        window
            .dialog()
            .message(&msg)
            .kind(MessageDialogKind::Error)
            .title("Save Error")
            .show(|_| {});
        return Err(msg);
    }

    *state.current_file.lock().unwrap() = Some(target.clone());
    *state.dirty.lock().unwrap() = false;
    update_window_title(&window, Some(&target), false);

    let filename = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Untitled.md");
    let base = target
        .parent()
        .map(|p| format!("file://{}/", p.display()))
        .unwrap_or_else(|| "file:///".to_string());

    let _ = window.eval(&format!(
        "editorAPI.setMarkdown({}, {}, {});",
        js_string(&content),
        js_string(filename),
        js_string(&base)
    ));

    Ok(target.to_string_lossy().to_string())
}

#[tauri::command]
fn native_message(
    name: String,
    data: Option<serde_json::Value>,
    window: WebviewWindow,
    app: AppHandle,
) {
    let state = app.state::<AppState>();
    match name.as_str() {
        "ready" => {
            let pending = state.pending_file.lock().unwrap().take();
            if let Some(path) = pending {
                open_file_at_path(&window, path, &app);
            } else {
                update_window_title(&window, None, false);
            }
        }
        "changed" => {
            *state.dirty.lock().unwrap() = true;
            let current = state.current_file.lock().unwrap().clone();
            update_window_title(&window, current.as_ref(), true);
        }
        "openLink" => {
            if let Some(serde_json::Value::String(url)) = data {
                let _ = open::that(&url);
            }
        }
        "command" => {
            let cmd = data
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            match cmd.as_str() {
                "new" => prompt_new_document(&window, &app),
                "open" => prompt_open_dialog(&window, &app),
                "save" => {
                    let has_file = state.current_file.lock().unwrap().is_some();
                    if has_file {
                        let _ = window.eval(
                            "window.__TAURI_INTERNALS__.invoke('save_content', { path: null, content: editorAPI.getMarkdown() });",
                        );
                    } else {
                        prompt_save_as(&window, &app);
                    }
                }
                "save_as" => prompt_save_as(&window, &app),
                "print" | "export_pdf" => {
                    let _ = window.eval("editorAPI.print();");
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let file_menu = Submenu::with_items(
        app,
        "File",
        true,
        &[
            &MenuItem::with_id(app, "new", "New", true, Some("CmdOrCtrl+N"))?,
            &MenuItem::with_id(app, "open", "Open…", true, Some("CmdOrCtrl+O"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "save", "Save", true, Some("CmdOrCtrl+S"))?,
            &MenuItem::with_id(app, "save_as", "Save As…", true, Some("CmdOrCtrl+Shift+S"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "print", "Print / Export PDF…", true, Some("CmdOrCtrl+P"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, Some("Quit"))?,
        ],
    )?;

    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, Some("Undo"))?,
            &PredefinedMenuItem::redo(app, Some("Redo"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, Some("Cut"))?,
            &PredefinedMenuItem::copy(app, Some("Copy"))?,
            &PredefinedMenuItem::paste(app, Some("Paste"))?,
            &PredefinedMenuItem::select_all(app, Some("Select All"))?,
        ],
    )?;

    let format_menu = Submenu::with_items(
        app,
        "Format",
        true,
        &[
            &MenuItem::with_id(app, "fmt_bold", "Bold", true, Some("CmdOrCtrl+B"))?,
            &MenuItem::with_id(app, "fmt_italic", "Italic", true, Some("CmdOrCtrl+I"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "fmt_h1", "Heading 1", true, Some("CmdOrCtrl+1"))?,
            &MenuItem::with_id(app, "fmt_h2", "Heading 2", true, Some("CmdOrCtrl+2"))?,
            &MenuItem::with_id(app, "fmt_h3", "Heading 3", true, Some("CmdOrCtrl+3"))?,
            &MenuItem::with_id(app, "fmt_paragraph", "Paragraph", true, Some("CmdOrCtrl+0"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "fmt_unordered", "Bulleted List", true, None::<&str>)?,
            &MenuItem::with_id(app, "fmt_ordered", "Numbered List", true, None::<&str>)?,
            &MenuItem::with_id(app, "fmt_quote", "Block Quote", true, None::<&str>)?,
            &MenuItem::with_id(app, "fmt_code", "Code Block", true, None::<&str>)?,
            &MenuItem::with_id(app, "fmt_link", "Link…", true, Some("CmdOrCtrl+K"))?,
        ],
    )?;

    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &MenuItem::with_id(app, "view_source", "Toggle Markdown Source", true, Some("CmdOrCtrl+Shift+M"))?,
        ],
    )?;

    Menu::with_items(app, &[&file_menu, &edit_menu, &format_menu, &view_menu])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let pending_cli_file = std::env::args()
        .skip(1)
        .find(|arg| !arg.starts_with('-') && std::path::Path::new(arg).exists())
        .map(PathBuf::from);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            current_file: Mutex::new(None),
            dirty: Mutex::new(false),
            pending_file: Mutex::new(pending_cli_file),
        })
        .invoke_handler(tauri::generate_handler![native_message, save_content])
        .setup(|app| {
            let menu = build_menu(app.handle())?;
            app.set_menu(menu)?;

            app.on_menu_event(|app, event| {
                let id = event.id().as_ref();
                if let Some(window) = app.get_webview_window("main") {
                    let handle = app.clone();
                    match id {
                        "new" => prompt_new_document(&window, &handle),
                        "open" => prompt_open_dialog(&window, &handle),
                        "save" => {
                            let state = handle.state::<AppState>();
                            let has_file = state.current_file.lock().unwrap().is_some();
                            if has_file {
                                let _ = window.eval(
                                    "window.__TAURI_INTERNALS__.invoke('save_content', { path: null, content: editorAPI.getMarkdown() });",
                                );
                            } else {
                                prompt_save_as(&window, &handle);
                            }
                        }
                        "save_as" => prompt_save_as(&window, &handle),
                        "print" => {
                            let _ = window.eval("editorAPI.print();");
                        }
                        "view_source" => {
                            let _ = window.eval("editorAPI.toggleSource();");
                        }
                        "fmt_bold" => { let _ = window.eval("editorAPI.format('bold');"); }
                        "fmt_italic" => { let _ = window.eval("editorAPI.format('italic');"); }
                        "fmt_h1" => { let _ = window.eval("editorAPI.format('h1');"); }
                        "fmt_h2" => { let _ = window.eval("editorAPI.format('h2');"); }
                        "fmt_h3" => { let _ = window.eval("editorAPI.format('h3');"); }
                        "fmt_paragraph" => { let _ = window.eval("editorAPI.format('paragraph');"); }
                        "fmt_unordered" => { let _ = window.eval("editorAPI.format('unordered');"); }
                        "fmt_ordered" => { let _ = window.eval("editorAPI.format('ordered');"); }
                        "fmt_quote" => { let _ = window.eval("editorAPI.format('quote');"); }
                        "fmt_code" => { let _ = window.eval("editorAPI.format('code');"); }
                        "fmt_link" => { let _ = window.eval("editorAPI.format('link');"); }
                        _ => {}
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let state = app.state::<AppState>();
                let is_dirty = *state.dirty.lock().unwrap();
                if is_dirty {
                    api.prevent_close();
                    if let Some(webview) = window.get_webview_window("main") {
                        let filename = state
                            .current_file
                            .lock()
                            .unwrap()
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("Untitled.md")
                            .to_string();

                        let webview_clone = webview.clone();
                        webview
                            .dialog()
                            .message(format!("Save changes to {} before closing?", filename))
                            .title("Unsaved Changes")
                            .kind(MessageDialogKind::Warning)
                            .buttons(MessageDialogButtons::OkCancelCustom(
                                "Save".to_string(),
                                "Discard".to_string(),
                            ))
                            .show(move |save_choice| {
                                if save_choice {
                                    let _ = webview_clone.eval("postNativeMessage('command', 'save');");
                                } else {
                                    *webview_clone.app_handle().state::<AppState>().dirty.lock().unwrap() = false;
                                    let _ = webview_clone.close();
                                }
                            });
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    AppHandle, Manager, WebviewWindow, WindowEvent,
};
use tauri_plugin_dialog::{
    DialogExt, MessageDialogButtons, MessageDialogKind, MessageDialogResult,
};
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub enum PendingAction {
    None,
    New,
    Open(PathBuf),
    Close,
}

pub struct AppState {
    pub current_file: Mutex<Option<PathBuf>>,
    pub dirty: Mutex<bool>,
    pub pending_action: Mutex<PendingAction>,
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

fn atomic_write(target: &Path, content: &str) -> std::io::Result<()> {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let filename = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("document.md");
    use std::time::{SystemTime, UNIX_EPOCH};
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_name = format!(".{}.tmp.{}.{}", filename, std::process::id(), nonce);
    let temp_path = parent.join(temp_name);

    {
        use std::io::Write;
        let mut file = std::fs::File::create(&temp_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
    }

    if let Err(err) = std::fs::rename(&temp_path, target) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(err);
    }
    Ok(())
}

fn handle_save_request(window: &WebviewWindow, app: &AppHandle) {
    let state = app.state::<AppState>();
    let has_file = state.current_file.lock().unwrap().is_some();
    if has_file {
        let _ = window.eval(
            "window.__TAURI_INTERNALS__.invoke('save_content', { path: null, content: editorAPI.getMarkdown() });",
        );
    } else {
        prompt_save_as(window, app);
    }
}

fn is_save_choice(result: &MessageDialogResult) -> bool {
    match result {
        MessageDialogResult::Yes => true,
        MessageDialogResult::Custom(s) => s == "Save",
        _ => false,
    }
}

fn is_discard_choice(result: &MessageDialogResult) -> bool {
    match result {
        MessageDialogResult::No => true,
        MessageDialogResult::Custom(s) => s == "Discard",
        _ => false,
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
        let app_clone = app.clone();
        window
            .dialog()
            .message(format!("Save changes to {}?", title_name))
            .title("Unsaved Changes")
            .kind(MessageDialogKind::Warning)
            .buttons(MessageDialogButtons::YesNoCancelCustom(
                "Save".to_string(),
                "Discard".to_string(),
                "Cancel".to_string(),
            ))
            .show_with_result(move |result| {
                if is_save_choice(&result) {
                    *app_clone.state::<AppState>().pending_action.lock().unwrap() =
                        PendingAction::New;
                    let _ = window_clone.eval("editorAPI.save();");
                } else if is_discard_choice(&result) {
                    let state = window_clone.app_handle().state::<AppState>();
                    *state.current_file.lock().unwrap() = None;
                    *state.dirty.lock().unwrap() = false;
                    update_window_title(&window_clone, None, false);
                    let _ = window_clone
                        .eval("editorAPI.setMarkdown('', 'Untitled.md', 'file:///', true);");
                }
            });
    } else {
        *state.current_file.lock().unwrap() = None;
        *state.dirty.lock().unwrap() = false;
        update_window_title(window, None, false);
        let _ = window.eval("editorAPI.setMarkdown('', 'Untitled.md', 'file:///', true);");
    }
}

fn confirm_and_open_file(window: &WebviewWindow, path: PathBuf, app: &AppHandle) {
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
        let app_clone = app.clone();
        let target_path = path.clone();
        window
            .dialog()
            .message(format!("Save changes to {}?", title_name))
            .title("Unsaved Changes")
            .kind(MessageDialogKind::Warning)
            .buttons(MessageDialogButtons::YesNoCancelCustom(
                "Save".to_string(),
                "Discard".to_string(),
                "Cancel".to_string(),
            ))
            .show_with_result(move |result| {
                if is_save_choice(&result) {
                    *app_clone.state::<AppState>().pending_action.lock().unwrap() =
                        PendingAction::Open(target_path);
                    let _ = window_clone.eval("editorAPI.save();");
                } else if is_discard_choice(&result) {
                    open_file_at_path(&window_clone, target_path, &app_clone);
                }
            });
    } else {
        open_file_at_path(window, path, app);
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
                confirm_and_open_file(&window_clone, p, &app_clone);
            }
        });
}

fn prompt_save_as(window: &WebviewWindow, app: &AppHandle) {
    let window_clone = window.clone();
    let app_clone = app.clone();
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
            } else {
                *app_clone.state::<AppState>().pending_action.lock().unwrap() =
                    PendingAction::None;
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

    if let Err(e) = atomic_write(&target, &content) {
        *state.pending_action.lock().unwrap() = PendingAction::None;
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

    let pending = std::mem::replace(
        &mut *state.pending_action.lock().unwrap(),
        PendingAction::None,
    );
    match pending {
        PendingAction::New => {
            *state.current_file.lock().unwrap() = None;
            *state.dirty.lock().unwrap() = false;
            update_window_title(&window, None, false);
            let _ = window
                .eval("editorAPI.setMarkdown('', 'Untitled.md', 'file:///', true);");
        }
        PendingAction::Open(pending_path) => {
            open_file_at_path(&window, pending_path, &app);
        }
        PendingAction::Close => {
            let _ = window.close();
        }
        PendingAction::None => {
            let _ = window.eval(&format!(
                "editorAPI.setMarkdown({}, {}, {});",
                js_string(&content),
                js_string(filename),
                js_string(&base)
            ));
        }
    }

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
            let mut pending = state.pending_action.lock().unwrap();
            if let PendingAction::Open(path) = &*pending {
                let path_to_open = path.clone();
                *pending = PendingAction::None;
                open_file_at_path(&window, path_to_open, &app);
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
            if let Some(serde_json::Value::String(url_str)) = data {
                if let Ok(parsed) = Url::parse(&url_str) {
                    if parsed.scheme() == "file" {
                        if let Ok(file_path) = parsed.to_file_path() {
                            let ext = file_path
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("")
                                .to_lowercase();
                            if ext == "md" || ext == "markdown" || ext == "txt" {
                                confirm_and_open_file(&window, file_path, &app);
                                return;
                            }
                        }
                    }
                }
                let _ = open::that(&url_str);
            }
        }
        "command" => {
            let cmd = data
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            match cmd.as_str() {
                "new" => prompt_new_document(&window, &app),
                "open" => prompt_open_dialog(&window, &app),
                "save" => handle_save_request(&window, &app),
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
    let initial_pending = match pending_cli_file {
        Some(path) => PendingAction::Open(path),
        None => PendingAction::None,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            current_file: Mutex::new(None),
            dirty: Mutex::new(false),
            pending_action: Mutex::new(initial_pending),
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
                        "save" => handle_save_request(&window, &handle),
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
                        let app_clone = app.clone();
                        webview
                            .dialog()
                            .message(format!("Save changes to {} before closing?", filename))
                            .title("Unsaved Changes")
                            .kind(MessageDialogKind::Warning)
                            .buttons(MessageDialogButtons::YesNoCancelCustom(
                                "Save".to_string(),
                                "Discard".to_string(),
                                "Cancel".to_string(),
                            ))
                            .show_with_result(move |result| {
                                if is_save_choice(&result) {
                                    *app_clone
                                        .state::<AppState>()
                                        .pending_action
                                        .lock()
                                        .unwrap() = PendingAction::Close;
                                    let _ = webview_clone.eval("editorAPI.save();");
                                } else if is_discard_choice(&result) {
                                    *app_clone
                                        .state::<AppState>()
                                        .dirty
                                        .lock()
                                        .unwrap() = false;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_atomic_write_creates_and_overwrites() {
        let dir = std::env::temp_dir().join(format!("markdowndesk_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let file = dir.join("test_doc.md");

        assert!(atomic_write(&file, "first content").is_ok());
        assert_eq!(fs::read_to_string(&file).unwrap(), "first content");

        assert!(atomic_write(&file, "second content").is_ok());
        assert_eq!(fs::read_to_string(&file).unwrap(), "second content");

        let _ = fs::remove_file(&file);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn test_dialog_choices() {
        assert!(is_save_choice(&MessageDialogResult::Yes));
        assert!(is_save_choice(&MessageDialogResult::Custom("Save".to_string())));
        assert!(!is_save_choice(&MessageDialogResult::No));
        assert!(!is_save_choice(&MessageDialogResult::Cancel));
        assert!(!is_save_choice(&MessageDialogResult::Custom("Discard".to_string())));

        assert!(is_discard_choice(&MessageDialogResult::No));
        assert!(is_discard_choice(&MessageDialogResult::Custom("Discard".to_string())));
        assert!(!is_discard_choice(&MessageDialogResult::Yes));
        assert!(!is_discard_choice(&MessageDialogResult::Cancel));
        assert!(!is_discard_choice(&MessageDialogResult::Custom("Save".to_string())));
    }
}


use serde::{Deserialize, Serialize};
use tauri::{Manager, Window};

#[derive(Serialize, Deserialize)]
pub struct OverlayPosition {
    pub x: i32,
    pub y: i32,
}

#[tauri::command]
pub fn toggle_overlay(window: Window) -> Result<bool, String> {
    if let Some(overlay) = window.get_webview_window("overlay") {
        let is_visible = overlay.is_visible().map_err(|e| e.to_string())?;

        if is_visible {
            overlay.hide().map_err(|e| e.to_string())?;
            Ok(false)
        } else {
            overlay.show().map_err(|e| e.to_string())?;
            Ok(true)
        }
    } else {
        Err("Overlay window not found".to_string())
    }
}

#[tauri::command]
pub fn show_overlay(window: Window) -> Result<(), String> {
    if let Some(overlay) = window.get_webview_window("overlay") {
        overlay.show().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn hide_overlay(window: Window) -> Result<(), String> {
    if let Some(overlay) = window.get_webview_window("overlay") {
        overlay.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn set_overlay_position(window: Window, position: OverlayPosition) -> Result<(), String> {
    if let Some(overlay) = window.get_webview_window("overlay") {
        overlay
            .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: position.x,
                y: position.y,
            }))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

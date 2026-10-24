// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Correção para o erro de protocolo do Wayland no WebKitGTK
    // (desativa o compositor DMABUF quando a sessão é Wayland; ver Instruções)
    if std::env::var("WAYLAND_DISPLAY").is_ok()
        && std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err()
    {
        unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") };
    }
    gamefic_lib::run()
}

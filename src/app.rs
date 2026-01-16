use gpui::{
    px, size, App, AppContext, Application, Bounds, WindowBackgroundAppearance, WindowBounds,
    WindowKind, WindowOptions,
};
use gpui_component::Root;
use gpui_component_assets::Assets;

use crate::clipboard::start_clipboard_history;
use crate::hotkeys::setup_global_hotkey;
use crate::ui::popup::{bind_popup_keys, PopupView};
use std::sync::mpsc;

pub fn run() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        gpui_component::init(cx);

        let bounds = Bounds::centered(None, size(px(750.), px(500.0)), cx);
        bind_popup_keys(cx);
        let (clipboard_tx, clipboard_rx) = mpsc::channel();
        start_clipboard_history(cx, clipboard_tx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_background: WindowBackgroundAppearance::Transparent,
                    titlebar: None,
                    kind: WindowKind::PopUp,
                    is_resizable: false,
                    is_minimizable: false,
                    ..Default::default()
                },
                move |window, cx| {
                    let view = cx.new(|cx| PopupView::new(window, cx, clipboard_rx));
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )
            .unwrap();

        if let Err(err) = setup_global_hotkey(cx, window) {
            eprintln!("Failed to register global hotkey: {err}");
        }

        cx.activate(true);
    });
}

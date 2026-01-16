#[cfg(not(target_os = "windows"))]
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
#[cfg(not(target_os = "windows"))]
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use gpui::{App, WindowHandle};
use gpui_component::Root;
use std::sync::mpsc;
use std::time::Duration;

use crate::ui::popup::PopupView;

#[cfg(not(target_os = "windows"))]
const HOTKEY_MODS: Modifiers = Modifiers::ALT | Modifiers::SHIFT;
#[cfg(not(target_os = "windows"))]
const HOTKEY_KEY: Code = Code::KeyV;

#[cfg(not(target_os = "windows"))]
struct HotKeyRegistration {
    _manager: Option<GlobalHotKeyManager>,
    _hotkey: HotKey,
}

#[cfg(not(target_os = "windows"))]
impl Global for HotKeyRegistration {}

pub fn setup_global_hotkey(cx: &mut App, handle: WindowHandle<Root>) -> anyhow::Result<()> {
    let (event_tx, event_rx) = mpsc::channel::<()>();

    #[cfg(target_os = "windows")]
    {
        register_windows_hotkey(event_tx)?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            if event.state() == HotKeyState::Pressed {
                let _ = event_tx.send(());
            }
        }));

        let hotkey = HotKey::new(Some(HOTKEY_MODS), HOTKEY_KEY);
        let manager = GlobalHotKeyManager::new()?;
        manager.register(hotkey)?;
        cx.set_global(HotKeyRegistration {
            _manager: Some(manager),
            _hotkey: hotkey,
        });
    }

    cx.spawn(async move |cx| {
        loop {
            if event_rx.try_recv().is_ok() {
                let _ = cx.update(|cx| {
                    cx.activate(true);
                    let window = handle;
                    let _ = window.update(cx, |root, window, cx| {
                        if let Ok(view) = root.view().clone().downcast::<PopupView>() {
                            view.update(cx, |view, cx| {
                                view.toggle_visible(window, cx);
                            });
                        }
                    });
                });
            }

            cx.background_executor()
                .timer(Duration::from_millis(30))
                .await;
        }
    })
    .detach();

    Ok(())
}

#[cfg(target_os = "windows")]
fn register_windows_hotkey(event_tx: mpsc::Sender<()>) -> anyhow::Result<()> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        MOD_ALT, MOD_SHIFT, RegisterHotKey, VK_V,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_HOTKEY,
    };

    const HOTKEY_ID: i32 = 1;

    let (status_tx, status_rx) = mpsc::channel::<Result<(), String>>();
    std::thread::spawn(move || unsafe {
        let modifiers = MOD_ALT | MOD_SHIFT;
        if RegisterHotKey(std::ptr::null_mut(), HOTKEY_ID, modifiers, VK_V as u32) == 0 {
            let _ = status_tx.send(Err(std::io::Error::last_os_error().to_string()));
            return;
        }

        let _ = status_tx.send(Ok(()));
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            if msg.message == WM_HOTKEY && msg.wParam == HOTKEY_ID as usize {
                let _ = event_tx.send(());
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });

    match status_rx.recv_timeout(Duration::from_secs(2)) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(message)) => Err(anyhow::anyhow!(message)),
        Err(err) => Err(anyhow::anyhow!(
            "Timed out waiting for hotkey registration: {err}"
        )),
    }
}

use gpui::{
    actions, canvas, div, fill, img, list, point, prelude::*, px, relative, rgb, rgba, size,
    uniform_list, AnyElement, App, Bounds, ClipboardItem, Context, Element, ElementId,
    GlobalElementId, KeyBinding, LayoutId, ListAlignment, ListState, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ObjectFit, Pixels, Point, ScrollStrategy, ShapedLine,
    SharedString, Style, TextRun, UniformListScrollHandle, Window,
};
use gpui_component::{
    input::{Input, InputState},
    menu::{ContextMenuExt, PopupMenuItem},
    Icon, IconName, Sizable, WindowExt,
};
use sea_orm::DatabaseConnection;
use std::path::PathBuf;

use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;

use crate::storage::entity::Model;
use crate::storage::history::{delete_clipboard_entry, load_entries_page, open_db};
use crate::storage::path::default_db_path;

actions!(popup, [TogglePopup, MoveUp, MoveDown, ConfirmSelection]);

pub fn bind_popup_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", MoveUp, Some("Popup")),
        KeyBinding::new("down", MoveDown, Some("Popup")),
        KeyBinding::new("enter", ConfirmSelection, Some("Popup")),
        KeyBinding::new("enter", ConfirmSelection, Some("Input")),
    ]);
}

pub struct PopupView {
    is_visible: bool,
    search_input: gpui::Entity<InputState>,
    entries: Vec<Model>,
    search_query: String,
    selected_index: usize,
    db: Option<DatabaseConnection>,
    list_scroll: UniformListScrollHandle,
    detail_list_state: ListState,
    list_scroll_drag: Option<Point<Pixels>>,
    history_scrollbar_visible: bool,
    history_scrollbar_hide_gen: u64,
    last_scroll_offset: Option<Pixels>,
    is_loading: bool,
    has_more: bool,
    page_offset: u64,
    page_size: u64,
    load_generation: u64,
}

impl PopupView {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        clipboard_updates: std::sync::mpsc::Receiver<()>,
    ) -> Self {
        cx.observe_window_activation(window, |view, window, cx| {
            if window.is_window_active() {
                return;
            }
            view.hide(window, cx);
        })
        .detach();
        let search_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Search clipboard...")
                .clean_on_escape()
        });
        let view = Self {
            is_visible: true,
            search_input,
            entries: Vec::new(),
            search_query: String::new(),
            selected_index: 0,
            db: None,
            list_scroll: UniformListScrollHandle::new(),
            detail_list_state: ListState::new(1, ListAlignment::Top, px(20.)),
            list_scroll_drag: None,
            history_scrollbar_visible: false,
            history_scrollbar_hide_gen: 0,
            last_scroll_offset: None,
            is_loading: false,
            has_more: true,
            page_offset: 0,
            page_size: 100,
            load_generation: 0,
        };

        cx.on_next_frame(window, |view, window, cx| {
            cx.focus_view(&view.search_input, window);
        });

        cx.spawn(
            |view: gpui::WeakEntity<PopupView>, cx: &mut gpui::AsyncApp| {
                let mut async_cx = cx.clone();
                async move {
                    let db_path = match default_db_path() {
                        Ok(path) => path,
                        Err(err) => {
                            eprintln!("Failed to resolve clipboard database path: {err}");
                            return;
                        }
                    };

                    let db = match open_db(&db_path).await {
                        Ok(db) => db,
                        Err(err) => {
                            eprintln!("Failed to open clipboard database: {err}");
                            return;
                        }
                    };

                    if let Some(handle) = view.upgrade() {
                        let _ = async_cx.update_entity(&handle, |view: &mut PopupView, cx| {
                            view.db = Some(db);
                            view.reset_and_load(cx);
                        });
                    }
                }
            },
        )
        .detach();

        cx.spawn(
            move |view: gpui::WeakEntity<PopupView>, cx: &mut gpui::AsyncApp| {
                let mut async_cx = cx.clone();
                async move {
                    loop {
                        let mut received = false;
                        while clipboard_updates.try_recv().is_ok() {
                            received = true;
                        }
                        if received {
                            if let Some(handle) = view.upgrade() {
                                let _ =
                                    async_cx.update_entity(&handle, |view: &mut PopupView, cx| {
                                        view.reset_and_load(cx);
                                    });
                            } else {
                                break;
                            }
                        }

                        async_cx
                            .background_executor()
                            .timer(Duration::from_millis(30))
                            .await;
                    }
                }
            },
        )
        .detach();

        view
    }

    fn hide(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_visible {
            return;
        }
        window.minimize_window();
        self.is_visible = false;
        cx.notify();
    }

    fn show(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_visible {
            return;
        }
        window.activate_window();
        self.is_visible = true;
        cx.on_next_frame(window, |view, window, cx| {
            cx.focus_view(&view.search_input, window);
        });
        cx.notify();
    }

    fn on_toggle_action(&mut self, _: &TogglePopup, window: &mut Window, cx: &mut Context<Self>) {
        self.toggle_visible(window, cx);
    }

    fn on_move_up(&mut self, _: &MoveUp, _: &mut Window, cx: &mut Context<Self>) {
        self.move_selection(-1, cx);
    }

    fn on_move_down(&mut self, _: &MoveDown, _: &mut Window, cx: &mut Context<Self>) {
        self.move_selection(1, cx);
    }

    fn on_confirm_selection(
        &mut self,
        _: &ConfirmSelection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.copy_selected(cx);
        self.hide(window, cx);
    }

    fn move_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.entries.is_empty() {
            return;
        }
        let len = self.entries.len();
        if delta > 0 && self.selected_index + 1 >= len {
            self.load_entries(false, cx);
            return;
        }
        let next = (self.selected_index as isize + delta).clamp(0, len as isize - 1) as usize;
        if next != self.selected_index {
            self.selected_index = next;
            self.list_scroll
                .scroll_to_item(self.selected_index, ScrollStrategy::Center);
            self.detail_list_state.reset(1);
            cx.notify();
        }
    }

    fn refresh_entries(&mut self, cx: &mut Context<Self>) {
        let Some(_) = self.db else {
            return;
        };

        let generation = self.load_generation.wrapping_add(1);
        self.load_generation = generation;

        cx.spawn(
            move |view: gpui::WeakEntity<PopupView>, cx: &mut gpui::AsyncApp| {
                let mut async_cx = cx.clone();
                async move {
                    async_cx
                        .background_executor()
                        .timer(Duration::from_millis(500))
                        .await;
                    if let Some(handle) = view.upgrade() {
                        let _ = async_cx.update_entity(&handle, |view: &mut PopupView, cx| {
                            if view.load_generation != generation {
                                return;
                            }
                            view.reset_and_load(cx);
                        });
                    }
                }
            },
        )
        .detach();
    }

    fn copy_selected(&mut self, cx: &mut Context<Self>) {
        let Some(payload) = self.payload_for_index(self.selected_index) else {
            return;
        };
        self.copy_payload(payload, cx);
    }

    fn copy_entry_by_id(&mut self, id: i32, cx: &mut Context<Self>) {
        let Some(payload) = self.payload_for_id(id) else {
            return;
        };
        self.copy_payload(payload, cx);
    }

    fn copy_payload(&mut self, payload: String, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(payload));
        self.refresh_entries(cx);
    }

    fn payload_for_index(&self, index: usize) -> Option<String> {
        let entry = self.entries.get(index)?;
        Some(self.payload_for_entry(entry))
    }

    fn payload_for_id(&self, id: i32) -> Option<String> {
        let entry = self.entries.iter().find(|entry| entry.id == id)?;
        Some(self.payload_for_entry(entry))
    }

    fn payload_for_entry(&self, entry: &Model) -> String {
        entry
            .text_content
            .as_deref()
            .or(entry.file_paths.as_deref())
            .unwrap_or(entry.content.as_str())
            .to_string()
    }

    fn delete_entry(&mut self, id: i32, cx: &mut Context<Self>) {
        let Some(db) = self.db.clone() else {
            return;
        };

        cx.spawn(
            move |view: gpui::WeakEntity<PopupView>, cx: &mut gpui::AsyncApp| {
                let mut async_cx = cx.clone();
                async move {
                    if let Err(err) = delete_clipboard_entry(&db, id).await {
                        eprintln!("Failed to delete clipboard entry: {err}");
                        return;
                    }
                    if let Some(handle) = view.upgrade() {
                        let _ = async_cx.update_entity(&handle, |view: &mut PopupView, cx| {
                            view.reset_and_load(cx);
                        });
                    }
                }
            },
        )
        .detach();
    }

    fn select_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.entries.len() {
            return;
        }
        if index == self.selected_index {
            return;
        }
        self.selected_index = index;
        self.detail_list_state.reset(1);
        cx.notify();
    }

    fn reset_and_load(&mut self, cx: &mut Context<Self>) {
        self.selected_index = 0;
        self.page_offset = 0;
        self.has_more = true;
        self.is_loading = false;
        self.load_generation = self.load_generation.wrapping_add(1);
        self.detail_list_state.reset(1);
        self.list_scroll
            .scroll_to_item(self.selected_index, ScrollStrategy::Center);
        self.load_entries(true, cx);
    }

    fn load_entries(&mut self, replace: bool, cx: &mut Context<Self>) {
        if self.is_loading || !self.has_more {
            return;
        }
        let Some(db) = self.db.clone() else {
            return;
        };

        let query = self.search_query.trim().to_string();
        let query = if query.is_empty() { None } else { Some(query) };
        let offset = if replace { 0 } else { self.page_offset };
        let limit = self.page_size;
        let generation = self.load_generation.wrapping_add(1);
        self.load_generation = generation;
        self.is_loading = true;

        cx.spawn(
            move |view: gpui::WeakEntity<PopupView>, cx: &mut gpui::AsyncApp| {
                let mut async_cx = cx.clone();
                async move {
                    let entries =
                        match load_entries_page(&db, query.as_deref(), offset, limit).await {
                            Ok(entries) => entries,
                            Err(err) => {
                                eprintln!("Failed to load clipboard history: {err}");
                                Vec::new()
                            }
                        };
                    if let Some(handle) = view.upgrade() {
                        let _ = async_cx.update_entity(&handle, |view: &mut PopupView, cx| {
                            if view.load_generation != generation {
                                return;
                            }
                            view.is_loading = false;
                            if replace {
                                view.entries = entries;
                                view.page_offset = view.entries.len() as u64;
                                view.has_more = view.entries.len() as u64 == limit;
                                view.selected_index = 0;
                                view.detail_list_state.reset(1);
                                view.list_scroll
                                    .scroll_to_item(view.selected_index, ScrollStrategy::Center);
                                cx.notify();
                                return;
                            }
                            if entries.is_empty() {
                                view.has_more = false;
                                cx.notify();
                                return;
                            }
                            view.page_offset += entries.len() as u64;
                            view.has_more = entries.len() as u64 == limit;
                            view.entries.extend(entries);
                            cx.notify();
                        });
                    }
                }
            },
        )
        .detach();
    }

    fn maybe_load_more(&mut self, cx: &mut Context<Self>) {
        if self.is_loading || !self.has_more {
            return;
        }
        let Some((list_bounds, content_height)) = self.history_scroll_metrics() else {
            return;
        };
        let list_height = list_bounds.size.height;
        if list_height == px(0.) || content_height <= list_height {
            return;
        }
        let scroll_offset = -self.list_scroll.0.borrow().base_handle.offset().y;
        let remaining = (content_height - list_height - scroll_offset).max(px(0.));
        if remaining <= px(240.) {
            self.load_entries(false, cx);
        }
    }

    fn history_scroll_metrics(&self) -> Option<(Bounds<Pixels>, Pixels)> {
        let state = self.list_scroll.0.borrow();
        let list_bounds = state.base_handle.bounds();
        let content_height = state.last_item_size?.contents.height;
        Some((list_bounds, content_height))
    }

    fn render_history_scrollbar(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        const SCROLLBAR_WIDTH: Pixels = px(6.);
        const SCROLLBAR_MIN_HEIGHT: Pixels = px(24.);
        const SCROLLBAR_PADDING: Pixels = px(2.);

        let Some((list_bounds, content_height)) = self.history_scroll_metrics() else {
            return div().id("history-scrollbar");
        };

        let list_height = list_bounds.size.height;
        if list_height == px(0.) || content_height <= list_height {
            self.history_scrollbar_visible = false;
            return div().id("history-scrollbar");
        }

        let scroll_offset = -self.list_scroll.0.borrow().base_handle.offset().y;
        let max_scroll = (content_height - list_height).max(px(1.));
        let percentage = (scroll_offset / max_scroll).clamp(0., 1.);
        let should_show = self
            .last_scroll_offset
            .is_some_and(|last| last != scroll_offset);
        self.last_scroll_offset = Some(scroll_offset);
        if should_show {
            self.show_history_scrollbar(cx);
        }
        if !self.history_scrollbar_visible {
            return div().id("history-scrollbar");
        }

        let total_padding = SCROLLBAR_PADDING + SCROLLBAR_PADDING;
        let thumb_height = ((list_height / content_height) * list_height)
            .clamp(SCROLLBAR_MIN_HEIGHT, list_height - total_padding);
        let thumb_top =
            (list_height - thumb_height - total_padding) * percentage + SCROLLBAR_PADDING;

        let entity = cx.entity();
        let scroll_handle = self.list_scroll.0.borrow().base_handle.clone();

        div()
            .id("history-scrollbar")
            .absolute()
            .top(thumb_top)
            .right_1()
            .h(thumb_height)
            .w(SCROLLBAR_WIDTH)
            .bg(rgba(0xffffff24))
            .hover(|style| style.bg(rgba(0xffffff3a)))
            .rounded_lg()
            .child(
                canvas(
                    |_, _, _| (),
                    move |thumb_bounds, _, window, _| {
                        window.on_mouse_event({
                            let entity = entity.clone();
                            move |ev: &MouseDownEvent, _, _, cx| {
                                if !thumb_bounds.contains(&ev.position) {
                                    return;
                                }

                                entity.update(cx, |view, _| {
                                    view.list_scroll_drag = Some(
                                        ev.position - thumb_bounds.origin - list_bounds.origin,
                                    );
                                });
                            }
                        });

                        window.on_mouse_event({
                            let entity = entity.clone();
                            move |_: &MouseUpEvent, _, _, cx| {
                                entity.update(cx, |view, _| {
                                    view.list_scroll_drag = None;
                                });
                            }
                        });

                        window.on_mouse_event(move |ev: &MouseMoveEvent, _, _, cx| {
                            if !ev.dragging() {
                                return;
                            }

                            let Some(drag_pos) = entity.read(cx).list_scroll_drag else {
                                return;
                            };

                            let percentage = ((ev.position.y - list_bounds.origin.y + drag_pos.y)
                                / list_bounds.size.height)
                                .clamp(0., 1.);
                            let offset_y = (max_scroll * percentage).clamp(px(0.), max_scroll);
                            scroll_handle.set_offset(point(px(0.), -offset_y));
                            cx.notify(entity.entity_id());
                        })
                    },
                )
                .size_full(),
            )
    }

    fn show_history_scrollbar(&mut self, cx: &mut Context<Self>) {
        self.history_scrollbar_visible = true;
        self.history_scrollbar_hide_gen = self.history_scrollbar_hide_gen.wrapping_add(1);
        let hide_gen = self.history_scrollbar_hide_gen;

        cx.spawn(
            move |view: gpui::WeakEntity<PopupView>, cx: &mut gpui::AsyncApp| {
                let mut async_cx = cx.clone();
                async move {
                    async_cx
                        .background_executor()
                        .timer(Duration::from_secs(2))
                        .await;
                    if let Some(handle) = view.upgrade() {
                        let _ = async_cx.update_entity(&handle, |view: &mut PopupView, cx| {
                            if view.history_scrollbar_hide_gen == hide_gen {
                                view.history_scrollbar_visible = false;
                                cx.notify();
                            }
                        });
                    }
                }
            },
        )
        .detach();

        cx.notify();
    }

    pub(crate) fn toggle_visible(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_visible {
            self.hide(window, cx);
        } else {
            self.show(window, cx);
        }
    }
}

impl Render for PopupView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let query = self.search_input.read(cx).value().trim().to_string();
        if query != self.search_query {
            self.search_query = query;
            self.reset_and_load(cx);
        }

        if !self.is_visible {
            return div()
                .size_full()
                .on_action(cx.listener(Self::on_toggle_action));
        }

        let root = div()
            .size_full()
            .flex()
            .flex_col()
            .gap_1()
            .bg(rgb(0x1a1a1a))
            .text_color(rgb(0xf3f4f6))
            .text_sm()
            .p_1p5()
            .key_context("Popup")
            .on_action(cx.listener(Self::on_toggle_action))
            .on_action(cx.listener(Self::on_move_up))
            .on_action(cx.listener(Self::on_move_down))
            .on_action(cx.listener(Self::on_confirm_selection))
            .child(
                div().w_full().child(
                    Input::new(&self.search_input)
                        .prefix(Icon::new(IconName::Search).small())
                        .appearance(false),
                ),
            )
            .child(div().w_full().h(px(1.)).bg(rgba(0xffffff20)).mb_1())
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .w_1_3()
                            .flex_shrink_0()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .relative()
                                    .flex_1()
                                    .child(history_list(
                                        &self.entries,
                                        self.search_query.clone(),
                                        self.is_loading,
                                        cx,
                                        self.list_scroll.clone(),
                                    ))
                                    .child(self.render_history_scrollbar(window, cx)),
                            ),
                    )
                    .child(div().w(px(1.)).bg(rgba(0xffffff20)))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .justify_between()
                            .gap_3()
                            .child(div().flex_1().px_1().text_color(rgb(0xf1f5f9)).child(
                                detail_body_list(
                                    &self.entries,
                                    self.selected_index,
                                    &self.search_query,
                                    cx,
                                    self.detail_list_state.clone(),
                                ),
                            ))
                            .child(
                                div()
                                    .text_color(rgb(0x9aa4af))
                                    .child(detail_info_panel(&self.entries, self.selected_index)),
                            ),
                    ),
            );

        self.maybe_load_more(cx);
        root
    }
}

fn history_list(
    entries: &[Model],
    query: String,
    is_loading: bool,
    cx: &mut Context<PopupView>,
    scroll_handle: UniformListScrollHandle,
) -> AnyElement {
    if entries.is_empty() {
        if is_loading && query.is_empty() {
            return div()
                .rounded_md()
                .bg(rgb(0x222834))
                .p_2()
                .text_color(rgb(0x9aa4af))
                .child("Loading history...")
                .into_any_element();
        }
        if !query.is_empty() {
            return div()
                .rounded_md()
                .bg(rgb(0x222834))
                .p_2()
                .text_color(rgb(0x9aa4af))
                .child("No matches for your search.")
                .into_any_element();
        }
        return div()
            .rounded_md()
            .bg(rgb(0x222834))
            .p_2()
            .text_color(rgb(0x9aa4af))
            .child("No clipboard history yet.")
            .into_any_element();
    }

    uniform_list(
        "history-list",
        entries.len(),
        cx.processor(move |view, range, _window, _cx| {
            let mut items = Vec::new();
            let view_handle = _cx.entity();
            for index in range {
                let entry: &Model = &view.entries[index];
                let is_selected = index == view.selected_index;
                let background = if is_selected {
                    rgb(0x313131)
                } else {
                    rgba(0x00000000)
                };
                let text_color = rgb(0xd2d8df);
                let entry_id = entry.id;
                let mut item = div()
                    .id(index)
                    .rounded_md()
                    .bg(background)
                    .w_full()
                    .text_color(text_color)
                    .on_mouse_down(
                        MouseButton::Left,
                        _cx.listener(move |view, event: &MouseDownEvent, window, cx| {
                            view.select_index(index, cx);
                            if event.click_count >= 2 {
                                view.copy_selected(cx);
                                view.hide(window, cx);
                            }
                        }),
                    );
                if !is_selected {
                    item = item.hover(|style| style.bg(rgba(0x31313150)));
                }
                let view_handle = view_handle.clone();
                let item = item.context_menu(move |menu, _window, _cx| {
                    let view_handle = view_handle.clone();
                    menu.item(PopupMenuItem::new("Copy").on_click({
                        let view_handle = view_handle.clone();
                        move |_, _, cx| {
                            view_handle.update(cx, |view, cx| {
                                view.copy_entry_by_id(entry_id, cx);
                            });
                        }
                    }))
                    .separator()
                    .item(PopupMenuItem::new("Delete").on_click({
                        let view_handle = view_handle.clone();
                        move |_, window, cx| {
                            let view_handle = view_handle.clone();
                            window.open_dialog(cx, move |dialog, _, _| {
                                let view_handle = view_handle.clone();
                                dialog
                                    .title("Delete entry")
                                    .confirm()
                                    .child("Delete this clipboard entry?")
                                    .on_ok(move |_, _, cx| {
                                        view_handle.update(cx, |view, cx| {
                                            view.delete_entry(entry_id, cx);
                                        });
                                        true
                                    })
                                    .on_cancel(|_, _, _| true)
                            });
                        }
                    }))
                });
                let query = view.search_query.clone();
                let preview_text = history_preview_text(entry);
                if entry.content_type == "image" {
                    if let Some(path) = entry.image_path.as_ref() {
                        let thumbnail = div()
                            .w_full()
                            .h(px(26.))
                            .overflow_hidden()
                            .rounded_sm()
                            .bg(rgba(0xffffff0f))
                            .child(
                                img(PathBuf::from(path))
                                    .w_full()
                                    .h_full()
                                    .object_fit(ObjectFit::Cover),
                            );
                        items.push(
                            item.p_2()
                                .h(px(36.))
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(thumbnail),
                        );
                    } else {
                        let item = item.p_2().h(px(36.)).text_ellipsis();
                        items.push(item.child(HighlightedText::new(preview_text, query)));
                    }
                } else {
                    let item = item.p_2().h(px(36.)).text_ellipsis();
                    items.push(item.child(HighlightedText::new(preview_text, query)));
                }
            }
            items
        }),
    )
    .h_full()
    .w_full()
    .track_scroll(scroll_handle)
    .into_any_element()
}

fn history_preview_text(entry: &Model) -> String {
    if entry.content_type == "link" {
        let url = entry.link_url.as_deref().or(entry.text_content.as_deref());
        if let Some(url) = url {
            if !url.trim().is_empty() {
                return url.to_string();
            }
        }
    }
    let text = entry
        .text_content
        .as_deref()
        .or(entry.file_paths.as_deref())
        .unwrap_or(entry.content.as_str());
    let preview_lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    if preview_lines.is_empty() {
        text.trim().to_string()
    } else {
        preview_lines.join(" ")
    }
}

fn detail_body_list(
    entries: &[Model],
    selected_index: usize,
    query: &str,
    cx: &mut Context<PopupView>,
    list_state: ListState,
) -> AnyElement {
    if let Some(entry) = entries.get(selected_index) {
        if entry.content_type == "image" {
            if let Some(path) = entry.image_path.as_deref() {
                return detail_image_body_list(path, cx, list_state);
            }
        }
        if entry.content_type == "link" {
            if let Some(panel) = detail_link_body_list(entry, query, cx, list_state.clone()) {
                return panel;
            }
        }
    }

    let body = detail_body(entries, selected_index);
    let query = query.to_string();
    let lines: Vec<SharedString> = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string().into())
        .collect();
    list(
        list_state,
        cx.processor(move |_view, _index, _window, _cx| {
            let mut container = div().w_full().whitespace_normal().flex().flex_col().gap_1();
            for line in &lines {
                container = container.child(HighlightedText::new_with_mode(
                    line.clone(),
                    query.clone(),
                    HighlightMatchMode::AnyToken,
                ));
            }
            container.into_any_element()
        }),
    )
    .h_full()
    .w_full()
    .into_any_element()
}

fn detail_image_body_list(
    image_path: &str,
    cx: &mut Context<PopupView>,
    list_state: ListState,
) -> AnyElement {
    let image_path = PathBuf::from(image_path);
    list(
        list_state,
        cx.processor(move |_view, _index, _window, _cx| {
            img(image_path.clone())
                .w_full()
                .object_fit(ObjectFit::Contain)
                .into_any_element()
        }),
    )
    .h_full()
    .w_full()
    .into_any_element()
}

fn detail_link_body_list(
    entry: &Model,
    query: &str,
    cx: &mut Context<PopupView>,
    list_state: ListState,
) -> Option<AnyElement> {
    let title = entry.link_title.as_deref().and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });
    let url = entry
        .link_url
        .as_deref()
        .or(entry.text_content.as_deref())
        .or(Some(entry.content.as_str()))
        .and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
    let description = entry.link_description.as_deref().and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });
    let site_label = entry.link_site_name.as_deref().and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(format!("Source: {trimmed}"))
        }
    });

    if title.is_none() && url.is_none() && description.is_none() && site_label.is_none() {
        return None;
    }

    let title = title.map(SharedString::from);
    let url = url.map(SharedString::from);
    let description = description.map(SharedString::from);
    let site_label = site_label.map(SharedString::from);
    let query = query.to_string();

    Some(
        list(
            list_state,
            cx.processor(move |_view, _index, _window, _cx| {
                let mut container = div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .rounded_md()
                    .bg(rgba(0xffffff08))
                    .p_2();

                if let Some(title) = title.clone() {
                    container =
                        container.child(div().text_color(rgb(0xf1f5f9)).whitespace_normal().child(
                            HighlightedText::new_with_mode(
                                title,
                                query.clone(),
                                HighlightMatchMode::AnyToken,
                            ),
                        ));
                }

                if let Some(url) = url.clone() {
                    let url_for_open = url.to_string();
                    container = container.child(
                        div()
                            .id("detail-link-url")
                            .text_color(rgb(0x93c5fd))
                            .whitespace_normal()
                            .cursor_pointer()
                            .hover(|style| style.underline())
                            .on_click(move |_, _, cx| cx.open_url(&url_for_open))
                            .child(HighlightedText::new_with_mode(
                                url,
                                query.clone(),
                                HighlightMatchMode::AnyToken,
                            )),
                    );
                }

                if let Some(description) = description.clone() {
                    container =
                        container.child(div().text_color(rgb(0x9aa4af)).whitespace_normal().child(
                            HighlightedText::new_with_mode(
                                description,
                                query.clone(),
                                HighlightMatchMode::AnyToken,
                            ),
                        ));
                }

                if let Some(site_label) = site_label.clone() {
                    container = container.child(div().text_xs().text_color(rgb(0x94a3b8)).child(
                        HighlightedText::new_with_mode(
                            site_label,
                            query.clone(),
                            HighlightMatchMode::AnyToken,
                        ),
                    ));
                }

                container.into_any_element()
            }),
        )
        .h_full()
        .w_full()
        .into_any_element(),
    )
}

fn detail_info_panel(entries: &[Model], selected_index: usize) -> AnyElement {
    let Some(entry) = entries.get(selected_index) else {
        return div()
            .px_1()
            .text_xs()
            .text_color(rgb(0x9aa4af))
            .child("No entry selected.")
            .into_any_element();
    };

    let source = source_label(entry);
    let content_type = format_content_type(&entry.content_type);
    let (characters, words) = entry_metrics(entry);

    let mut items = vec![
        ("Source".to_string(), source),
        ("Type".to_string(), content_type),
    ];

    if entry.content_type == "link" {
        if let Some(title) = entry.link_title.as_deref() {
            if !title.trim().is_empty() {
                items.push(("Title".to_string(), title.to_string()));
            }
        }
        if let Some(site_name) = entry.link_site_name.as_deref() {
            if !site_name.trim().is_empty() {
                items.push(("Site".to_string(), site_name.to_string()));
            }
        }
        let url = entry.link_url.as_deref().or(entry.text_content.as_deref());
        if let Some(url) = url {
            if !url.trim().is_empty() {
                items.push(("URL".to_string(), url.to_string()));
            }
        }
    }

    items.push(("Characters".to_string(), characters.to_string()));
    items.push(("Words".to_string(), words.to_string()));

    let mut list = div().flex().flex_col().gap_1();
    let item_count = items.len();
    for (index, (label, value)) in items.into_iter().enumerate() {
        list = list.child(
            div()
                .flex()
                .items_start()
                .justify_between()
                .gap_2()
                .text_xs()
                .child(div().text_color(rgb(0xb6c0cb)).child(label))
                .child(
                    div()
                        .max_w(px(400.))
                        .text_color(rgb(0xf1f5f9))
                        .text_ellipsis()
                        .child(value),
                ),
        );
        if index + 1 < item_count {
            list = list.child(div().h(px(1.)).bg(rgba(0xffffff12)));
        }
    }

    div()
        .px_1()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_color(rgb(0xf1f5f9))
                .child("Information")
                .text_xs(),
        )
        .child(list)
        .into_any_element()
}

fn detail_body(entries: &[Model], selected_index: usize) -> String {
    if let Some(entry) = entries.get(selected_index) {
        if entry.content_type == "link" {
            let mut lines = Vec::new();
            if let Some(title) = entry.link_title.as_deref() {
                if !title.trim().is_empty() {
                    lines.push(title.to_string());
                }
            }
            let url = entry
                .link_url
                .as_deref()
                .or(entry.text_content.as_deref())
                .unwrap_or(entry.content.as_str());
            if !url.trim().is_empty() {
                lines.push(url.to_string());
            }
            if let Some(description) = entry.link_description.as_deref() {
                if !description.trim().is_empty() {
                    lines.push(description.to_string());
                }
            }
            if !lines.is_empty() {
                return lines.join("\n\n");
            }
        }
        if let Some(text) = entry.text_content.as_ref() {
            text.clone()
        } else if let Some(paths) = entry.file_paths.as_ref() {
            paths.clone()
        } else if let Some(path) = entry.image_path.as_ref() {
            format!("Image saved at {path}")
        } else {
            entry.content.clone()
        }
    } else {
        "Select a clipboard item to see details.".to_string()
    }
}

fn source_label(entry: &Model) -> String {
    if let Some(title) = entry.source_app_title.as_deref() {
        if !title.trim().is_empty() {
            return title.to_string();
        }
    }

    if let Some(path) = entry.source_exe_path.as_deref() {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            if let Some(file_name) = std::path::Path::new(trimmed)
                .file_name()
                .and_then(|name| name.to_str())
            {
                return file_name.to_string();
            }
            return trimmed.to_string();
        }
    }

    "Unknown".to_string()
}

fn format_content_type(content_type: &str) -> String {
    match content_type {
        "text" => "Text",
        "image" => "Image",
        "files" => "Files",
        "link" => "Link",
        other => other,
    }
    .to_string()
}

fn entry_metrics(entry: &Model) -> (usize, usize) {
    let text = if entry.content_type == "image" {
        entry.ocr_text.as_deref().unwrap_or("")
    } else if entry.content_type == "link" {
        entry
            .link_description
            .as_deref()
            .or(entry.link_title.as_deref())
            .or(entry.link_url.as_deref())
            .or(entry.text_content.as_deref())
            .unwrap_or(entry.content.as_str())
    } else {
        entry
            .text_content
            .as_deref()
            .or(entry.file_paths.as_deref())
            .unwrap_or(entry.content.as_str())
    };
    let characters = text.chars().count();
    let words = text.unicode_words().count();
    (characters, words)
}

enum HighlightMatchMode {
    AllTokens,
    AnyToken,
}

struct HighlightedText {
    query: SharedString,
    display_text: SharedString,
    match_mode: HighlightMatchMode,
}

impl HighlightedText {
    fn new(text: impl Into<SharedString>, query: impl Into<SharedString>) -> Self {
        Self::new_with_mode(text, query, HighlightMatchMode::AllTokens)
    }

    fn new_with_mode(
        text: impl Into<SharedString>,
        query: impl Into<SharedString>,
        match_mode: HighlightMatchMode,
    ) -> Self {
        let text: SharedString = text.into();
        let display_text: SharedString = text.replace('\n', " ").into();
        Self {
            query: query.into(),
            display_text,
            match_mode,
        }
    }
}

impl IntoElement for HighlightedText {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for HighlightedText {
    type RequestLayoutState = ();
    type PrepaintState = Option<(ShapedLine, Vec<std::ops::Range<usize>>)>;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let runs = vec![TextRun {
            len: self.display_text.len(),
            font: style.font(),
            color: style.color,
            background_color: None,
            underline: style.underline,
            strikethrough: None,
        }];
        let line =
            window
                .text_system()
                .shape_line(self.display_text.clone(), font_size, &runs, None);
        let ranges = match_ranges(&self.display_text, &self.query, &self.match_mode);
        Some((line, ranges))
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let (line, ranges) = prepaint.take().unwrap();
        if !ranges.is_empty() {
            let highlight_color = rgba(0x3311ff30);
            for range in ranges {
                let start_x = bounds.left() + line.x_for_index(range.start);
                let end_x = bounds.left() + line.x_for_index(range.end);
                if end_x <= start_x {
                    continue;
                }
                let quad = fill(
                    Bounds::new(
                        point(start_x, bounds.top()),
                        size(end_x - start_x, bounds.bottom() - bounds.top()),
                    ),
                    highlight_color,
                );
                window.paint_quad(quad);
            }
        }
        line.paint(bounds.origin, window.line_height(), window, cx)
            .unwrap();
    }
}

fn match_ranges(
    text: &str,
    query: &str,
    match_mode: &HighlightMatchMode,
) -> Vec<std::ops::Range<usize>> {
    let positions = token_prefix_positions(text, query, match_mode);
    positions_to_ranges(text, &positions)
}

fn token_prefix_positions(text: &str, query: &str, match_mode: &HighlightMatchMode) -> Vec<usize> {
    let tokens: Vec<&str> = query.split_whitespace().collect();
    if tokens.is_empty() {
        return Vec::new();
    }

    let word_starts = word_start_positions(text);
    let mut all_positions = Vec::new();

    for token in tokens {
        if token.is_empty() {
            continue;
        }
        let positions = prefix_positions_for_token(text, &word_starts, token);
        match (match_mode, positions) {
            (HighlightMatchMode::AllTokens, Some(positions)) => {
                all_positions.extend(positions);
            }
            (HighlightMatchMode::AllTokens, None) => {
                return Vec::new();
            }
            (HighlightMatchMode::AnyToken, Some(positions)) => {
                all_positions.extend(positions);
            }
            (HighlightMatchMode::AnyToken, None) => {}
        }
    }

    if all_positions.is_empty() {
        return Vec::new();
    }

    all_positions.sort_unstable();
    all_positions.dedup();
    all_positions
}

fn word_start_positions(text: &str) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut prev_is_word = false;
    for (byte_index, ch) in text.char_indices() {
        let is_word = ch.is_ascii_alphanumeric();
        if is_word && !prev_is_word {
            starts.push(byte_index);
        }
        prev_is_word = is_word;
    }
    starts
}

fn prefix_positions_for_token(
    text: &str,
    word_starts: &[usize],
    token: &str,
) -> Option<Vec<usize>> {
    let token_chars: Vec<char> = token.chars().flat_map(|c| c.to_lowercase()).collect();
    if token_chars.is_empty() {
        return Some(Vec::new());
    }

    for &start in word_starts {
        let mut positions = Vec::new();
        let mut matched = true;
        let mut token_index = 0;
        let mut cursor = start;
        while token_index < token_chars.len() {
            if cursor >= text.len() {
                matched = false;
                break;
            }
            let ch = text[cursor..].chars().next().unwrap();
            let mut ch_lower_iter = ch.to_lowercase();
            let ch_lower = ch_lower_iter.next().unwrap();
            if ch_lower != token_chars[token_index] {
                matched = false;
                break;
            }
            positions.push(cursor);
            cursor += ch.len_utf8();
            token_index += 1;
        }

        if matched {
            return Some(positions);
        }
    }

    None
}

fn positions_to_ranges(text: &str, positions: &[usize]) -> Vec<std::ops::Range<usize>> {
    if positions.is_empty() {
        return Vec::new();
    }

    let mut ranges = Vec::new();
    let mut current_start = positions[0];
    let mut current_end = current_start + char_len_at(text, current_start);

    for &pos in positions.iter().skip(1) {
        let len = char_len_at(text, pos);
        if pos == current_end {
            current_end = pos + len;
        } else {
            ranges.push(current_start..current_end);
            current_start = pos;
            current_end = pos + len;
        }
    }

    ranges.push(current_start..current_end);
    ranges
}

fn char_len_at(text: &str, byte_index: usize) -> usize {
    text[byte_index..]
        .chars()
        .next()
        .map(|ch| ch.len_utf8())
        .unwrap_or(0)
}

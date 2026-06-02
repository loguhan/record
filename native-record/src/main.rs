#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{
    egui::{
        self, Align, Align2, CentralPanel, Color32, Context, CornerRadius, FontData,
        FontDefinitions, FontFamily, FontId, Frame, Label, Layout, Margin, Pos2, Rect, Response,
        RichText, ScrollArea, Sense, Stroke, StrokeKind, TextEdit, TopBottomPanel, Ui, UiBuilder,
        Vec2, ViewportCommand, WindowLevel,
    },
    NativeOptions, Renderer,
};
use record_native::store::{
    create_task, default_store_path, load_or_create_store, save_tasks_at_path, update_task, Task,
    TaskDraft,
};
use std::{hash::Hash, path::PathBuf, sync::Arc};

const APP_ICON_BYTES: &[u8] = include_bytes!("../../src-tauri/icons/icon.png");

fn main() -> eframe::Result {
    let viewport = egui::ViewportBuilder::default()
        .with_title("Record")
        .with_inner_size([420.0, 600.0])
        .with_min_inner_size([340.0, 460.0])
        .with_icon(Arc::new(load_window_icon()));

    eframe::run_native(
        "Record",
        NativeOptions {
            viewport,
            renderer: Renderer::Wgpu,
            ..Default::default()
        },
        Box::new(|creation_context| Ok(Box::new(RecordApp::new(creation_context)))),
    )
}

struct RecordApp {
    store_path: PathBuf,
    tasks: Vec<Task>,
    error: Option<String>,
    editor: Option<EditorState>,
    delete_candidate: Option<String>,
    always_on_top: bool,
}

#[derive(Clone)]
struct EditorState {
    mode: EditorMode,
    draft: TaskDraft,
    error: Option<String>,
}

#[derive(Clone)]
enum EditorMode {
    Create,
    Edit(String),
}

enum PendingAction {
    Toggle(String, bool),
    Edit(String),
    Delete(String),
}

impl RecordApp {
    fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        install_system_fonts(&creation_context.egui_ctx);
        configure_style(&creation_context.egui_ctx);

        let store_path = default_store_path();
        let (tasks, error) = match load_or_create_store(&store_path) {
            Ok(store) => (store.tasks, None),
            Err(error) => (Vec::new(), Some(error)),
        };

        Self {
            store_path,
            tasks,
            error,
            editor: None,
            delete_candidate: None,
            always_on_top: false,
        }
    }

    fn open_create_editor(&mut self) {
        self.editor = Some(EditorState {
            mode: EditorMode::Create,
            draft: TaskDraft {
                title: String::new(),
                note: String::new(),
                due_date: String::new(),
            },
            error: None,
        });
    }

    fn open_edit_editor(&mut self, task_id: &str) {
        if let Some(task) = self.tasks.iter().find(|task| task.id == task_id) {
            self.editor = Some(EditorState {
                mode: EditorMode::Edit(task.id.clone()),
                draft: TaskDraft {
                    title: task.title.clone(),
                    note: task.note.clone(),
                    due_date: task.due_date.clone().unwrap_or_default(),
                },
                error: None,
            });
        }
    }

    fn persist(&mut self) {
        match save_tasks_at_path(&self.store_path, self.tasks.clone()) {
            Ok(store) => {
                self.tasks = store.tasks;
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn save_editor(&mut self) {
        let Some(editor) = self.editor.clone() else {
            return;
        };

        let result = match editor.mode {
            EditorMode::Create => create_task(&editor.draft).map(|task| {
                self.tasks.push(task);
            }),
            EditorMode::Edit(task_id) => {
                let Some(index) = self.tasks.iter().position(|task| task.id == task_id) else {
                    return;
                };
                update_task(&self.tasks[index], &editor.draft).map(|task| {
                    self.tasks[index] = task;
                })
            }
        };

        match result {
            Ok(()) => {
                self.editor = None;
                self.persist();
            }
            Err(error) => {
                if let Some(editor) = &mut self.editor {
                    editor.error = Some(error);
                }
            }
        }
    }

    fn task_counts(&self) -> (usize, usize) {
        let completed = self.tasks.iter().filter(|task| task.done).count();
        (self.tasks.len() - completed, completed)
    }

    fn toggle_always_on_top(&mut self, ctx: &Context) {
        self.always_on_top = !self.always_on_top;
        let level = if self.always_on_top {
            WindowLevel::AlwaysOnTop
        } else {
            WindowLevel::Normal
        };
        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(level));
    }
}

impl eframe::App for RecordApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let theme = theme_for_ctx(ctx);

        TopBottomPanel::top("record-header")
            .exact_height(58.0)
            .frame(
                Frame::NONE
                    .fill(theme.header)
                    .inner_margin(Margin::symmetric(14, 8)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    draw_small_logo(ui);
                    ui.add_space(7.0);
                    ui.vertical(|ui| {
                        ui.add_space(1.0);
                        ui.label(
                            RichText::new("Record")
                                .strong()
                                .size(14.0)
                                .color(theme.text),
                        );
                        let (open, completed) = self.task_counts();
                        ui.label(
                            RichText::new(format!("{open} 个待办 · {completed} 个已完成"))
                                .size(12.0)
                                .color(theme.muted),
                        );
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if pill_button(ui, IconKind::Add, "新建", ButtonTone::Primary, false)
                            .clicked()
                        {
                            self.open_create_editor();
                        }
                        ui.add_space(4.0);
                        let pin_label = if self.always_on_top {
                            "已置顶"
                        } else {
                            "置顶"
                        };
                        if pill_button(
                            ui,
                            IconKind::Pin,
                            pin_label,
                            ButtonTone::Quiet,
                            self.always_on_top,
                        )
                        .clicked()
                        {
                            self.toggle_always_on_top(ctx);
                        }
                    });
                });
            });

        CentralPanel::default()
            .frame(Frame::NONE.fill(theme.background))
            .show(ctx, |ui| {
                ui.add_space(12.0);
                if let Some(error) = &self.error {
                    Frame::NONE
                        .fill(theme.danger_soft)
                        .stroke(Stroke::new(1.0, theme.danger_border))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.label(RichText::new(error).size(12.0).color(theme.danger));
                        });
                    ui.add_space(10.0);
                }

                if self.tasks.is_empty() {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        ui.add_space(136.0);
                        draw_empty_mark(ui);
                        ui.add_space(12.0);
                        ui.label(
                            RichText::new("还没有任务")
                                .strong()
                                .size(15.0)
                                .color(theme.text),
                        );
                        ui.label(
                            RichText::new("新建一条任务，标题、备注和日期就能开始用了")
                                .size(12.0)
                                .color(theme.muted),
                        );
                        ui.add_space(14.0);
                        if pill_button(ui, IconKind::Add, "新建任务", ButtonTone::Primary, false)
                            .clicked()
                        {
                            self.open_create_editor();
                        }
                    });
                } else {
                    let mut pending_action = None;
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add_space(2.0);
                            Frame::NONE
                                .fill(theme.panel)
                                .stroke(Stroke::new(1.0, theme.border))
                                .corner_radius(CornerRadius::same(9))
                                .inner_margin(Margin::ZERO)
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    for (index, task) in self.tasks.iter().enumerate() {
                                        if let Some(action) = draw_task_row(ui, task) {
                                            pending_action = Some(action);
                                        }

                                        if index + 1 < self.tasks.len() {
                                            let y = ui.cursor().min.y;
                                            let rect = ui.max_rect();
                                            ui.painter().line_segment(
                                                [
                                                    Pos2::new(rect.left() + 42.0, y),
                                                    Pos2::new(rect.right() - 10.0, y),
                                                ],
                                                Stroke::new(1.0, theme.divider),
                                            );
                                        }
                                    }
                                });
                            ui.add_space(12.0);
                        });

                    if let Some(action) = pending_action {
                        match action {
                            PendingAction::Toggle(task_id, done) => {
                                if let Some(task) =
                                    self.tasks.iter_mut().find(|task| task.id == task_id)
                                {
                                    task.done = done;
                                    task.updated_at = chrono::Utc::now()
                                        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                                    self.persist();
                                }
                            }
                            PendingAction::Edit(task_id) => self.open_edit_editor(&task_id),
                            PendingAction::Delete(task_id) => self.delete_candidate = Some(task_id),
                        }
                    }
                }
            });

        self.draw_editor(ctx);
        self.draw_delete_confirm(ctx);
    }
}

impl RecordApp {
    fn draw_editor(&mut self, ctx: &Context) {
        let Some(editor) = &mut self.editor else {
            return;
        };

        let title = match editor.mode {
            EditorMode::Create => "新建任务",
            EditorMode::Edit(_) => "编辑任务",
        };
        let mut open = true;
        let mut save_clicked = false;
        let mut cancel_clicked = false;

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .default_width(340.0)
            .frame(dialog_frame(ctx))
            .open(&mut open)
            .show(ctx, |ui| {
                let theme = theme(ui);
                ui.label(RichText::new("标题").size(12.0).color(theme.muted));
                ui.add(TextEdit::singleline(&mut editor.draft.title).hint_text("例如：整理周报"));
                ui.add_space(8.0);

                ui.label(RichText::new("备注").size(12.0).color(theme.muted));
                ui.add(
                    TextEdit::multiline(&mut editor.draft.note)
                        .desired_rows(4)
                        .hint_text("补充一点上下文，也可以留空"),
                );
                ui.add_space(8.0);

                ui.label(RichText::new("截止日期").size(12.0).color(theme.muted));
                ui.add(TextEdit::singleline(&mut editor.draft.due_date).hint_text("YYYY-MM-DD"));

                if let Some(error) = &editor.error {
                    ui.add_space(8.0);
                    ui.label(RichText::new(error).size(12.0).color(theme.danger));
                }

                ui.add_space(14.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if pill_button(ui, IconKind::Check, "保存", ButtonTone::Primary, false)
                        .clicked()
                    {
                        save_clicked = true;
                    }
                    if pill_button(ui, IconKind::Close, "取消", ButtonTone::Quiet, false).clicked()
                    {
                        cancel_clicked = true;
                    }
                });
            });

        if save_clicked {
            self.save_editor();
        } else if cancel_clicked || !open {
            self.editor = None;
        }
    }

    fn draw_delete_confirm(&mut self, ctx: &Context) {
        let Some(task_id) = self.delete_candidate.clone() else {
            return;
        };

        let mut open = true;
        let mut confirm = false;
        let mut cancel = false;

        egui::Window::new("删除任务")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .default_width(300.0)
            .frame(dialog_frame(ctx))
            .open(&mut open)
            .show(ctx, |ui| {
                let theme = theme(ui);
                ui.label(
                    RichText::new("确定删除这条任务吗？")
                        .size(13.0)
                        .color(theme.text),
                );
                ui.add_space(12.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if pill_button(ui, IconKind::Trash, "删除", ButtonTone::Danger, false).clicked()
                    {
                        confirm = true;
                    }
                    if pill_button(ui, IconKind::Close, "取消", ButtonTone::Quiet, false).clicked()
                    {
                        cancel = true;
                    }
                });
            });

        if confirm {
            self.tasks.retain(|task| task.id != task_id);
            self.delete_candidate = None;
            self.persist();
        } else if cancel || !open {
            self.delete_candidate = None;
        }
    }
}

fn draw_task_row(ui: &mut egui::Ui, task: &Task) -> Option<PendingAction> {
    let mut pending_action = None;
    let theme = theme(ui);
    let row_height = 62.0;
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, row_height), Sense::hover());

    if response.hovered() {
        ui.painter()
            .rect_filled(rect.shrink(1.0), CornerRadius::same(8), theme.row_hover);
    }

    let check_rect = Rect::from_center_size(
        Pos2::new(rect.left() + 22.0, rect.center().y),
        Vec2::splat(20.0),
    );
    if checkbox_at(ui, check_rect, &task.id, task.done).clicked() {
        pending_action = Some(PendingAction::Toggle(task.id.clone(), !task.done));
    }

    let delete_rect = Rect::from_center_size(
        Pos2::new(rect.right() - 22.0, rect.center().y),
        Vec2::splat(28.0),
    );
    if icon_button_at(
        ui,
        delete_rect,
        (&task.id, "delete"),
        IconKind::Trash,
        "删除",
        ButtonTone::Danger,
        false,
    )
    .clicked()
    {
        pending_action = Some(PendingAction::Delete(task.id.clone()));
    }

    let edit_rect = Rect::from_center_size(
        Pos2::new(rect.right() - 54.0, rect.center().y),
        Vec2::splat(28.0),
    );
    if icon_button_at(
        ui,
        edit_rect,
        (&task.id, "edit"),
        IconKind::Edit,
        "编辑",
        ButtonTone::Quiet,
        false,
    )
    .clicked()
    {
        pending_action = Some(PendingAction::Edit(task.id.clone()));
    }

    let text_rect = Rect::from_min_max(
        Pos2::new(rect.left() + 42.0, rect.top() + 9.0),
        Pos2::new(rect.right() - 76.0, rect.bottom() - 8.0),
    );
    ui.scope_builder(
        UiBuilder::new()
            .max_rect(text_rect)
            .layout(Layout::top_down(Align::Min)),
        |ui| {
            ui.set_clip_rect(text_rect);
            ui.set_width(text_rect.width());

            let title_color = if task.done { theme.muted } else { theme.text };
            let mut title = RichText::new(&task.title)
                .size(13.5)
                .strong()
                .color(title_color);
            if task.done {
                title = title.strikethrough();
            }
            ui.add(Label::new(title).truncate());

            ui.add_space(3.0);
            let meta = task_meta(task);
            let meta_text = if meta.is_empty() {
                RichText::new("无备注").size(12.0).color(theme.subtle)
            } else {
                RichText::new(meta).size(12.0).color(theme.muted)
            };
            ui.add(Label::new(meta_text).truncate());
        },
    );

    pending_action
}

fn note_summary(note: &str) -> String {
    let trimmed = note.trim();
    let mut summary = trimmed.chars().take(36).collect::<String>();
    if trimmed.chars().count() > 36 {
        summary.push('…');
    }
    summary
}

fn task_meta(task: &Task) -> String {
    let note = note_summary(&task.note);
    match (&task.due_date, note.is_empty()) {
        (Some(due_date), false) => format!("截止 {due_date} · {note}"),
        (Some(due_date), true) => format!("截止 {due_date}"),
        (None, false) => note,
        (None, true) => String::new(),
    }
}

#[derive(Clone, Copy)]
struct AppTheme {
    background: Color32,
    header: Color32,
    panel: Color32,
    row_hover: Color32,
    border: Color32,
    divider: Color32,
    text: Color32,
    muted: Color32,
    subtle: Color32,
    accent: Color32,
    accent_hover: Color32,
    accent_soft: Color32,
    danger: Color32,
    danger_soft: Color32,
    danger_border: Color32,
}

fn theme_for_ctx(ctx: &Context) -> AppTheme {
    if ctx.style().visuals.dark_mode {
        dark_theme()
    } else {
        light_theme()
    }
}

fn theme(ui: &Ui) -> AppTheme {
    if ui.visuals().dark_mode {
        dark_theme()
    } else {
        light_theme()
    }
}

fn light_theme() -> AppTheme {
    AppTheme {
        background: Color32::from_rgb(243, 245, 247),
        header: Color32::from_rgb(255, 255, 255),
        panel: Color32::from_rgb(255, 255, 255),
        row_hover: Color32::from_rgb(247, 250, 252),
        border: Color32::from_rgb(216, 224, 229),
        divider: Color32::from_rgb(235, 240, 243),
        text: Color32::from_rgb(17, 22, 26),
        muted: Color32::from_rgb(101, 112, 122),
        subtle: Color32::from_rgb(155, 166, 174),
        accent: Color32::from_rgb(14, 165, 233),
        accent_hover: Color32::from_rgb(2, 132, 199),
        accent_soft: Color32::from_rgb(230, 246, 253),
        danger: Color32::from_rgb(220, 38, 38),
        danger_soft: Color32::from_rgb(255, 241, 242),
        danger_border: Color32::from_rgb(248, 204, 209),
    }
}

fn dark_theme() -> AppTheme {
    AppTheme {
        background: Color32::from_rgb(17, 20, 21),
        header: Color32::from_rgb(23, 27, 28),
        panel: Color32::from_rgb(23, 27, 28),
        row_hover: Color32::from_rgb(31, 37, 38),
        border: Color32::from_rgb(49, 55, 56),
        divider: Color32::from_rgb(39, 45, 46),
        text: Color32::from_rgb(242, 245, 246),
        muted: Color32::from_rgb(154, 164, 170),
        subtle: Color32::from_rgb(104, 114, 121),
        accent: Color32::from_rgb(56, 189, 248),
        accent_hover: Color32::from_rgb(14, 165, 233),
        accent_soft: Color32::from_rgb(19, 50, 61),
        danger: Color32::from_rgb(248, 113, 113),
        danger_soft: Color32::from_rgb(45, 25, 28),
        danger_border: Color32::from_rgb(91, 49, 55),
    }
}

#[derive(Clone, Copy)]
enum ButtonTone {
    Primary,
    Quiet,
    Danger,
}

#[derive(Clone, Copy)]
enum IconKind {
    Add,
    Pin,
    Edit,
    Trash,
    Check,
    Close,
}

fn pill_button(
    ui: &mut Ui,
    icon: IconKind,
    label: &str,
    tone: ButtonTone,
    selected: bool,
) -> Response {
    let theme = theme(ui);
    let text_color = button_text_color(theme, tone, selected);
    let galley =
        ui.painter()
            .layout_no_wrap(label.to_string(), FontId::proportional(12.0), text_color);
    let width = (galley.size().x + 34.0).max(64.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 30.0), Sense::click());
    let (fill, stroke, icon_color) = button_paint(theme, tone, selected, response.hovered());

    ui.painter().rect_filled(rect, CornerRadius::same(7), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(7),
        Stroke::new(1.0, stroke),
        StrokeKind::Inside,
    );

    let icon_center = Pos2::new(rect.left() + 16.0, rect.center().y);
    draw_icon(ui, icon, icon_center, icon_color, 13.0);
    ui.painter().galley(
        Pos2::new(rect.left() + 27.0, rect.center().y - galley.size().y / 2.0),
        galley,
        text_color,
    );

    response.on_hover_text(label)
}

fn icon_button_at(
    ui: &mut Ui,
    rect: Rect,
    id_salt: impl Hash,
    icon: IconKind,
    tooltip: &str,
    tone: ButtonTone,
    selected: bool,
) -> Response {
    let response = ui.interact(rect, ui.make_persistent_id(id_salt), Sense::click());
    let theme = theme(ui);
    let (fill, stroke, icon_color) = button_paint(theme, tone, selected, response.hovered());

    ui.painter().rect_filled(rect, CornerRadius::same(7), fill);
    if stroke != Color32::TRANSPARENT {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(7),
            Stroke::new(1.0, stroke),
            StrokeKind::Inside,
        );
    }
    draw_icon(ui, icon, rect.center(), icon_color, 13.0);

    response.on_hover_text(tooltip)
}

fn checkbox_at(ui: &mut Ui, rect: Rect, id_salt: impl Hash, checked: bool) -> Response {
    let response = ui.interact(
        rect,
        ui.make_persistent_id((id_salt, "check")),
        Sense::click(),
    );
    let theme = theme(ui);
    let fill = if checked {
        theme.accent
    } else if response.hovered() {
        theme.row_hover
    } else {
        theme.panel
    };
    let stroke = if checked { theme.accent } else { theme.border };

    ui.painter()
        .rect_filled(rect.shrink(1.0), CornerRadius::same(5), fill);
    ui.painter().rect_stroke(
        rect.shrink(1.0),
        CornerRadius::same(5),
        Stroke::new(1.0, stroke),
        StrokeKind::Inside,
    );

    if checked {
        draw_icon(ui, IconKind::Check, rect.center(), Color32::WHITE, 13.0);
    }

    response.on_hover_text("切换完成状态")
}

fn button_text_color(theme: AppTheme, tone: ButtonTone, selected: bool) -> Color32 {
    match tone {
        ButtonTone::Primary => Color32::WHITE,
        ButtonTone::Quiet if selected => theme.accent,
        ButtonTone::Quiet => theme.text,
        ButtonTone::Danger => theme.danger,
    }
}

fn button_paint(
    theme: AppTheme,
    tone: ButtonTone,
    selected: bool,
    hovered: bool,
) -> (Color32, Color32, Color32) {
    match tone {
        ButtonTone::Primary => (
            if hovered {
                theme.accent_hover
            } else {
                theme.accent
            },
            Color32::TRANSPARENT,
            Color32::WHITE,
        ),
        ButtonTone::Quiet => (
            if selected || hovered {
                theme.accent_soft
            } else {
                Color32::TRANSPARENT
            },
            if selected { theme.accent } else { theme.border },
            if selected { theme.accent } else { theme.muted },
        ),
        ButtonTone::Danger => (
            if hovered {
                theme.danger_soft
            } else {
                Color32::TRANSPARENT
            },
            if hovered {
                theme.danger_border
            } else {
                Color32::TRANSPARENT
            },
            theme.danger,
        ),
    }
}

fn draw_icon(ui: &Ui, icon: IconKind, center: Pos2, color: Color32, size: f32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.6, color);
    let h = size / 2.0;

    match icon {
        IconKind::Add => {
            painter.line_segment(
                [
                    center + Vec2::new(-h * 0.65, 0.0),
                    center + Vec2::new(h * 0.65, 0.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    center + Vec2::new(0.0, -h * 0.65),
                    center + Vec2::new(0.0, h * 0.65),
                ],
                stroke,
            );
        }
        IconKind::Pin => {
            painter.line_segment(
                [center + Vec2::new(-3.0, -5.0), center + Vec2::new(4.0, 2.0)],
                stroke,
            );
            painter.line_segment(
                [center + Vec2::new(-5.0, 0.0), center + Vec2::new(0.0, -5.0)],
                stroke,
            );
            painter.line_segment(
                [center + Vec2::new(1.5, 1.5), center + Vec2::new(-3.5, 6.0)],
                stroke,
            );
        }
        IconKind::Edit => {
            painter.line_segment(
                [center + Vec2::new(-5.0, 4.0), center + Vec2::new(3.5, -4.5)],
                stroke,
            );
            painter.line_segment(
                [center + Vec2::new(1.0, -5.5), center + Vec2::new(5.0, -1.5)],
                stroke,
            );
            painter.line_segment(
                [center + Vec2::new(-5.5, 5.0), center + Vec2::new(-2.0, 4.0)],
                stroke,
            );
        }
        IconKind::Trash => {
            painter.line_segment(
                [
                    center + Vec2::new(-5.0, -4.0),
                    center + Vec2::new(5.0, -4.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    center + Vec2::new(-3.0, -1.0),
                    center + Vec2::new(-2.0, 5.0),
                ],
                stroke,
            );
            painter.line_segment(
                [center + Vec2::new(3.0, -1.0), center + Vec2::new(2.0, 5.0)],
                stroke,
            );
            painter.line_segment(
                [center + Vec2::new(-2.0, 5.0), center + Vec2::new(2.0, 5.0)],
                stroke,
            );
            painter.line_segment(
                [
                    center + Vec2::new(-2.0, -6.0),
                    center + Vec2::new(2.0, -6.0),
                ],
                stroke,
            );
        }
        IconKind::Check => {
            painter.line_segment(
                [center + Vec2::new(-5.0, 0.0), center + Vec2::new(-1.5, 4.0)],
                stroke,
            );
            painter.line_segment(
                [center + Vec2::new(-1.5, 4.0), center + Vec2::new(5.5, -4.0)],
                stroke,
            );
        }
        IconKind::Close => {
            painter.line_segment(
                [center + Vec2::new(-4.5, -4.5), center + Vec2::new(4.5, 4.5)],
                stroke,
            );
            painter.line_segment(
                [center + Vec2::new(4.5, -4.5), center + Vec2::new(-4.5, 4.5)],
                stroke,
            );
        }
    }
}

fn draw_empty_mark(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(48.0), Sense::hover());
    let theme = theme(ui);
    let painter = ui.painter();

    painter.rect_filled(rect, CornerRadius::same(12), theme.panel);
    painter.rect_stroke(
        rect,
        CornerRadius::same(12),
        Stroke::new(1.0, theme.border),
        StrokeKind::Inside,
    );
    let left = rect.left() + 13.0;
    for index in 0..3 {
        let y = rect.top() + 15.0 + index as f32 * 8.0;
        painter.circle_filled(Pos2::new(left, y), 1.5, theme.accent);
        painter.line_segment(
            [Pos2::new(left + 6.0, y), Pos2::new(rect.right() - 12.0, y)],
            Stroke::new(1.4, theme.muted),
        );
    }
}

fn dialog_frame(ctx: &Context) -> Frame {
    let theme = theme_for_ctx(ctx);
    Frame::window(&ctx.style())
        .fill(theme.panel)
        .stroke(Stroke::new(1.0, theme.border))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(14, 12))
}

fn draw_small_logo(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(28.0), egui::Sense::hover());
    let theme = theme(ui);
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(8), theme.accent);
    painter.line_segment(
        [
            rect.left_top() + Vec2::new(8.0, 11.0),
            rect.left_top() + Vec2::new(13.0, 16.0),
        ],
        Stroke::new(2.4, Color32::WHITE),
    );
    painter.line_segment(
        [
            rect.left_top() + Vec2::new(13.0, 16.0),
            rect.left_top() + Vec2::new(21.0, 9.0),
        ],
        Stroke::new(2.4, Color32::WHITE),
    );
    painter.line_segment(
        [
            rect.left_top() + Vec2::new(8.0, 22.0),
            rect.left_top() + Vec2::new(21.0, 22.0),
        ],
        Stroke::new(2.0, Color32::from_white_alpha(190)),
    );
}

fn configure_style(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(8.0, 7.0);
    style.spacing.button_padding = Vec2::new(10.0, 5.0);
    style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);
    style.visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
    style.visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
    style.visuals.widgets.active.corner_radius = CornerRadius::same(8);
    ctx.set_style(style);
}

fn install_system_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    if let Some(bytes) = load_cjk_font() {
        fonts
            .font_data
            .insert("system_cjk".to_string(), FontData::from_owned(bytes).into());
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "system_cjk".to_string());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "system_cjk".to_string());
    }
    ctx.set_fonts(fonts);
}

fn load_cjk_font() -> Option<Vec<u8>> {
    let candidates = [
        "C:/Windows/Fonts/msyh.ttc",
        "C:/Windows/Fonts/simhei.ttf",
        "/System/Library/Fonts/PingFang.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.otf",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/arphic/uming.ttc",
    ];

    candidates.iter().find_map(|path| std::fs::read(path).ok())
}

fn load_window_icon() -> egui::IconData {
    let image = image::load_from_memory(APP_ICON_BYTES)
        .expect("embedded Record icon should decode")
        .into_rgba8();
    let (width, height) = image.dimensions();

    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}

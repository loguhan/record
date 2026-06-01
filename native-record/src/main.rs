#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{
    egui::{
        self, Align, Align2, CentralPanel, Color32, Context, CornerRadius, FontData,
        FontDefinitions, FontFamily, Frame, Layout, Margin, RichText, Stroke, TextEdit,
        TopBottomPanel, Vec2, ViewportCommand, WindowLevel,
    },
    NativeOptions,
};
use record_native::store::{
    create_task, default_store_path, load_or_create_store, save_tasks_at_path, update_task, Task,
    TaskDraft,
};
use std::{path::PathBuf, sync::Arc};

const APP_ICON_BYTES: &[u8] = include_bytes!("../../src-tauri/icons/icon.png");

fn main() -> eframe::Result {
    let viewport = egui::ViewportBuilder::default()
        .with_title("Record")
        .with_inner_size([430.0, 620.0])
        .with_min_inner_size([360.0, 500.0])
        .with_icon(Arc::new(load_window_icon()));

    eframe::run_native(
        "Record",
        NativeOptions {
            viewport,
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
        TopBottomPanel::top("record-header").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                draw_small_logo(ui);
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("Record").strong().size(15.0));
                    let (open, completed) = self.task_counts();
                    ui.label(
                        RichText::new(format!("{open} 个待办 · {completed} 个已完成"))
                            .size(12.0)
                            .color(muted_text(ui)),
                    );
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(8.0);
                    if ui.button("+ 新建").clicked() {
                        self.open_create_editor();
                    }
                    let pin_label = if self.always_on_top {
                        "取消置顶"
                    } else {
                        "置顶"
                    };
                    if ui.button(pin_label).clicked() {
                        self.toggle_always_on_top(ctx);
                    }
                });
            });
            ui.add_space(6.0);
        });

        CentralPanel::default().show(ctx, |ui| {
            if let Some(error) = &self.error {
                Frame::NONE
                    .fill(Color32::from_rgb(68, 28, 32))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.label(RichText::new(error).color(Color32::from_rgb(255, 210, 216)));
                    });
                ui.add_space(8.0);
            }

            if self.tasks.is_empty() {
                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    ui.add_space(150.0);
                    ui.label(RichText::new("还没有任务").strong().size(16.0));
                    ui.label(
                        RichText::new("新建一条任务，标题、备注和日期就能开始用了。")
                            .color(muted_text(ui)),
                    );
                    ui.add_space(10.0);
                    if ui.button("+ 新建任务").clicked() {
                        self.open_create_editor();
                    }
                });
            } else {
                let mut pending_action = None;
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for task in &self.tasks {
                            if let Some(action) = draw_task_row(ui, task) {
                                pending_action = Some(action);
                            }
                            ui.add_space(6.0);
                        }
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
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(RichText::new("标题").color(muted_text(ui)));
                ui.add(TextEdit::singleline(&mut editor.draft.title).hint_text("例如：整理周报"));
                ui.add_space(8.0);

                ui.label(RichText::new("备注").color(muted_text(ui)));
                ui.add(
                    TextEdit::multiline(&mut editor.draft.note)
                        .desired_rows(4)
                        .hint_text("补充一点上下文，也可以留空"),
                );
                ui.add_space(8.0);

                ui.label(RichText::new("截止日期").color(muted_text(ui)));
                ui.add(TextEdit::singleline(&mut editor.draft.due_date).hint_text("YYYY-MM-DD"));

                if let Some(error) = &editor.error {
                    ui.add_space(8.0);
                    ui.label(RichText::new(error).color(Color32::from_rgb(220, 72, 84)));
                }

                ui.add_space(14.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("保存").clicked() {
                        save_clicked = true;
                    }
                    if ui.button("取消").clicked() {
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
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label("确定删除这条任务吗？");
                ui.add_space(12.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("删除").clicked() {
                        confirm = true;
                    }
                    if ui.button("取消").clicked() {
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
    let fill = if task.done {
        ui.visuals().faint_bg_color
    } else {
        ui.visuals().panel_fill
    };

    Frame::NONE
        .fill(fill)
        .stroke(Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(10, 9))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let mut done = task.done;
                if ui.checkbox(&mut done, "").changed() {
                    pending_action = Some(PendingAction::Toggle(task.id.clone(), done));
                }

                ui.vertical(|ui| {
                    let mut title = RichText::new(&task.title).strong();
                    if task.done {
                        title = title.strikethrough().color(muted_text(ui));
                    }
                    ui.label(title);

                    if !task.note.trim().is_empty() {
                        ui.label(
                            RichText::new(note_summary(&task.note))
                                .size(12.0)
                                .color(muted_text(ui)),
                        );
                    }

                    if let Some(due_date) = &task.due_date {
                        ui.label(
                            RichText::new(format!("截止 {due_date}"))
                                .size(12.0)
                                .color(Color32::from_rgb(70, 119, 222)),
                        );
                    }
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.small_button("删除").clicked() {
                        pending_action = Some(PendingAction::Delete(task.id.clone()));
                    }
                    if ui.small_button("编辑").clicked() {
                        pending_action = Some(PendingAction::Edit(task.id.clone()));
                    }
                });
            });
        });

    pending_action
}

fn note_summary(note: &str) -> String {
    let trimmed = note.trim();
    let mut summary = trimmed.chars().take(42).collect::<String>();
    if trimmed.chars().count() > 42 {
        summary.push('…');
    }
    summary
}

fn draw_small_logo(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(34.0), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(8), Color32::from_rgb(32, 98, 196));
    painter.line_segment(
        [
            rect.left_top() + Vec2::new(10.0, 12.0),
            rect.left_top() + Vec2::new(15.0, 18.0),
        ],
        Stroke::new(2.4, Color32::WHITE),
    );
    painter.line_segment(
        [
            rect.left_top() + Vec2::new(15.0, 18.0),
            rect.left_top() + Vec2::new(25.0, 10.0),
        ],
        Stroke::new(2.4, Color32::WHITE),
    );
    painter.line_segment(
        [
            rect.left_top() + Vec2::new(10.0, 24.0),
            rect.left_top() + Vec2::new(25.0, 24.0),
        ],
        Stroke::new(2.2, Color32::from_rgb(210, 230, 255)),
    );
}

fn muted_text(ui: &egui::Ui) -> Color32 {
    ui.visuals().weak_text_color()
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

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use dpi::PhysicalSize;
use egui::text::{CCursor, CCursorRange};
use egui::text_edit::TextEditState;
use egui::{
    Key, Label, LayerId, Modifiers, PaintCallback, TopBottomPanel, Vec2, WidgetInfo, WidgetType,
    pos2,
};
use egui_glow::{CallbackFn, EguiGlow};
use egui_winit::EventResponse;
use euclid::{Box2D, Length, Point2D, Rect, Scale, Size2D};
use log::warn;
use servo::{
    DeviceIndependentPixel, DevicePixel, Image, LoadStatus, OffscreenRenderingContext, PixelFormat,
    PrefValue, RenderingContext, ServoUrl, WebView, WebViewId,
};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::Window;

use super::geometry::winit_position_to_euclid_point;
use crate::desktop::event_loop::AppEvent;
use crate::desktop::headed_window;
use crate::prefs::{EXPERIMENTAL_PREFS, ServoShellPreferences};
use crate::running_app_state::RunningAppState;
use crate::window::ServoShellWindow;

/// The user interface of a headed servoshell. Currently this is implemented via
/// egui.
pub struct Gui {
    rendering_context: Rc<OffscreenRenderingContext>,
    context: EguiGlow,
    event_queue: Vec<GuiCommand>,
    toolbar_height: Length<f32, DeviceIndependentPixel>,

    last_mouse_position: Option<Point2D<f32, DeviceIndependentPixel>>,
    location: String,

    /// Whether the location has been edited by the user without clicking Go.
    location_dirty: bool,

    /// The [`LoadStatus`] of the active `WebView`.
    load_status: LoadStatus,

    /// The text to display in the status bar on the bottom of the window.
    status_text: Option<String>,

    /// Whether or not the current `WebView` can navigate backward.
    can_go_back: bool,

    /// Whether or not the current `WebView` can navigate forward.
    can_go_forward: bool,

    /// Handle to the GPU texture of the favicon.
    ///
    /// These need to be cached across egui draw calls.
    favicon_textures: HashMap<WebViewId, (egui::TextureHandle, egui::load::SizedTexture)>,

    /// Whether the user has enabled experimental preferences.
    experimental_prefs_enabled: bool,
}

/// A command received via the user interacting with the user interface.
pub enum GuiCommand {
    /// Go button clicked.
    Go(String),
    Back,
    Forward,
    Reload,
    ReloadAll,
    NewWebView,
    CloseWebView(WebViewId),
}

fn truncate_with_ellipsis(input: &str, max_length: usize) -> String {
    if input.chars().count() > max_length {
        let truncated: String = input.chars().take(max_length.saturating_sub(1)).collect();
        format!("{}…", truncated)
    } else {
        input.to_string()
    }
}

impl Drop for Gui {
    fn drop(&mut self) {
        self.context.destroy();
    }
}

impl Gui {
    pub(crate) fn new(
        winit_window: &Window,
        event_loop: &ActiveEventLoop,
        event_loop_proxy: EventLoopProxy<AppEvent>,
        rendering_context: Rc<OffscreenRenderingContext>,
        initial_url: ServoUrl,
        preferences: &ServoShellPreferences,
    ) -> Self {
        #[allow(clippy::arc_with_non_send_sync)]
        let mut context = EguiGlow::new(
            event_loop,
            rendering_context.glow_gl_api(),
            None,
            None,
            false,
        );

        context
            .egui_winit
            .init_accesskit(event_loop, winit_window, event_loop_proxy);
        winit_window.set_visible(true);

        context.egui_ctx.options_mut(|options| {
            // Disable the builtin egui handlers for the Ctrl+Plus, Ctrl+Minus and Ctrl+0
            // shortcuts as they don't work well with servoshell's `device-pixel-ratio` CLI argument.
            options.zoom_with_keyboard = false;

            // On platforms where winit fails to obtain a system theme, fall back to a light theme
            // since it is the more common default.
            options.fallback_theme = egui::Theme::Light;
        });

        Self {
            rendering_context,
            context,
            event_queue: vec![],
            toolbar_height: Default::default(),
            last_mouse_position: None,
            location: initial_url.to_string(),
            location_dirty: false,
            load_status: LoadStatus::Complete,
            status_text: None,
            can_go_back: false,
            can_go_forward: false,
            favicon_textures: Default::default(),
            experimental_prefs_enabled: preferences.experimental_prefs_enabled,
        }
    }

    pub(crate) fn take_commands(&mut self) -> Vec<GuiCommand> {
        std::mem::take(&mut self.event_queue)
    }

    pub(crate) fn on_window_event(
        &mut self,
        winit_window: &Window,
        event: &WindowEvent,
    ) -> EventResponse {
        let mut result = self.context.on_window_event(winit_window, event);
        result.consumed &= match event {
            WindowEvent::CursorMoved { position, .. } => {
                let scale = Scale::<_, DeviceIndependentPixel, _>::new(
                    self.context.egui_ctx.pixels_per_point(),
                );
                self.last_mouse_position =
                    Some(winit_position_to_euclid_point(*position).to_f32() / scale);
                self.last_mouse_position
                    .is_some_and(|p| self.is_in_egui_toolbar_rect(p))
            },
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Forward,
                ..
            } => {
                self.event_queue.push(GuiCommand::Forward);
                true
            },
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Back,
                ..
            } => {
                self.event_queue.push(GuiCommand::Back);
                true
            },
            WindowEvent::MouseWheel { .. } | WindowEvent::MouseInput { .. } => self
                .last_mouse_position
                .is_some_and(|p| self.is_in_egui_toolbar_rect(p)),
            _ => true,
        };
        result
    }

    /// The height of the top toolbar of this user inteface ie the distance from the top of the
    /// window to the position of the `WebView`.
    pub(crate) fn toolbar_height(&self) -> Length<f32, DeviceIndependentPixel> {
        self.toolbar_height
    }

    /// Return true iff the given position is over the egui toolbar.
    fn is_in_egui_toolbar_rect(&self, position: Point2D<f32, DeviceIndependentPixel>) -> bool {
        position.y < self.toolbar_height.get()
    }

    /// Create a frameless button with square sizing, as used in the toolbar.
    #[allow(dead_code)]
    fn toolbar_button(text: &str) -> egui::Button<'_> {
        egui::Button::new(text)
            .frame(false)
            .min_size(Vec2 { x: 28.0, y: 28.0 })
    }

    /// Create a styled navigation button (back, forward, reload, stop)
    #[allow(dead_code)]
    fn nav_button(text: &str) -> egui::Button<'_> {
        egui::Button::new(egui::RichText::new(text).size(16.0))
            .frame(false)
            .min_size(Vec2 { x: 32.0, y: 32.0 })
            .corner_radius(6.0)
    }

    /// Calculate the width for each tab based on available space
    fn calculate_tab_width(
        available_width: f32,
        tab_count: usize,
        new_tab_button_width: f32,
    ) -> f32 {
        const MIN_TAB_WIDTH: f32 = 60.0; // Minimum width to show at least icon + X
        const MAX_TAB_WIDTH: f32 = 200.0; // Maximum comfortable tab width
        const TAB_MARGIN: f32 = 4.0; // Margin between tabs

        if tab_count == 0 {
            return MAX_TAB_WIDTH;
        }

        let usable_width = available_width - new_tab_button_width - 16.0; // 16px padding
        let width_per_tab = (usable_width / tab_count as f32) - TAB_MARGIN;

        width_per_tab.clamp(MIN_TAB_WIDTH, MAX_TAB_WIDTH)
    }

    /// Draws a browser tab, checking for clicks and queues appropriate [`GuiCommand`]s.
    /// Using a custom widget here would've been nice, but it doesn't seem as though egui
    /// supports that, so we arrange multiple Widgets in a way that they look connected.
    fn browser_tab(
        ui: &mut egui::Ui,
        window: &ServoShellWindow,
        webview: WebView,
        event_queue: &mut Vec<GuiCommand>,
        favicon_texture: Option<egui::load::SizedTexture>,
        tab_width: f32,
    ) {
        let label = match (webview.page_title(), webview.url()) {
            (Some(title), _) if !title.is_empty() => title,
            (_, Some(url)) => url.to_string(),
            _ => "New Tab".into(),
        };

        let active = window.active_webview().map(|webview| webview.id()) == Some(webview.id());

        // Modern color scheme
        let visuals = ui.visuals();
        let inactive_bg_color = if visuals.dark_mode {
            egui::Color32::from_rgb(45, 45, 48)
        } else {
            egui::Color32::from_rgb(235, 235, 238)
        };
        let active_bg_color = if visuals.dark_mode {
            egui::Color32::from_rgb(60, 60, 64)
        } else {
            egui::Color32::from_rgb(255, 255, 255)
        };
        let hover_bg_color = if visuals.dark_mode {
            egui::Color32::from_rgb(55, 55, 58)
        } else {
            egui::Color32::from_rgb(245, 245, 248)
        };
        let close_hover_color = egui::Color32::from_rgb(200, 60, 60);

        // Calculate label truncation based on tab width
        // Reserve space for: padding (10) + favicon (16) + spacing (6) + close button (20) + padding (6)
        let available_text_width = tab_width - 58.0;
        let char_width = 7.0; // Approximate width per character
        let max_chars = ((available_text_width / char_width).max(3.0)) as usize;

        // Setup a tab frame with fixed width
        let mut tab_frame = egui::Frame::NONE
            .corner_radius(egui::CornerRadius {
                nw: 8,
                ne: 8,
                sw: 0,
                se: 0,
            })
            .inner_margin(egui::Margin::symmetric(5, 4))
            .begin(ui);
        {
            // Make the content use horizontal layout
            tab_frame.content_ui.set_min_width(tab_width - 10.0);
            tab_frame.content_ui.set_max_width(tab_width - 10.0);

            let visuals = tab_frame.content_ui.visuals_mut();
            // Remove strokes
            visuals.widgets.active.bg_stroke.width = 0.0;
            visuals.widgets.hovered.bg_stroke.width = 0.0;
            visuals.widgets.noninteractive.weak_bg_fill = egui::Color32::TRANSPARENT;
            visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
            visuals.widgets.hovered.weak_bg_fill = egui::Color32::TRANSPARENT;
            visuals.widgets.active.weak_bg_fill = egui::Color32::TRANSPARENT;
            visuals.widgets.active.expansion = 0.0;
            visuals.widgets.hovered.expansion = 0.0;

            tab_frame.content_ui.horizontal(|ui| {
                // Favicon
                if let Some(favicon) = favicon_texture {
                    ui.add(
                        egui::Image::from_texture(favicon)
                            .fit_to_exact_size(egui::vec2(16.0, 16.0))
                            .bg_fill(egui::Color32::TRANSPARENT),
                    );
                    ui.add_space(4.0);
                } else {
                    // Default page icon
                    ui.label(egui::RichText::new("[#]").size(10.0));
                    ui.add_space(4.0);
                }

                // Tab title - use remaining space except for close button
                let remaining_width = ui.available_width() - 24.0;
                ui.allocate_ui_with_layout(
                    egui::vec2(remaining_width, ui.available_height()),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.set_clip_rect(ui.max_rect());
                        let tab = ui.add(
                            egui::Label::new(
                                egui::RichText::new(truncate_with_ellipsis(&label, max_chars))
                                    .size(12.0),
                            )
                            .selectable(false)
                            .sense(egui::Sense::click()),
                        );

                        if !active && tab.clicked() {
                            window.activate_webview(webview.id());
                        }
                        if tab.middle_clicked() {
                            event_queue.push(GuiCommand::CloseWebView(webview.id()));
                        }
                        tab.on_hover_ui(|ui| {
                            ui.label(&label);
                        });
                    },
                );

                // Close button
                let close_response = ui.add(
                    egui::Button::new(egui::RichText::new("x").size(14.0))
                        .frame(false)
                        .min_size(egui::vec2(18.0, 18.0)),
                );
                if close_response.hovered() {
                    ui.painter().rect_filled(
                        close_response.rect,
                        4.0,
                        close_hover_color.gamma_multiply(0.3),
                    );
                }
                close_response.widget_info(|| {
                    let mut info = WidgetInfo::new(WidgetType::Button);
                    info.label = Some("Close".into());
                    info
                });
                if close_response.clicked() || close_response.middle_clicked() {
                    event_queue.push(GuiCommand::CloseWebView(webview.id()));
                }
            });
        }

        let response = tab_frame.allocate_space(ui);
        let fill_color = if active {
            active_bg_color
        } else if response.hovered() {
            hover_bg_color
        } else {
            inactive_bg_color
        };
        tab_frame.frame.fill = fill_color;

        // Add bottom border for active tab (like Chrome)
        if active {
            let rect = response.rect;
            let accent_color = if ui.visuals().dark_mode {
                egui::Color32::from_rgb(100, 150, 255)
            } else {
                egui::Color32::from_rgb(26, 115, 232)
            };
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(rect.left(), rect.bottom() - 2.0),
                    egui::vec2(rect.width(), 2.0),
                ),
                0.0,
                accent_color,
            );
        }

        tab_frame.end(ui);
    }

    /// Update the user interface, but do not paint the updated state.
    pub(crate) fn update(
        &mut self,
        state: &RunningAppState,
        window: &ServoShellWindow,
        headed_window: &headed_window::Window,
    ) {
        let Self {
            rendering_context,
            context,
            event_queue,
            toolbar_height,
            location,
            location_dirty,
            favicon_textures,
            ..
        } = self;

        let winit_window = headed_window.winit_window();
        context.run(winit_window, |ctx| {
            load_pending_favicons(ctx, window, favicon_textures);

            // TODO: While in fullscreen add some way to mitigate the increased phishing risk
            // when not displaying the URL bar: https://github.com/servo/servo/issues/32443
            if winit_window.fullscreen().is_none() {
                // Modern toolbar styling
                let toolbar_bg = if ctx.style().visuals.dark_mode {
                    egui::Color32::from_rgb(40, 40, 44)
                } else {
                    egui::Color32::from_rgb(248, 249, 250)
                };

                let frame = egui::Frame::NONE
                    .fill(toolbar_bg)
                    .inner_margin(egui::Margin::symmetric(8, 6));

                TopBottomPanel::top("toolbar").frame(frame).show(ctx, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;

                        // Navigation buttons group
                        let nav_button_size = egui::vec2(32.0, 32.0);

                        let back_button = ui.add_enabled(
                            self.can_go_back,
                            egui::Button::new(egui::RichText::new("<").size(14.0))
                                .frame(false)
                                .min_size(nav_button_size)
                                .corner_radius(6.0),
                        );
                        back_button.widget_info(|| {
                            let mut info = WidgetInfo::new(WidgetType::Button);
                            info.label = Some("Back".into());
                            info
                        });
                        if back_button.clicked() {
                            event_queue.push(GuiCommand::Back);
                        }

                        let forward_button = ui.add_enabled(
                            self.can_go_forward,
                            egui::Button::new(egui::RichText::new(">").size(14.0))
                                .frame(false)
                                .min_size(nav_button_size)
                                .corner_radius(6.0),
                        );
                        forward_button.widget_info(|| {
                            let mut info = WidgetInfo::new(WidgetType::Button);
                            info.label = Some("Forward".into());
                            info
                        });
                        if forward_button.clicked() {
                            event_queue.push(GuiCommand::Forward);
                        }

                        match self.load_status {
                            LoadStatus::Started | LoadStatus::HeadParsed => {
                                let stop_button = ui.add(
                                    egui::Button::new(egui::RichText::new("x").size(14.0))
                                        .frame(false)
                                        .min_size(nav_button_size)
                                        .corner_radius(6.0),
                                );
                                stop_button.widget_info(|| {
                                    let mut info = WidgetInfo::new(WidgetType::Button);
                                    info.label = Some("Stop".into());
                                    info
                                });
                                if stop_button.clicked() {
                                    warn!("Do not support stop yet.");
                                }
                            },
                            LoadStatus::Complete => {
                                let reload_button = ui.add(
                                    egui::Button::new(egui::RichText::new("R").size(14.0))
                                        .frame(false)
                                        .min_size(nav_button_size)
                                        .corner_radius(6.0),
                                );
                                reload_button.widget_info(|| {
                                    let mut info = WidgetInfo::new(WidgetType::Button);
                                    info.label = Some("Reload".into());
                                    info
                                });
                                if reload_button.clicked() {
                                    event_queue.push(GuiCommand::Reload);
                                }
                            },
                        }

                        ui.add_space(8.0);

                        // Modern address bar - takes remaining width
                        let address_bar_bg = if ctx.style().visuals.dark_mode {
                            egui::Color32::from_rgb(55, 55, 60)
                        } else {
                            egui::Color32::from_rgb(255, 255, 255)
                        };

                        // Reserve space for: prefs button (32) + spacing (12)
                        let available_width = (ui.available_width() - 48.0).max(100.0);

                        let location_id = egui::Id::new("location_input");

                        // Styled address bar frame
                        egui::Frame::NONE
                            .fill(address_bar_bg)
                            .corner_radius(20.0)
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(180)))
                            .inner_margin(egui::Margin::symmetric(12, 6))
                            .show(ui, |ui| {
                                ui.set_min_width(available_width);

                                // Lock/security icon
                                let url_text = location.as_str();
                                if url_text.starts_with("https://") {
                                    ui.label(
                                        egui::RichText::new("[S]")
                                            .size(10.0)
                                            .color(egui::Color32::GREEN),
                                    );
                                } else if url_text.starts_with("http://") {
                                    ui.label(
                                        egui::RichText::new("[!]")
                                            .size(10.0)
                                            .color(egui::Color32::from_rgb(200, 150, 0)),
                                    );
                                }

                                let location_field = ui.add(
                                    egui::TextEdit::singleline(location)
                                        .id(location_id)
                                        .frame(false)
                                        .desired_width(available_width - 30.0)
                                        .hint_text("Search or enter address")
                                        .font(egui::TextStyle::Body),
                                );

                                if location_field.changed() {
                                    *location_dirty = true;
                                }

                                // Handle address bar shortcut.
                                if ui.input(|i| {
                                    if cfg!(target_os = "macos") {
                                        i.clone().consume_key(Modifiers::COMMAND, Key::L)
                                    } else {
                                        i.clone().consume_key(Modifiers::COMMAND, Key::L)
                                            || i.clone().consume_key(Modifiers::ALT, Key::D)
                                    }
                                }) {
                                    location_field.request_focus();
                                }

                                // Select address bar text when it's focused
                                if location_field.gained_focus() {
                                    if let Some(mut state) =
                                        TextEditState::load(ui.ctx(), location_id)
                                    {
                                        state.cursor.set_char_range(Some(CCursorRange::two(
                                            CCursor::new(0),
                                            CCursor::new(location.len()),
                                        )));
                                        state.store(ui.ctx(), location_id);
                                    }
                                }

                                // Navigate when enter is pressed
                                if location_field.lost_focus()
                                    && ui.input(|i| i.clone().key_pressed(Key::Enter))
                                {
                                    event_queue.push(GuiCommand::Go(location.clone()));
                                }
                            });

                        ui.add_space(4.0);

                        ui.add_space(4.0);

                        // Settings button
                        let settings_btn = ui
                            .add(
                                egui::Button::new(egui::RichText::new("⚙").size(16.0))
                                    .frame(false)
                                    .min_size(egui::vec2(32.0, 32.0))
                                    .corner_radius(6.0),
                            )
                            .on_hover_text("Settings");

                        if settings_btn.clicked() {
                            event_queue.push(GuiCommand::Go("servo:preferences".to_string()));
                        }
                    });
                });

                // Modern Tab bar with adaptive scaling
                let tabs_bg_color = if ctx.style().visuals.dark_mode {
                    egui::Color32::from_rgb(35, 35, 38)
                } else {
                    egui::Color32::from_rgb(222, 225, 230)
                };

                let tabs_frame = egui::Frame::NONE
                    .fill(tabs_bg_color)
                    .inner_margin(egui::Margin {
                        left: 8,
                        right: 8,
                        top: 4,
                        bottom: 0,
                    });

                TopBottomPanel::top("tabs")
                    .frame(tabs_frame)
                    .show(ctx, |ui| {
                        let available_width = ui.available_width();
                        let webviews: Vec<_> = window.webviews().into_iter().collect();
                        let tab_count = webviews.len();
                        let new_tab_button_width = 36.0;
                        let tab_width = Self::calculate_tab_width(
                            available_width,
                            tab_count,
                            new_tab_button_width,
                        );

                        // Use horizontal scroll for many tabs
                        egui::ScrollArea::horizontal()
                            .scroll_bar_visibility(
                                egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                            )
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 2.0;

                                    for (id, webview) in webviews.into_iter() {
                                        let favicon = favicon_textures
                                            .get(&id)
                                            .map(|(_, favicon)| favicon)
                                            .copied();
                                        Self::browser_tab(
                                            ui,
                                            window,
                                            webview,
                                            event_queue,
                                            favicon,
                                            tab_width,
                                        );
                                    }

                                    // New tab button - styled to match tabs
                                    let new_tab_btn = ui.add(
                                        egui::Button::new(egui::RichText::new("+").size(18.0))
                                            .frame(false)
                                            .min_size(egui::vec2(32.0, 28.0))
                                            .corner_radius(6.0),
                                    );
                                    new_tab_btn.widget_info(|| {
                                        let mut info = WidgetInfo::new(WidgetType::Button);
                                        info.label = Some("New tab".into());
                                        info
                                    });
                                    if new_tab_btn.clicked() {
                                        event_queue.push(GuiCommand::NewWebView);
                                    }
                                });
                            });
                    });
            };

            // The toolbar height is where the Context’s available rect starts.
            // For reasons that are unclear, the TopBottomPanel’s ui cursor exceeds this by one egui
            // point, but the Context is correct and the TopBottomPanel is wrong.
            *toolbar_height = Length::new(ctx.available_rect().min.y);

            let scale =
                Scale::<_, DeviceIndependentPixel, DevicePixel>::new(ctx.pixels_per_point());

            headed_window.for_each_active_dialog(window, |dialog| dialog.update(ctx));

            // Paint background for webview area to prevent flash when loading new tabs
            let webview_bg = if ctx.style().visuals.dark_mode {
                egui::Color32::from_rgb(28, 28, 30)
            } else {
                egui::Color32::from_rgb(255, 255, 255)
            };
            let available_rect = ctx.available_rect();
            ctx.layer_painter(LayerId::background())
                .rect_filled(available_rect, 0.0, webview_bg);

            // If the top parts of the GUI changed size, then update the size of the WebView and also
            // the size of its RenderingContext.
            let rect = ctx.available_rect();
            let size = Size2D::new(rect.width(), rect.height()) * scale;
            let rect = Box2D::from_origin_and_size(Point2D::origin(), size);
            if let Some(webview) = window.active_webview()
                && rect != webview.rect()
            {
                // `rect` is sized to just the WebView viewport, which is required by
                // `OffscreenRenderingContext` See:
                // <https://github.com/servo/servo/issues/38369#issuecomment-3138378527>
                webview.resize(PhysicalSize::new(size.width as u32, size.height as u32))
            }

            if let Some(status_text) = &self.status_text {
                egui::Tooltip::always_open(
                    ctx.clone(),
                    LayerId::background(),
                    "tooltip layer".into(),
                    pos2(0.0, ctx.available_rect().max.y),
                )
                .show(|ui| ui.add(Label::new(status_text.clone()).extend()));
            }

            window.repaint_webviews();

            if let Some(render_to_parent) = rendering_context.render_to_parent_callback() {
                ctx.layer_painter(LayerId::background()).add(PaintCallback {
                    rect: ctx.available_rect(),
                    callback: Arc::new(CallbackFn::new(move |info, painter| {
                        let clip = info.viewport_in_pixels();
                        let rect_in_parent = Rect::new(
                            Point2D::new(clip.left_px, clip.from_bottom_px),
                            Size2D::new(clip.width_px, clip.height_px),
                        );
                        render_to_parent(painter.gl(), rect_in_parent)
                    })),
                });
            }
        });
    }

    /// Paint the GUI, as of the last update.
    pub(crate) fn paint(&mut self, window: &Window) {
        self.rendering_context
            .parent_context()
            .prepare_for_rendering();
        self.context.paint(window);
        self.rendering_context.parent_context().present();
    }

    /// Updates the location field from the given [`RunningAppState`], unless the user has started
    /// editing it without clicking Go, returning true iff it has changed (needing an egui update).
    fn update_location_in_toolbar(&mut self, window: &ServoShellWindow) -> bool {
        // User edited without clicking Go?
        if self.location_dirty {
            return false;
        }

        let current_url_string = window
            .active_webview()
            .and_then(|webview| Some(webview.url()?.to_string()));
        match current_url_string {
            Some(location) if location != self.location => {
                self.location = location.to_owned();
                true
            },
            _ => false,
        }
    }

    pub(crate) fn update_location_dirty(&mut self, dirty: bool) {
        self.location_dirty = dirty;
    }

    fn update_load_status(&mut self, window: &ServoShellWindow) -> bool {
        let state_status = window
            .active_webview()
            .map(|webview| webview.load_status())
            .unwrap_or(LoadStatus::Complete);
        let old_status = std::mem::replace(&mut self.load_status, state_status);
        old_status != self.load_status
    }

    fn update_status_text(&mut self, window: &ServoShellWindow) -> bool {
        let state_status = window
            .active_webview()
            .and_then(|webview| webview.status_text());
        let old_status = std::mem::replace(&mut self.status_text, state_status);
        old_status != self.status_text
    }

    fn update_can_go_back_and_forward(&mut self, window: &ServoShellWindow) -> bool {
        let (can_go_back, can_go_forward) = window
            .active_webview()
            .map(|webview| (webview.can_go_back(), webview.can_go_forward()))
            .unwrap_or((false, false));
        let old_can_go_back = std::mem::replace(&mut self.can_go_back, can_go_back);
        let old_can_go_forward = std::mem::replace(&mut self.can_go_forward, can_go_forward);
        old_can_go_back != self.can_go_back || old_can_go_forward != self.can_go_forward
    }

    /// Updates all fields taken from the given [`ServoShellWindow`], such as the location field.
    /// Returns true iff the egui needs an update.
    pub(crate) fn update_webview_data(&mut self, window: &ServoShellWindow) -> bool {
        // Note: We must use the "bitwise OR" (|) operator here instead of "logical OR" (||)
        //       because logical OR would short-circuit if any of the functions return true.
        //       We want to ensure that all functions are called. The "bitwise OR" operator
        //       does not short-circuit.
        self.update_location_in_toolbar(window)
            | self.update_load_status(window)
            | self.update_status_text(window)
            | self.update_can_go_back_and_forward(window)
    }

    /// Returns true if a redraw is required after handling the provided event.
    pub(crate) fn handle_accesskit_event(
        &mut self,
        event: &egui_winit::accesskit_winit::WindowEvent,
    ) -> bool {
        match event {
            egui_winit::accesskit_winit::WindowEvent::InitialTreeRequested => {
                self.context.egui_ctx.enable_accesskit();
                true
            },
            egui_winit::accesskit_winit::WindowEvent::ActionRequested(req) => {
                self.context
                    .egui_winit
                    .on_accesskit_action_request(req.clone());
                true
            },
            egui_winit::accesskit_winit::WindowEvent::AccessibilityDeactivated => {
                self.context.egui_ctx.disable_accesskit();
                false
            },
        }
    }

    pub(crate) fn set_zoom_factor(&self, factor: f32) {
        self.context.egui_ctx.set_zoom_factor(factor);
    }
}

fn embedder_image_to_egui_image(image: &Image) -> egui::ColorImage {
    let width = image.width as usize;
    let height = image.height as usize;

    match image.format {
        PixelFormat::K8 => egui::ColorImage::from_gray([width, height], image.data()),
        PixelFormat::KA8 => {
            // Convert to rgba
            let data: Vec<u8> = image
                .data()
                .chunks_exact(2)
                .flat_map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
                .collect();
            egui::ColorImage::from_rgba_unmultiplied([width, height], &data)
        },
        PixelFormat::RGB8 => egui::ColorImage::from_rgb([width, height], image.data()),
        PixelFormat::RGBA8 => {
            egui::ColorImage::from_rgba_unmultiplied([width, height], image.data())
        },
        PixelFormat::BGRA8 => {
            // Convert from BGRA to RGBA
            let data: Vec<u8> = image
                .data()
                .chunks_exact(4)
                .flat_map(|chunk| [chunk[2], chunk[1], chunk[0], chunk[3]])
                .collect();
            egui::ColorImage::from_rgba_unmultiplied([width, height], &data)
        },
    }
}

/// Uploads all favicons that have not yet been processed to the GPU.
fn load_pending_favicons(
    ctx: &egui::Context,
    window: &ServoShellWindow,
    texture_cache: &mut HashMap<WebViewId, (egui::TextureHandle, egui::load::SizedTexture)>,
) {
    for id in window.take_pending_favicon_loads() {
        let Some(webview) = window.webview_by_id(id) else {
            continue;
        };
        let Some(favicon) = webview.favicon() else {
            continue;
        };

        let egui_image = embedder_image_to_egui_image(&favicon);
        let handle = ctx.load_texture(format!("favicon-{id:?}"), egui_image, Default::default());
        let texture = egui::load::SizedTexture::new(
            handle.id(),
            egui::vec2(favicon.width as f32, favicon.height as f32),
        );

        // We don't need the handle anymore but we can't drop it either since that would cause
        // the texture to be freed.
        texture_cache.insert(id, (handle, texture));
    }
}

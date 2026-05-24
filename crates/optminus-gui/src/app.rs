//! Root view: header (device carousel placeholder), body (mouse model area
//! and configuration panel placeholder), footer (settings / version).
//!
//! Phase 1 of UI.md: the body currently holds six circular buttons that
//! breathe a blue glow on hover, validating the `with_animation` API. They
//! will be replaced by the real mouse model + config panel in later phases.

use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, AnyElement, BoxShadow, Context, Entity, FontWeight,
    InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled,
    Window, div, ease_in_out, point, px, rgb,
};
use gpui_component::{ActiveTheme, h_flex, v_flex};
use optminus_core::device::DeviceInventory;

use crate::state::AppState;
use crate::theme::{
    ACCENT_BLUE, BG_DARK, BORDER, FOOTER_H, HEADER_H, SURFACE, SURFACE_HOVER, TEXT_MUTED,
    TEXT_PRIMARY, accent_glow,
};

/// Application root view.
pub struct AppView {
    /// Inventory snapshot from the startup HID probe. Will feed the carousel
    /// in Phase 3; held here so the data survives the restructuring.
    inventories: Vec<DeviceInventory>,
    /// Index of the currently hovered Phase 1 demo button (`None` = nothing
    /// hovered). Stored on the view rather than in `AppState` because it's
    /// purely demo-screen scratch state.
    hovered_demo_button: Option<usize>,
}

impl AppView {
    pub fn new(inventories: Vec<DeviceInventory>, cx: &mut Context<Self>) -> Self {
        // Seed the global on first construction; later views (DPI panel,
        // popover) will mutate it.
        if !cx.has_global::<AppState>() {
            cx.set_global(AppState::new());
        }
        Self {
            inventories,
            hovered_demo_button: None,
        }
    }
}

impl Render for AppView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        v_flex()
            .size_full()
            .bg(rgb(BG_DARK))
            .text_color(rgb(TEXT_PRIMARY))
            .child(header(self.inventories.len()))
            .child(body(self.hovered_demo_button, &entity))
            .child(footer(cx))
    }
}

fn header(device_count: usize) -> impl IntoElement {
    // Placeholder strip — Phase 3 will replace this with the carousel proper.
    h_flex()
        .h(px(HEADER_H))
        .w_full()
        .px_5()
        .gap_3()
        .items_center()
        .border_b_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::SEMIBOLD)
                .child("Options−"),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(TEXT_MUTED))
                .child(format!("{device_count} receivers")),
        )
}

fn body(hovered: Option<usize>, entity: &Entity<AppView>) -> impl IntoElement {
    h_flex()
        .flex_1()
        .w_full()
        .min_h_0()
        .items_center()
        .justify_center()
        .gap_8()
        .p_8()
        .child(phase1_pulse_row(hovered, entity))
}

fn footer(cx: &Context<AppView>) -> impl IntoElement {
    let theme = cx.theme();
    h_flex()
        .h(px(FOOTER_H))
        .w_full()
        .px_5()
        .gap_4()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child("Settings · About"),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(concat!("v", env!("CARGO_PKG_VERSION"))),
        )
}

// --- Phase 1 demo ------------------------------------------------------------

const DEMO_BUTTON_COUNT: usize = 6;

fn phase1_pulse_row(hovered: Option<usize>, entity: &Entity<AppView>) -> impl IntoElement {
    h_flex()
        .gap_5()
        .items_center()
        .children((0..DEMO_BUTTON_COUNT).map(|idx| pulse_button(idx, hovered == Some(idx), entity)))
}

fn pulse_button(idx: usize, hovered: bool, entity: &Entity<AppView>) -> AnyElement {
    let base = div()
        .id(("pulse-btn", idx))
        .size(px(64.))
        .rounded_full()
        .bg(rgb(if hovered { SURFACE_HOVER } else { SURFACE }))
        .border_1()
        .border_color(rgb(if hovered { ACCENT_BLUE } else { BORDER }))
        .flex()
        .items_center()
        .justify_center()
        .text_color(rgb(TEXT_MUTED))
        .child(format!("{}", idx + 1))
        .hover(|s| s.bg(rgb(SURFACE_HOVER)))
        .on_hover({
            let entity = entity.clone();
            move |is_hovered, _window, cx| {
                let is_hovered = *is_hovered;
                entity.update(cx, |this, cx| {
                    if is_hovered {
                        this.hovered_demo_button = Some(idx);
                    } else if this.hovered_demo_button == Some(idx) {
                        this.hovered_demo_button = None;
                    }
                    cx.notify();
                });
            }
        });

    if hovered {
        // sin(delta * PI) gives a 0→1→0 bell over each cycle, so the glow
        // breathes in and out without a snap at the loop boundary.
        base.with_animation(
            ("pulse", idx),
            Animation::new(Duration::from_millis(1400))
                .repeat()
                .with_easing(ease_in_out),
            |this, delta| {
                let intensity = (delta * std::f32::consts::PI).sin();
                this.shadow(vec![BoxShadow {
                    color: accent_glow(intensity),
                    offset: point(px(0.), px(0.)),
                    blur_radius: px(8. + intensity * 16.),
                    spread_radius: px(2.),
                }])
            },
        )
        .into_any_element()
    } else {
        base.into_any_element()
    }
}

//! GPUI window listing connected Logitech HID++ devices.
//!
//! v0.0.1: static render. We enumerate devices once on startup (via tokio),
//! then hand the result to the GPUI view. Live polling lands when there is
//! something to react to (device connect/disconnect events).

use anyhow::{Context as _, Result};
use gpui::{
    AppContext, Context, IntoElement, ParentElement, Render, Styled, Window, WindowOptions, div,
    prelude::FluentBuilder,
};
use gpui_component::{ActiveTheme, Root, StyledExt};
use optminus_core::device::{BatteryInfo, DeviceInventory, PairedDevice};
use tracing_subscriber::EnvFilter;

/// View backing the main window.
pub struct DeviceListView {
    inventories: Vec<DeviceInventory>,
}

impl Render for DeviceListView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let header = div().text_lg().child("OptMinus — Devices");
        div()
            .v_flex()
            .gap_4()
            .p_6()
            .size_full()
            .child(header)
            .when(self.inventories.is_empty(), |this| {
                this.child("No Logitech HID++ receivers found.")
            })
            .when(!self.inventories.is_empty(), |this| {
                this.children(self.inventories.iter().map(render_inventory))
            })
    }
}

fn render_inventory(inv: &DeviceInventory) -> impl IntoElement {
    let header = format!(
        "{}  ({}, vid={:04x} pid={:04x})",
        inv.receiver.name,
        inv.receiver.unique_id.as_deref().unwrap_or("—"),
        inv.receiver.vendor_id,
        inv.receiver.product_id,
    );
    div()
        .v_flex()
        .gap_1()
        .child(header)
        .when(inv.paired.is_empty(), |this| {
            this.child(div().pl_4().child("no paired devices"))
        })
        .when(!inv.paired.is_empty(), |this| {
            this.children(
                inv.paired
                    .iter()
                    .map(|d| div().pl_4().child(format_device(d))),
            )
        })
}

fn format_device(d: &PairedDevice) -> String {
    let dot = if d.online { "●" } else { "○" };
    let kind = format!("{:?}", d.kind).to_lowercase();
    let wpid = d
        .wpid
        .map_or_else(|| "wpid=?".to_string(), |w| format!("wpid={w:04x}"));
    let battery = d
        .battery
        .as_ref()
        .map_or_else(|| "battery=—".to_string(), format_battery);
    format!(
        "slot {} {dot} {} ({kind}, {wpid}, {battery})",
        d.slot,
        d.codename.as_deref().unwrap_or("Unknown device"),
    )
}

fn format_battery(b: &BatteryInfo) -> String {
    let level = format!("{:?}", b.level).to_lowercase();
    let status = format!("{:?}", b.status).to_lowercase();
    format!("battery={}% {level} ({status})", b.percentage)
}

fn main() -> Result<()> {
    init_tracing();

    // GPUI owns the main thread, so run the one-shot HID probe synchronously
    // first and pass the result into the application closure.
    let inventories = enumerate_blocking().context("HID enumeration failed")?;

    gpui_platform::application().run(move |cx| {
        gpui_component::init(cx);
        cx.spawn(async move |cx| {
            #[allow(
                clippy::expect_used,
                reason = "failure to open the main window is fatal; nothing useful to recover to"
            )]
            cx.open_window(WindowOptions::default(), move |window, cx| {
                let view = cx.new(|_| DeviceListView { inventories });
                cx.new(|cx| Root::new(view, window, cx).bg(cx.theme().background))
            })
            .expect("opening the main window should not fail");
        })
        .detach();
    });

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_env("OPTMINUS_LOG").unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}

fn enumerate_blocking() -> Result<Vec<DeviceInventory>> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("tokio runtime init")?;
    rt.block_on(optminus_hid::enumerate())
        .context("optminus_hid::enumerate")
}

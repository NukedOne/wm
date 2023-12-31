use penrose::{
    builtin::hooks::SpacingHook,
    core::{bindings::parse_keybindings_with_xmodmap, Config, WindowManager},
    extensions::hooks::{
        add_ewmh_hooks, add_named_scratchpads, manage::FloatingCentered, NamedScratchPad,
        SpawnOnStartup,
    },
    x::query::ClassName,
    x11rb::RustConn,
    Result,
};
use std::{
    collections::HashMap,
    process::{ChildStdin, Command, Stdio},
};
use tracing_subscriber::{self, prelude::*};
use wm::actions::{add_fixed_workspaces_state, add_namedscratchpads_state, add_xmobar_handle};
use wm::bindings::raw_key_bindings;
use wm::layouts::layouts;
use wm::{BAR_HEIGHT_PX, INNER_PX, OUTER_PX};

fn main() -> Result<()> {
    let file_appender = tracing_appender::rolling::daily("/home/alex/wmlogs", "log_");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let tracing_builder = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_writer(non_blocking)
        .with_filter_reloading();

    let reload_handle = tracing_builder.reload_handle();
    tracing_builder.finish().init();

    let layout_hook = SpacingHook {
        inner_px: INNER_PX,
        outer_px: OUTER_PX,
        top_px: BAR_HEIGHT_PX,
        bottom_px: 0,
    };

    let startup_hook = SpawnOnStartup::boxed("/home/alex/bin/wm_startup");

    let config = add_ewmh_hooks(Config {
        default_layouts: layouts(),
        layout_hook: Some(Box::new(layout_hook)),
        startup_hook: Some(startup_hook),
        floating_classes: vec!["pia-client".to_owned()],
        tags: (1..=12).map(|n| n.to_string()).collect::<Vec<String>>(),
        ..Config::default()
    });

    let (nsp, toggle_scratch) = NamedScratchPad::new(
        "term",
        "alacritty --class __text_scratchpad",
        ClassName("__text_scratchpad"),
        FloatingCentered::new(0.8, 0.8),
        true,
    );

    let (nsp2, toggle_scratch2) = NamedScratchPad::new(
        "vpn",
        "protonvpn-app",
        ClassName("Protonvpn-app"),
        FloatingCentered::new(0.8, 0.8),
        true,
    );

    let conn = RustConn::new()?;
    let key_bindings = parse_keybindings_with_xmodmap(raw_key_bindings(
        reload_handle,
        toggle_scratch,
        toggle_scratch2,
    ))?;

    let xmobar_handle = get_xmobar_handle()?;

    let wm = add_namedscratchpads_state(
        add_named_scratchpads(
            add_xmobar_handle(
                add_fixed_workspaces_state(WindowManager::new(
                    config,
                    key_bindings,
                    HashMap::new(),
                    conn,
                )?),
                xmobar_handle,
            ),
            vec![nsp, nsp2],
        ),
        vec!["term", "vpn"],
    );

    wm.run()
}

#[cfg(not(feature = "laptop"))]
fn get_xmobar_handle() -> Result<ChildStdin> {
    Command::new("xmobar")
        .args(["/home/alex/.config/xmobar/xmobarrc_0", "-x", "0"])
        .spawn()?;

    let mut xmobar_right = Command::new("xmobar")
        .args(["/home/alex/.config/xmobar/xmobarrc_1", "-x", "1"])
        .stdin(Stdio::piped())
        .spawn()?;

    Ok(xmobar_right.stdin.take().unwrap())
}

#[cfg(feature = "laptop")]
fn get_xmobar_handle() -> Result<ChildStdin> {
    let mut xmobar = Command::new("xmobar")
        .arg("/home/alex/.config/xmobar/xmobarrc")
        .stdin(Stdio::piped())
        .spawn()?;

    Ok(xmobar.stdin.take().unwrap())
}

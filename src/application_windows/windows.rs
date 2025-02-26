use crate::error::AppResult;
use anyhow::Context;
use bevy::app::{App, Plugin, Update};
use bevy::log::error;
use bevy::prelude::*;
use bevy::window::Window;
use bevy::winit::WinitWindows;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{SetWindowLongW, GWL_EXSTYLE, GWL_STYLE, WS_POPUP};
use winit::raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

pub struct ApplicationWindowOnWindowsPlugin;


impl Plugin for ApplicationWindowOnWindowsPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<UninitializedWindow>()
            .add_systems(Update, setup_window_to_transparent);

        app
            .world_mut()
            .register_component_hooks::<Window>()
            .on_add(|mut world, entity, _| {
                world.commands().entity(entity).insert(UninitializedWindow);
            });
    }
}

#[derive(Debug, Component, Eq, PartialEq, Copy, Clone, Reflect)]
#[reflect(Component)]
struct UninitializedWindow;

fn setup_window_to_transparent(
    mut commands: Commands,
    winit_windows: NonSend<WinitWindows>,
    windows: Query<Entity, With<UninitializedWindow>>,
) {
    for window in windows.iter() {
        let Some(winit_window) = winit_windows.get_window(window) else {
            return;
        };
        if let Err(e) = set_transparent(winit_window) {
            error!("Failed to set transparent: {:?}", e);
        }
        commands.entity(window).remove::<UninitializedWindow>();
    }
}

fn set_transparent(
    winit_window: &winit::window::Window,
) -> AppResult {
    let hwnd = obtain_window_handle_from_winit_window(winit_window)
        .context("Failed to obtain the window handle")?;
    unsafe {
        const WS_EX_LAYERD: i32 = 0x080000;
        const WS_EX_TRANSPARENT: i32 = 0x00000020;

        SetWindowLongW(hwnd, GWL_STYLE, WS_POPUP.0 as i32);
        SetWindowLongW(hwnd, GWL_EXSTYLE, WS_EX_LAYERD | WS_EX_TRANSPARENT);

        let mut margin = windows::Win32::UI::Controls::MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: -1,
            cyTopHeight: -1,
            cyBottomHeight: -1,
        };
        windows::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea(
            hwnd,
            &mut margin,
        )?;
    }

    Ok(())
}

fn obtain_window_handle_from_winit_window(
    winit_window: &winit::window::Window,
) -> Option<HWND> {
    let handle = winit_window.raw_window_handle().ok()?;
    if let RawWindowHandle::Win32(handle) = handle {
        Some(HWND(handle.hwnd.get() as *mut _))
    } else {
        None
    }
}
use crate::application_windows::setup::A;
use crate::error::AppResult;
use anyhow::Context;
use bevy::app::{App, Plugin, Update};
use bevy::log::error;
use bevy::prelude::*;
use bevy::window::{Window, WindowWrapper};
use bevy::winit::WinitWindows;
use windows::Win32::Foundation::{COLORREF, HWND};
use windows::Win32::UI::WindowsAndMessaging::{SetLayeredWindowAttributes, SetWindowLongW, GWL_EXSTYLE, GWL_STYLE, LWA_ALPHA, LWA_COLORKEY, WS_POPUP};
use winit::raw_window_handle::{DisplayHandle, HasDisplayHandle, HasRawWindowHandle, HasWindowHandle, RawWindowHandle};
use winit::window::Theme;

pub struct ApplicationWindowOnWindowsPlugin;


impl Plugin for ApplicationWindowOnWindowsPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<UninitializedWindow>()
            .add_systems(Update, (
                setup_window_to_transparent,
                draw_buffer,
            ));

        app
            .world_mut()
            .register_component_hooks::<A>()
            .on_add(|mut world, entity, _| {
                world.commands().entity(entity).insert(UninitializedWindow);
            });
    }
}

#[derive(Component)]
pub struct DisplayContext(softbuffer::Context<DisplayHandle<'static>>);

#[derive(Debug, Component, Eq, PartialEq, Copy, Clone, Reflect)]
#[reflect(Component)]
struct UninitializedWindow;

fn setup_window_to_transparent(
    mut commands: Commands,
    winit_windows: NonSend<WinitWindows>,
    windows: Query<Entity, With<UninitializedWindow>>,
) {
    for window in windows.iter() {
        let Some(display_context) = winit_windows
            .get_window(window)
            .and_then(|winit_window| winit_window.display_handle().ok())
            .and_then(|display_handle| softbuffer::Context::new(display_handle).ok())
        else {
            return;
        };
        commands.entity(window).insert(DisplayContext(display_context));
        // if let Err(e) = set_transparent(winit_window) {
        //     error!("Failed to set transparent: {:?}", e);
        // }
        commands.entity(window).remove::<UninitializedWindow>();
    }
}

fn draw_buffer(
    winit_windows: NonSend<WinitWindows>,
    windows: Query<(Entity, &DisplayContext)>,
) {
    for (entity, context) in windows.iter() {
        let Some(winit_window) = winit_windows.get_window(entity) else {
            return;
        };
        let Ok(mut surface) = softbuffer::Surface::new(
            &context.0,
            winit_window.window_handle().unwrap(),
        ) else {
            return;
        };
        let Ok(mut buffer) = surface.buffer_mut() else {
            return;
        };

        let safe_area_size = winit_window.inner_size();
        for y in 0..safe_area_size.height {
            for x in 0..safe_area_size.width {
                let index = (y * safe_area_size.width + x) as usize;
                buffer[index] = 0x00000000;
            }
        }

        winit_window.pre_present_notify();
        let _ = buffer.present();
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


        // SetWindowLongW(hwnd, GWL_STYLE, WS_POPUP.0 as i32);
        // SetWindowLongW(hwnd, GWL_EXSTYLE, WS_EX_LAYERD);
        // SetLayeredWindowAttributes(hwnd, COLORREF(0), 0, LWA_COLORKEY);
        // SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA);
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
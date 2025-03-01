use crate::application_windows::setup::A;
use crate::error::AppResult;
use anyhow::Context;
use bevy::app::{App, Plugin, Update};
use bevy::core_pipeline::core_3d::graph::{Core3d, Node3d};
use bevy::ecs::query::QueryItem;
use bevy::log::error;
use bevy::prelude::*;
use bevy::render::graph::CameraDriverLabel;
use bevy::render::render_graph::{NodeRunError, RenderGraph, RenderGraphApp, RenderGraphContext, ViewNodeRunner};
use bevy::render::renderer::{render_system, RenderContext, WgpuWrapper};
use bevy::render::view::ExtractedWindows;
use bevy::render::{Render, RenderApp};
use bevy::utils::HashMap;
use bevy::window::{PrimaryWindow, Window, WindowWrapper};
use bevy::winit::WinitWindows;
use std::num::NonZeroU32;
use windows::Win32::Foundation::{COLORREF, HWND};
use windows::Win32::UI::WindowsAndMessaging::{SetLayeredWindowAttributes, SetWindowLongW, GWL_EXSTYLE, GWL_STYLE, LWA_ALPHA, LWA_COLORKEY, WS_POPUP};
use winit::raw_window_handle::{DisplayHandle, HasDisplayHandle, HasRawWindowHandle, HasWindowHandle, RawWindowHandle, WindowHandle};
use winit::window::Theme;

pub struct ApplicationWindowOnWindowsPlugin;


impl Plugin for ApplicationWindowOnWindowsPlugin {
    fn build(&self, app: &mut App) {
        // app
        //     .register_type::<UninitializedWindow>()
        //     .init_non_send_resource::<Surfaces>()
        //     .add_systems(Update, (
        //         draw_buffer,
        //     ));
        app
        .add_systems(Startup, |mut commands: Commands, window: Query<Entity, With<PrimaryWindow>>|{
            commands.entity(window.single()).insert(A);
        })
        .add_systems(Update, setup_window_to_transparent);
        let mut render_app = app.sub_app_mut(RenderApp);
        // render_app.add_render_graph_node::<ViewNodeRunner<TransparentWindowNode>>(
        //     Core3d,
        //     Node3d::PostProcessing,
        // );
        render_app.add_systems(bevy::render::Render, draw_buffer
            .after(render_system));
        // let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        // render_graph.add_node(CameraDriverLabel, TransparentWindowNode);
        app
            .world_mut()
            .register_component_hooks::<A>()
            .on_add(|mut world, entity, _| {
                world.commands().entity(entity).insert(UninitializedWindow);
            });
    }
}

#[derive(Default)]
pub struct Surfaces(HashMap<Entity, softbuffer::Surface<DisplayHandle<'static>, WindowHandle<'static>>>);

pub struct DisplayContext(softbuffer::Context<DisplayHandle<'static>>);

#[derive(Debug, Component, Eq, PartialEq, Copy, Clone, Reflect)]
#[reflect(Component)]
struct UninitializedWindow;

fn setup_window_to_transparent(
    mut commands: Commands,
    winit_windows: NonSend<WinitWindows>,
    windows: Query<Entity, With<A>>,
) {
    for window in windows.iter() {
        let Some(winit_window) = winit_windows.get_window(window) else {
            continue;
        };
        set_transparent(winit_window).unwrap();
        commands.entity(window).remove::<A>();
    }
}


#[derive(Default)]
struct TransparentWindowNode;


impl bevy::render::render_graph::ViewNode for TransparentWindowNode {
    type ViewQuery = ();

    fn run<'w>(&self, graph: &mut RenderGraphContext, render_context: &mut RenderContext<'w>, view_query: QueryItem<'w, Self::ViewQuery>, world: &'w World) -> Result<(), NodeRunError> {
        let windows = world.resource::<ExtractedWindows>();
        for (entity, window) in windows.iter() {
            let Ok(display_context) = softbuffer::Context::new(
                unsafe {
                    DisplayHandle::borrow_raw(window.handle.display_handle)
                }
            ) else {
                continue;
            };

            let Ok(mut surface) = softbuffer::Surface::new(
                &display_context,
                unsafe {
                    WindowHandle::borrow_raw(window.handle.window_handle)
                },
            ) else {
                continue;
            };
            surface.resize(
                NonZeroU32::new(window.physical_width).unwrap(),
                NonZeroU32::new(window.physical_height).unwrap(),
            );
            let Ok(mut buffer) = surface.buffer_mut() else {
                continue;
            };

            for y in 0..window.physical_height {
                for x in 0..window.physical_width {
                    let index = (y * window.physical_width + x) as usize;
                    buffer[index] = 0xFF00FF00;
                }
            }

            let _ = buffer.present();
        }
        Ok(())
    }
}

fn draw_buffer(
    windows: Res<ExtractedWindows>,
) {
    for (entity, window) in windows.iter() {
        let Ok(display_context) = softbuffer::Context::new(
            unsafe {
                DisplayHandle::borrow_raw(window.handle.display_handle)
            }
        ) else {
            continue;
        };

        let Ok(mut surface) = softbuffer::Surface::new(
            &display_context,
            unsafe {
                WindowHandle::borrow_raw(window.handle.window_handle)
            },
        ) else {
            continue;
        };
        surface.resize(
            NonZeroU32::new(window.physical_width).unwrap(),
            NonZeroU32::new(window.physical_height).unwrap(),
        );
        let Ok(mut buffer) = surface.buffer_mut() else {
            continue;
        };

        for y in 0..window.physical_height {
            for x in 0..window.physical_width {
                let index = (y * window.physical_width + x) as usize;
                buffer[index] = 0xFF000000;
            }
        }
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
        SetWindowLongW(hwnd, GWL_EXSTYLE, WS_EX_LAYERD | WS_EX_TRANSPARENT);
        SetLayeredWindowAttributes(hwnd, COLORREF(0x00000000), 0, LWA_COLORKEY);
        // SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA);
        let mut margin = windows::Win32::UI::Controls::MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: 0,
            cyTopHeight: 0,
            cyBottomHeight: 0,
        };
        println!("Transparent");
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
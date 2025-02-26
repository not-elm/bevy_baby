#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
pub use crate::global_window::macos::*;

use bevy::math::{Rect, Vec2};
use bevy::prelude::Resource;

#[derive(Debug, Clone, PartialEq, Default, Resource)]
pub struct GlobalWindow {
    pub title: Option<String>,
    pub frame: Rect,
    #[cfg(target_os = "macos")]
    pub window_layer: i64,
    #[cfg(target_os = "macos")]
    pub window_number: objc2_foundation::NSInteger,
}

impl GlobalWindow {
    #[inline]
    pub fn sitting_pos(&self, drop_pos: Vec2) -> Vec2 {
        Vec2::new(drop_pos.x, self.frame.min.y)
    }

    /// Update the application_windows metadata.
    ///
    /// Returns `true` if the application_windows position has been updated.
    #[inline]
    pub fn update(&self) -> Option<GlobalWindow> {
        #[cfg(target_os = "macos")]
        {
            //TODO: Not implemented yet.
            if let Some(updated) = find_window_from_number(self.window_number) {
                if updated.frame != self.frame {
                    return Some(updated);
                }
            }
        }

        None
    }
}

#[derive(Debug, Default)]
pub struct GlobalWindows(Vec<GlobalWindow>);

impl GlobalWindows {
    pub const fn new(frames: Vec<GlobalWindow>) -> Self {
        Self(frames)
    }

    pub fn find_sitting_window(&self, drop_pos: Vec2) -> Option<GlobalWindow> {
        const SITTING_THRESHOLD_HEIGHT: f32 = 80.;
        let mut areas = Vec::new();
        for sitting_area in self.0.iter() {
            if hitting_sitting_area(&sitting_area.frame, drop_pos, SITTING_THRESHOLD_HEIGHT)
                && !areas.iter().any(|area: &&GlobalWindow| area.frame.contains(drop_pos))
            {
                return Some(sitting_area.clone());
            }
            areas.push(sitting_area);
        }
        None
    }
}

fn hitting_sitting_area(
    window_frame: &Rect,
    drop_viewport_point: Vec2,
    threshold_height: f32,
) -> bool {
    let min = window_frame.min;
    let max = window_frame.max;
    let sitting_area = Rect::from_corners(Vec2::new(min.x, min.y - threshold_height), Vec2::new(max.x, min.y));
    sitting_area.contains(drop_viewport_point)
}

#[cfg(test)]
mod tests {
    use crate::global_window::{hitting_sitting_area, GlobalWindow, GlobalWindows};
    use bevy::math::{Rect, Vec2};

    #[test]
    fn test_sitting_area() {
        assert!(hitting_sitting_area(
            &Rect::from_corners(Vec2::new(0., 0.), Vec2::new(100., 100.)),
            Vec2::new(0., 0.),
            40.,
        ));
    }

    #[test]
    fn exclude_intersect_area() {
        let areas = GlobalWindows::new(vec![
            GlobalWindow {
                frame: Rect::from_corners(Vec2::new(0., 0.), Vec2::new(100., 100.)),
                ..Default::default()
            },
            GlobalWindow {
                frame: Rect::from_corners(Vec2::splat(50.), Vec2::splat(100.)),
                ..Default::default()
            },
        ]);
        assert_eq!(areas.find_sitting_window(Vec2::new(50., 50.)), None);
    }

    #[test]
    fn intersect_area() {
        let bottom = GlobalWindow {
            frame: Rect::from_corners(Vec2::splat(50.), Vec2::splat(150.)),
            ..Default::default()
        };
        let areas = GlobalWindows::new(vec![
            GlobalWindow {
                frame: Rect::from_corners(Vec2::new(0., 0.), Vec2::new(100., 100.)),
                ..Default::default()
            },
            bottom.clone(),
        ]);
        assert_eq!(areas.find_sitting_window(Vec2::new(110., 50.)), Some(bottom));
    }
}


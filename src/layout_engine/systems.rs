#![allow(clippy::too_many_arguments)]

use enum_dispatch::enum_dispatch;
use objc2_core_foundation::CGRect;
use serde::{Deserialize, Serialize};

use crate::actor::app::{WindowId, pid_t};
use crate::layout_engine::{Direction, LayoutKind};

slotmap::new_key_type! { pub struct LayoutId; }

pub trait LayoutLifecycle: Send + Serialize + for<'de> Deserialize<'de> {
    fn create_layout(&mut self) -> LayoutId;
    fn clone_layout(&mut self, layout: LayoutId) -> LayoutId;
    fn remove_layout(&mut self, layout: LayoutId);
}

pub trait LayoutCore: Send + Serialize + for<'de> Deserialize<'de> {
    fn draw_tree(&self, layout: LayoutId) -> String;
    fn calculate_layout(
        &self,
        layout: LayoutId,
        screen: CGRect,
        stack_offset: f64,
        gaps: &crate::common::config::GapSettings,
        stack_line_thickness: f64,
        stack_line_horiz: crate::common::config::HorizontalPlacement,
        stack_line_vert: crate::common::config::VerticalPlacement,
    ) -> Vec<(WindowId, CGRect)>;
    fn visible_windows_in_layout(&self, layout: LayoutId) -> Vec<WindowId>;
    fn visible_windows_under_selection(&self, layout: LayoutId) -> Vec<WindowId>;
    fn selected_window(&self, layout: LayoutId) -> Option<WindowId>;
    fn contains_window(&self, layout: LayoutId, wid: WindowId) -> bool;
    fn select_window(&mut self, layout: LayoutId, wid: WindowId) -> bool;
    fn add_window_after_selection(&mut self, layout: LayoutId, wid: WindowId);
    fn remove_window(&mut self, wid: WindowId);
    fn remove_windows_for_app(&mut self, pid: pid_t);
    fn set_windows_for_app(&mut self, layout: LayoutId, pid: pid_t, desired: Vec<WindowId>);
    fn has_windows_for_app(&self, layout: LayoutId, pid: pid_t) -> bool;
    fn on_window_resized(
        &mut self,
        layout: LayoutId,
        wid: WindowId,
        old_frame: CGRect,
        new_frame: CGRect,
        screen: CGRect,
        gaps: &crate::common::config::GapSettings,
    );
}

pub trait LayoutNavigation {
    fn ascend_selection(&mut self, layout: LayoutId) -> bool;
    fn descend_selection(&mut self, layout: LayoutId) -> bool;
    fn move_focus(
        &mut self,
        layout: LayoutId,
        direction: Direction,
    ) -> (Option<WindowId>, Vec<WindowId>);
    fn window_in_direction(&self, layout: LayoutId, direction: Direction) -> Option<WindowId>;
    fn move_selection(&mut self, layout: LayoutId, direction: Direction) -> bool;
    fn move_selection_to_layout_after_selection(
        &mut self,
        from_layout: LayoutId,
        to_layout: LayoutId,
    );
}

pub trait LayoutResizable {
    fn resize_selection_by(&mut self, layout: LayoutId, amount: f64);
    fn rebalance(&mut self, layout: LayoutId);
}

pub trait LayoutSplittable {
    fn split_selection(&mut self, layout: LayoutId, kind: LayoutKind);
    fn toggle_tile_orientation(&mut self, layout: LayoutId);
    fn join_selection_with_direction(&mut self, layout: LayoutId, direction: Direction);
    fn unjoin_selection(&mut self, layout: LayoutId);
}

pub trait LayoutStacking {
    fn apply_stacking_to_parent_of_selection(
        &mut self,
        layout: LayoutId,
        default_orientation: crate::common::config::StackDefaultOrientation,
    ) -> Vec<WindowId>;
    fn unstack_parent_of_selection(
        &mut self,
        layout: LayoutId,
        default_orientation: crate::common::config::StackDefaultOrientation,
    ) -> Vec<WindowId>;
    fn parent_of_selection_is_stacked(&self, layout: LayoutId) -> bool;
}

pub trait LayoutSwappable {
    fn swap_windows(&mut self, layout: LayoutId, a: WindowId, b: WindowId) -> bool;
}

pub trait LayoutFullscreen {
    fn toggle_fullscreen_of_selection(&mut self, layout: LayoutId) -> Vec<WindowId>;
    fn toggle_fullscreen_within_gaps_of_selection(&mut self, layout: LayoutId) -> Vec<WindowId>;
}

#[enum_dispatch]
pub trait LayoutSystem: Serialize + for<'de> Deserialize<'de> {
    fn create_layout(&mut self) -> LayoutId;
    fn clone_layout(&mut self, layout: LayoutId) -> LayoutId;
    fn remove_layout(&mut self, layout: LayoutId);

    fn draw_tree(&self, layout: LayoutId) -> String;

    fn calculate_layout(
        &self,
        layout: LayoutId,
        screen: CGRect,
        stack_offset: f64,
        gaps: &crate::common::config::GapSettings,
        stack_line_thickness: f64,
        stack_line_horiz: crate::common::config::HorizontalPlacement,
        stack_line_vert: crate::common::config::VerticalPlacement,
    ) -> Vec<(WindowId, CGRect)>;

    fn selected_window(&self, layout: LayoutId) -> Option<WindowId>;
    fn visible_windows_in_layout(&self, layout: LayoutId) -> Vec<WindowId>;
    fn visible_windows_under_selection(&self, layout: LayoutId) -> Vec<WindowId>;
    fn ascend_selection(&mut self, layout: LayoutId) -> bool;
    fn descend_selection(&mut self, layout: LayoutId) -> bool;
    fn move_focus(
        &mut self,
        layout: LayoutId,
        direction: Direction,
    ) -> (Option<WindowId>, Vec<WindowId>);
    fn window_in_direction(&self, layout: LayoutId, direction: Direction) -> Option<WindowId>;
    fn add_window_after_selection(&mut self, layout: LayoutId, wid: WindowId);
    fn remove_window(&mut self, wid: WindowId);
    fn remove_windows_for_app(&mut self, pid: pid_t);
    fn set_windows_for_app(&mut self, layout: LayoutId, pid: pid_t, desired: Vec<WindowId>);
    fn has_windows_for_app(&self, layout: LayoutId, pid: pid_t) -> bool;
    fn contains_window(&self, layout: LayoutId, wid: WindowId) -> bool;
    fn select_window(&mut self, layout: LayoutId, wid: WindowId) -> bool;
    fn on_window_resized(
        &mut self,
        layout: LayoutId,
        wid: WindowId,
        old_frame: CGRect,
        new_frame: CGRect,
        screen: CGRect,
        gaps: &crate::common::config::GapSettings,
    );

    fn swap_windows(&mut self, layout: LayoutId, a: WindowId, b: WindowId) -> bool;

    fn move_selection(&mut self, layout: LayoutId, direction: Direction) -> bool;
    fn move_selection_to_layout_after_selection(
        &mut self,
        from_layout: LayoutId,
        to_layout: LayoutId,
    );
    fn split_selection(&mut self, layout: LayoutId, kind: LayoutKind);

    fn toggle_fullscreen_of_selection(&mut self, layout: LayoutId) -> Vec<WindowId>;
    fn toggle_fullscreen_within_gaps_of_selection(&mut self, layout: LayoutId) -> Vec<WindowId>;

    fn join_selection_with_direction(&mut self, layout: LayoutId, direction: Direction);
    fn apply_stacking_to_parent_of_selection(
        &mut self,
        layout: LayoutId,
        default_orientation: crate::common::config::StackDefaultOrientation,
    ) -> Vec<WindowId>;
    fn unstack_parent_of_selection(
        &mut self,
        layout: LayoutId,
        default_orientation: crate::common::config::StackDefaultOrientation,
    ) -> Vec<WindowId>;
    fn parent_of_selection_is_stacked(&self, layout: LayoutId) -> bool;
    fn unjoin_selection(&mut self, _layout: LayoutId);
    fn resize_selection_by(&mut self, layout: LayoutId, amount: f64);
    fn rebalance(&mut self, layout: LayoutId);
    fn toggle_tile_orientation(&mut self, layout: LayoutId);
}

impl<
    T: LayoutLifecycle
        + LayoutCore
        + LayoutNavigation
        + LayoutResizable
        + LayoutSplittable
        + LayoutStacking
        + LayoutSwappable
        + LayoutFullscreen,
> LayoutSystem for T
{
    fn create_layout(&mut self) -> LayoutId {
        LayoutLifecycle::create_layout(self)
    }
    fn clone_layout(&mut self, layout: LayoutId) -> LayoutId {
        LayoutLifecycle::clone_layout(self, layout)
    }
    fn remove_layout(&mut self, layout: LayoutId) {
        LayoutLifecycle::remove_layout(self, layout)
    }

    fn draw_tree(&self, layout: LayoutId) -> String {
        LayoutCore::draw_tree(self, layout)
    }

    fn calculate_layout(
        &self,
        layout: LayoutId,
        screen: CGRect,
        stack_offset: f64,
        gaps: &crate::common::config::GapSettings,
        stack_line_thickness: f64,
        stack_line_horiz: crate::common::config::HorizontalPlacement,
        stack_line_vert: crate::common::config::VerticalPlacement,
    ) -> Vec<(WindowId, CGRect)> {
        LayoutCore::calculate_layout(
            self,
            layout,
            screen,
            stack_offset,
            gaps,
            stack_line_thickness,
            stack_line_horiz,
            stack_line_vert,
        )
    }

    fn selected_window(&self, layout: LayoutId) -> Option<WindowId> {
        LayoutCore::selected_window(self, layout)
    }
    fn visible_windows_in_layout(&self, layout: LayoutId) -> Vec<WindowId> {
        LayoutCore::visible_windows_in_layout(self, layout)
    }
    fn visible_windows_under_selection(&self, layout: LayoutId) -> Vec<WindowId> {
        LayoutCore::visible_windows_under_selection(self, layout)
    }
    fn ascend_selection(&mut self, layout: LayoutId) -> bool {
        LayoutNavigation::ascend_selection(self, layout)
    }
    fn descend_selection(&mut self, layout: LayoutId) -> bool {
        LayoutNavigation::descend_selection(self, layout)
    }
    fn move_focus(
        &mut self,
        layout: LayoutId,
        direction: Direction,
    ) -> (Option<WindowId>, Vec<WindowId>) {
        LayoutNavigation::move_focus(self, layout, direction)
    }
    fn window_in_direction(&self, layout: LayoutId, direction: Direction) -> Option<WindowId> {
        LayoutNavigation::window_in_direction(self, layout, direction)
    }
    fn add_window_after_selection(&mut self, layout: LayoutId, wid: WindowId) {
        LayoutCore::add_window_after_selection(self, layout, wid)
    }
    fn remove_window(&mut self, wid: WindowId) {
        LayoutCore::remove_window(self, wid)
    }
    fn remove_windows_for_app(&mut self, pid: pid_t) {
        LayoutCore::remove_windows_for_app(self, pid)
    }
    fn set_windows_for_app(&mut self, layout: LayoutId, pid: pid_t, desired: Vec<WindowId>) {
        LayoutCore::set_windows_for_app(self, layout, pid, desired)
    }
    fn has_windows_for_app(&self, layout: LayoutId, pid: pid_t) -> bool {
        LayoutCore::has_windows_for_app(self, layout, pid)
    }
    fn contains_window(&self, layout: LayoutId, wid: WindowId) -> bool {
        LayoutCore::contains_window(self, layout, wid)
    }
    fn select_window(&mut self, layout: LayoutId, wid: WindowId) -> bool {
        LayoutCore::select_window(self, layout, wid)
    }
    fn on_window_resized(
        &mut self,
        layout: LayoutId,
        wid: WindowId,
        old_frame: CGRect,
        new_frame: CGRect,
        screen: CGRect,
        gaps: &crate::common::config::GapSettings,
    ) {
        LayoutCore::on_window_resized(self, layout, wid, old_frame, new_frame, screen, gaps)
    }

    fn swap_windows(&mut self, layout: LayoutId, a: WindowId, b: WindowId) -> bool {
        LayoutSwappable::swap_windows(self, layout, a, b)
    }

    fn move_selection(&mut self, layout: LayoutId, direction: Direction) -> bool {
        LayoutNavigation::move_selection(self, layout, direction)
    }
    fn move_selection_to_layout_after_selection(
        &mut self,
        from_layout: LayoutId,
        to_layout: LayoutId,
    ) {
        LayoutNavigation::move_selection_to_layout_after_selection(self, from_layout, to_layout)
    }
    fn split_selection(&mut self, layout: LayoutId, kind: LayoutKind) {
        LayoutSplittable::split_selection(self, layout, kind)
    }

    fn toggle_fullscreen_of_selection(&mut self, layout: LayoutId) -> Vec<WindowId> {
        LayoutFullscreen::toggle_fullscreen_of_selection(self, layout)
    }
    fn toggle_fullscreen_within_gaps_of_selection(&mut self, layout: LayoutId) -> Vec<WindowId> {
        LayoutFullscreen::toggle_fullscreen_within_gaps_of_selection(self, layout)
    }

    fn join_selection_with_direction(&mut self, layout: LayoutId, direction: Direction) {
        LayoutSplittable::join_selection_with_direction(self, layout, direction)
    }
    fn apply_stacking_to_parent_of_selection(
        &mut self,
        layout: LayoutId,
        default_orientation: crate::common::config::StackDefaultOrientation,
    ) -> Vec<WindowId> {
        LayoutStacking::apply_stacking_to_parent_of_selection(self, layout, default_orientation)
    }
    fn unstack_parent_of_selection(
        &mut self,
        layout: LayoutId,
        default_orientation: crate::common::config::StackDefaultOrientation,
    ) -> Vec<WindowId> {
        LayoutStacking::unstack_parent_of_selection(self, layout, default_orientation)
    }
    fn parent_of_selection_is_stacked(&self, layout: LayoutId) -> bool {
        LayoutStacking::parent_of_selection_is_stacked(self, layout)
    }
    fn unjoin_selection(&mut self, layout: LayoutId) {
        LayoutSplittable::unjoin_selection(self, layout)
    }
    fn resize_selection_by(&mut self, layout: LayoutId, amount: f64) {
        LayoutResizable::resize_selection_by(self, layout, amount)
    }
    fn rebalance(&mut self, layout: LayoutId) {
        LayoutResizable::rebalance(self, layout)
    }
    fn toggle_tile_orientation(&mut self, layout: LayoutId) {
        LayoutSplittable::toggle_tile_orientation(self, layout)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[enum_dispatch(LayoutSystem)]
pub enum LayoutSystemKind {
    Traditional(TraditionalLayoutSystem),
    Bsp(BspLayoutSystem),
    Dwindle(DwindleLayoutSystem),
}

mod bsp;
mod dwindle;
mod traditional;

pub use bsp::BspLayoutSystem;
pub use dwindle::DwindleLayoutSystem;
pub use traditional::TraditionalLayoutSystem;

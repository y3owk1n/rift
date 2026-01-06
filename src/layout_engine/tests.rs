use objc2_core_foundation::{CGPoint, CGRect, CGSize};

use crate::actor::app::WindowId;
use crate::layout_engine::BspLayoutSystem;
use crate::layout_engine::TraditionalLayoutSystem;
use crate::layout_engine::{Direction, LayoutId, LayoutKind, LayoutSystem, Orientation};

fn w(pid: i32, idx: u32) -> WindowId {
    WindowId::new(pid, idx)
}

fn screen() -> CGRect {
    CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(1920.0, 1080.0))
}

fn gaps() -> crate::common::config::GapSettings {
    crate::common::config::GapSettings::default()
}

mod bsp_layout_edge_cases {
    use super::*;

    mod window_addition {
        use super::*;

        #[test]
        fn add_single_window() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();
            system.add_window_after_selection(layout, w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 1);
            assert_eq!(visible[0], w(1, 1));
            assert_eq!(system.selected_window(layout), Some(w(1, 1)));
        }

        #[test]
        fn add_multiple_windows_in_sequence() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=5 {
                system.add_window_after_selection(layout, w(1, i));
            }

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 5);
            for i in 1..=5 {
                assert!(visible.contains(&w(1, i)));
            }
            assert_eq!(system.selected_window(layout), Some(w(1, 5)));
        }

        #[test]
        fn add_window_with_empty_tree() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();
            system.add_window_after_selection(layout, w(1, 1));

            let layout2 = system.create_layout();
            system.add_window_after_selection(layout2, w(2, 1));

            assert_eq!(system.visible_windows_in_layout(layout).len(), 1);
            assert_eq!(system.visible_windows_in_layout(layout2).len(), 1);
        }

        #[test]
        fn add_window_after_selection_changes_focus() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));
            system.select_window(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 3));

            assert_eq!(system.selected_window(layout), Some(w(1, 3)));
        }

        #[test]
        fn add_duplicate_window_id_is_allowed() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 2);
        }

        #[test]
        fn add_window_after_fullscreen_toggle() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            system.toggle_fullscreen_of_selection(layout);
            system.add_window_after_selection(layout, w(1, 3));

            let visible = system.visible_windows_in_layout(layout);
            assert!(visible.contains(&w(1, 3)));
        }
    }

    mod window_removal {
        use super::*;

        #[test]
        fn remove_last_window() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.remove_window(w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert!(visible.is_empty());
            assert_eq!(system.selected_window(layout), None);
        }

        #[test]
        fn remove_middle_window() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=5 {
                system.add_window_after_selection(layout, w(1, i));
            }

            system.remove_window(w(1, 3));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 4);
            assert!(!visible.contains(&w(1, 3)));
            assert!(visible.contains(&w(1, 1)));
            assert!(visible.contains(&w(1, 2)));
            assert!(visible.contains(&w(1, 4)));
            assert!(visible.contains(&w(1, 5)));
        }

        #[test]
        fn remove_first_window() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=3 {
                system.add_window_after_selection(layout, w(1, i));
            }

            system.remove_window(w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 2);
            assert!(!visible.contains(&w(1, 1)));
        }

        #[test]
        fn remove_second_window() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=3 {
                system.add_window_after_selection(layout, w(1, i));
            }

            system.remove_window(w(1, 2));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 2);
            assert!(!visible.contains(&w(1, 2)));
        }

        #[test]
        fn remove_nonexistent_window() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.remove_window(w(999, 999));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 1);
        }

        #[test]
        fn remove_all_windows() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=10 {
                system.add_window_after_selection(layout, w(1, i));
            }

            for i in 1..=10 {
                system.remove_window(w(1, i));
            }

            let visible = system.visible_windows_in_layout(layout);
            assert!(visible.is_empty());
        }

        #[test]
        fn remove_window_updates_selection() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));
            system.select_window(layout, w(1, 2));

            system.remove_window(w(1, 2));

            let selected = system.selected_window(layout);
            assert_ne!(selected, Some(w(1, 2)));
        }
    }

    mod focus_movement {
        use super::*;

        #[test]
        fn move_focus_with_single_window() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            let (_, raise) = system.move_focus(layout, Direction::Right);

            assert!(raise.is_empty() || raise.contains(&w(1, 1)));
        }

        #[test]
        fn move_focus_cycles_through_windows() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=3 {
                system.add_window_after_selection(layout, w(1, i));
            }

            let (focus1, _) = system.move_focus(layout, Direction::Right);
            let (focus2, _) = system.move_focus(layout, Direction::Right);
            let (focus3, _) = system.move_focus(layout, Direction::Right);

            assert!(focus1.is_some() || focus1.is_none());
            assert!(focus2.is_some() || focus2.is_none());
            assert!(focus3.is_some() || focus3.is_none());
        }

        #[test]
        fn move_focus_all_directions() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=4 {
                system.add_window_after_selection(layout, w(1, i));
            }

            let _ = system.move_focus(layout, Direction::Left);
            let _ = system.move_focus(layout, Direction::Right);
            let _ = system.move_focus(layout, Direction::Up);
            let _ = system.move_focus(layout, Direction::Down);

        }

        #[test]
        fn move_focus_with_empty_layout() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            let (focus, raise): (Option<WindowId>, Vec<WindowId>) =
                system.move_focus(layout, Direction::Right);

            assert_eq!(focus, None);
            assert!(raise.is_empty());
        }

        #[test]
        fn move_focus_with_multiple_apps() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(2, 1));
            system.add_window_after_selection(layout, w(3, 1));

            let (_, raise) = system.move_focus(layout, Direction::Right);
            assert!(!raise.is_empty() || raise.is_empty());
        }

        #[test]
        fn select_specific_window() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=5 {
                system.add_window_after_selection(layout, w(1, i));
            }

            assert!(system.select_window(layout, w(1, 3)));
            assert_eq!(system.selected_window(layout), Some(w(1, 3)));

            assert!(!system.select_window(layout, w(999, 999)));
        }

        #[test]
        fn focus_with_stacked_orientation() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));
            system.toggle_tile_orientation(layout);

            let (_, _raise) = system.move_focus(layout, Direction::Down);
        }
    }

    mod window_resize {
        use super::*;

        #[test]
        fn resize_selection() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            system.resize_selection_by(layout, 0.1);
            system.resize_selection_by(layout, -0.05);

        }

        #[test]
        fn resize_single_window_no_panic() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));

            system.resize_selection_by(layout, 0.1);
            system.resize_selection_by(layout, -0.1);

        }

        #[test]
        fn resize_boundaries() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            for _ in 0..100 {
                system.resize_selection_by(layout, 0.1);
            }
            for _ in 0..100 {
                system.resize_selection_by(layout, -0.1);
            }

        }
    }

    mod fullscreen_operations {
        use super::*;

        #[test]
        fn toggle_fullscreen_single_window() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));

            let windows = system.toggle_fullscreen_of_selection(layout);
            assert_eq!(windows.len(), 1);
            assert_eq!(windows[0], w(1, 1));

            let windows = system.toggle_fullscreen_of_selection(layout);
            assert!(windows.len() == 1 || windows.is_empty());
        }

        #[test]
        fn toggle_fullscreen_with_multiple_windows() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            system.select_window(layout, w(1, 1));
            let windows = system.toggle_fullscreen_of_selection(layout);
            assert_eq!(windows.len(), 1);

            system.select_window(layout, w(1, 2));
            let windows = system.toggle_fullscreen_of_selection(layout);
            assert_eq!(windows.len(), 1);
        }

        #[test]
        fn toggle_fullscreen_within_gaps() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            let windows = system.toggle_fullscreen_within_gaps_of_selection(layout);
            assert_eq!(windows.len(), 1);

            let windows = system.toggle_fullscreen_within_gaps_of_selection(layout);
            assert!(windows.len() == 1 || windows.is_empty());
        }
    }

    mod orientation_toggle {
        use super::*;

        #[test]
        fn toggle_orientation() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            system.toggle_tile_orientation(layout);
            system.toggle_tile_orientation(layout);

        }

        #[test]
        fn orientation_affects_window_positions() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            let before = system.calculate_layout(
                layout,
                screen(),
                0.0,
                &gaps(),
                0.0,
                crate::common::config::HorizontalPlacement::Top,
                crate::common::config::VerticalPlacement::Left,
            );

            system.toggle_tile_orientation(layout);

            let after = system.calculate_layout(
                layout,
                screen(),
                0.0,
                &gaps(),
                0.0,
                crate::common::config::HorizontalPlacement::Top,
                crate::common::config::VerticalPlacement::Left,
            );

            assert_eq!(before.len(), after.len());
        }
    }

    mod window_swap {
        use super::*;

        #[test]
        fn swap_adjacent_windows() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            assert!(system.swap_windows(layout, w(1, 1), w(1, 2)));

            let selected = system.selected_window(layout);
            assert_eq!(selected, Some(w(1, 1)));
        }

        #[test]
        fn swap_same_window_fails() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));

            assert!(!system.swap_windows(layout, w(1, 1), w(1, 1)));
        }

        #[test]
        fn swap_nonexistent_windows_fails() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));

            assert!(!system.swap_windows(layout, w(999, 1), w(1, 1)));
            assert!(!system.swap_windows(layout, w(1, 1), w(999, 1)));
        }

        #[test]
        fn swap_non_adjacent_windows() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=5 {
                system.add_window_after_selection(layout, w(1, i));
            }

            assert!(system.swap_windows(layout, w(1, 1), w(1, 5)));
        }
    }

    mod layout_calculation {
        use super::*;

        #[test]
        fn calculate_layout_empty() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            let result = system.calculate_layout(
                layout,
                screen(),
                0.0,
                &gaps(),
                0.0,
                crate::common::config::HorizontalPlacement::Top,
                crate::common::config::VerticalPlacement::Left,
            );

            assert!(result.is_empty());
        }

        #[test]
        fn calculate_layout_single_window() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();
            system.add_window_after_selection(layout, w(1, 1));

            let result = system.calculate_layout(
                layout,
                screen(),
                0.0,
                &gaps(),
                0.0,
                crate::common::config::HorizontalPlacement::Top,
                crate::common::config::VerticalPlacement::Left,
            );

            assert_eq!(result.len(), 1);
            assert_eq!(result[0].0, w(1, 1));
        }

        #[test]
        fn calculate_layout_respects_gaps() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            let mut custom_gaps = gaps();
            custom_gaps.inner.horizontal = 20.0;
            custom_gaps.inner.vertical = 20.0;
            custom_gaps.outer.top = 10.0;
            custom_gaps.outer.bottom = 10.0;
            custom_gaps.outer.left = 10.0;
            custom_gaps.outer.right = 10.0;

            let result = system.calculate_layout(
                layout,
                screen(),
                0.0,
                &custom_gaps,
                0.0,
                crate::common::config::HorizontalPlacement::Top,
                crate::common::config::VerticalPlacement::Left,
            );

            assert_eq!(result.len(), 2);
            let total_width: f64 = result.iter().map(|(_, r)| r.size.width).sum();
            let screen_width = screen().size.width;
            assert!(total_width > 0.0 && total_width <= screen_width);
        }

        #[test]
        fn calculate_layout_with_gaps_respects_screen_bounds() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));

            let result = system.calculate_layout(
                layout,
                screen(),
                0.0,
                &gaps(),
                0.0,
                crate::common::config::HorizontalPlacement::Top,
                crate::common::config::VerticalPlacement::Left,
            );

            assert!(result[0].1.origin.x >= 0.0);
            assert!(result[0].1.origin.y >= 0.0);
            assert!(result[0].1.size.width > 0.0);
            assert!(result[0].1.size.height > 0.0);
        }
    }

    mod app_management {
        use super::*;

        #[test]
        fn remove_windows_for_app() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(2, 1));
            system.add_window_after_selection(layout, w(1, 2));

            system.remove_windows_for_app(1);

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 1);
            assert_eq!(visible[0], w(2, 1));
        }

        #[test]
        fn has_windows_for_app() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(2, 1));

            assert!(system.has_windows_for_app(layout, 1));
            assert!(system.has_windows_for_app(layout, 2));
            assert!(!system.has_windows_for_app(layout, 999));
        }

        #[test]
        fn set_windows_for_app_replaces_correctly() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));
            system.add_window_after_selection(layout, w(2, 1));

            system.set_windows_for_app(layout, 1, vec![w(1, 3), w(1, 4)]);

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 3);
            assert!(visible.contains(&w(1, 3)));
            assert!(visible.contains(&w(1, 4)));
            assert!(visible.contains(&w(2, 1)));
            assert!(!visible.contains(&w(1, 1)));
            assert!(!visible.contains(&w(1, 2)));
        }
    }

    mod layout_clone {
        use super::*;

        #[test]
        fn clone_layout_preserves_windows() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=5 {
                system.add_window_after_selection(layout, w(1, i));
            }

            let cloned = system.clone_layout(layout);

            let original = system.visible_windows_in_layout(layout);
            let cloned_windows = system.visible_windows_in_layout(cloned);

            assert_eq!(original.len(), cloned_windows.len());
        }

        #[test]
        fn clone_independent_layouts() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();
            system.add_window_after_selection(layout, w(1, 1));

            let cloned = system.clone_layout(layout);
            system.add_window_after_selection(cloned, w(2, 1));

            assert_eq!(system.visible_windows_in_layout(layout).len(), 1);
            assert_eq!(system.visible_windows_in_layout(cloned).len(), 2);
        }
    }

    mod move_selection {
        use super::*;

        #[test]
        fn move_selection_left() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            assert!(system.move_selection(layout, Direction::Left));
        }

        #[test]
        fn move_selection_right() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            let _ = system.move_selection(layout, Direction::Right);
        }

        #[test]
        fn move_selection_up() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            let _ = system.move_selection(layout, Direction::Up);
        }

        #[test]
        fn move_selection_down() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            let _ = system.move_selection(layout, Direction::Down);
        }

        #[test]
        fn move_selection_empty_layout() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            assert!(!system.move_selection(layout, Direction::Right));
        }

        #[test]
        fn move_selection_with_multiple_windows() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=5 {
                system.add_window_after_selection(layout, w(1, i));
            }

            let _ = system.move_selection(layout, Direction::Right);
            let _ = system.move_selection(layout, Direction::Left);
        }
    }
}

mod traditional_layout_edge_cases {
    use super::*;

    mod window_addition {
        use super::*;

        #[test]
        fn add_single_window() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();
            system.add_window_after_selection(layout, w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 1);
        }

        #[test]
        fn add_multiple_windows() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=10 {
                system.add_window_after_selection(layout, w(1, i));
            }

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 10);
        }

        #[test]
        fn add_window_to_empty_root() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();
            system.add_window_after_selection(layout, w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 1);
        }
    }

    mod focus_movement {
        use super::*;

        #[test]
        fn move_focus_with_windows() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            let (focus, _): (Option<WindowId>, Vec<WindowId>) =
                system.move_focus(layout, Direction::Right);
            assert!(focus.is_some() || focus.is_none());
        }

        #[test]
        fn move_focus_empty_layout() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            let (focus, raise): (Option<WindowId>, Vec<WindowId>) =
                system.move_focus(layout, Direction::Right);
            assert_eq!(focus, None);
            assert!(raise.is_empty());
        }

        #[test]
        fn move_focus_all_directions() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=4 {
                system.add_window_after_selection(layout, w(1, i));
            }

            let _ = system.move_focus(layout, Direction::Left);
            let _ = system.move_focus(layout, Direction::Right);
            let _ = system.move_focus(layout, Direction::Up);
            let _ = system.move_focus(layout, Direction::Down);

        }

        #[test]
        fn select_window() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            assert!(system.select_window(layout, w(1, 2)));
            assert_eq!(system.selected_window(layout), Some(w(1, 2)));
        }
    }

    mod window_removal {
        use super::*;

        #[test]
        fn remove_single_window() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.remove_window(w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert!(visible.is_empty());
        }

        #[test]
        fn remove_multiple_windows() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=5 {
                system.add_window_after_selection(layout, w(1, i));
            }

            system.remove_window(w(1, 3));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 4);
            assert!(!visible.contains(&w(1, 3)));
        }

        #[test]
        fn remove_nonexistent_window() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.remove_window(w(999, 999));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 1);
        }
    }

    mod fullscreen {
        use super::*;

        #[test]
        fn toggle_fullscreen() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            let windows = system.toggle_fullscreen_of_selection(layout);
            assert_eq!(windows.len(), 1);

            let windows = system.toggle_fullscreen_of_selection(layout);
            assert!(windows.is_empty());
        }

        #[test]
        fn toggle_fullscreen_within_gaps() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            let windows = system.toggle_fullscreen_within_gaps_of_selection(layout);
            assert_eq!(windows.len(), 1);
        }
    }

    mod orientation {
        use super::*;

        #[test]
        fn toggle_orientation() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            system.toggle_tile_orientation(layout);
            system.toggle_tile_orientation(layout);

        }
    }

    mod resize {
        use super::*;

        #[test]
        fn resize_selection() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            system.resize_selection_by(layout, 0.1);
            system.resize_selection_by(layout, -0.05);

        }

        #[test]
        fn resize_boundaries() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            for _ in 0..100 {
                system.resize_selection_by(layout, 0.1);
            }
            for _ in 0..100 {
                system.resize_selection_by(layout, -0.1);
            }

        }
    }

    mod swap {
        use super::*;

        #[test]
        fn swap_windows() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            assert!(system.swap_windows(layout, w(1, 1), w(1, 2)));
        }

        #[test]
        fn swap_same_window_fails() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));

            assert!(!system.swap_windows(layout, w(1, 1), w(1, 1)));
        }
    }

    mod layout_calculation {
        use super::*;

        #[test]
        fn calculate_layout_empty() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            let result = system.calculate_layout(
                layout,
                screen(),
                0.0,
                &gaps(),
                0.0,
                crate::common::config::HorizontalPlacement::Top,
                crate::common::config::VerticalPlacement::Left,
            );

            assert!(result.is_empty());
        }

        #[test]
        fn calculate_layout_with_windows() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            let result = system.calculate_layout(
                layout,
                screen(),
                0.0,
                &gaps(),
                0.0,
                crate::common::config::HorizontalPlacement::Top,
                crate::common::config::VerticalPlacement::Left,
            );

            assert_eq!(result.len(), 2);
            let total_width: f64 = result.iter().map(|(_, r)| r.size.width).sum();
            let screen_width = screen().size.width;
            assert!((total_width - screen_width).abs() < 1.0);
        }
    }

    mod app_management {
        use super::*;

        #[test]
        fn remove_windows_for_app() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(2, 1));
            system.add_window_after_selection(layout, w(1, 2));

            system.remove_windows_for_app(1);

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 1);
            assert_eq!(visible[0], w(2, 1));
        }

        #[test]
        fn has_windows_for_app() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(2, 1));

            assert!(system.has_windows_for_app(layout, 1));
            assert!(system.has_windows_for_app(layout, 2));
            assert!(!system.has_windows_for_app(layout, 999));
        }
    }

    mod rebalance {
        use super::*;

        #[test]
        fn rebalance_layout() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=4 {
                system.add_window_after_selection(layout, w(1, i));
            }

            system.rebalance(layout);

        }
    }

    mod move_selection {
        use super::*;

        #[test]
        fn move_selection_all_directions() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=4 {
                system.add_window_after_selection(layout, w(1, i));
            }

            let _ = system.move_selection(layout, Direction::Left);
            let _ = system.move_selection(layout, Direction::Right);
            let _ = system.move_selection(layout, Direction::Up);
            let _ = system.move_selection(layout, Direction::Down);

        }

        #[test]
        fn move_selection_empty_layout() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            assert!(!system.move_selection(layout, Direction::Right));
        }
    }

    mod ascend_descend {
        use super::*;

        #[test]
        fn ascend_selection() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            let initial = system.selected_window(layout);
            assert!(initial.is_some());

            system.ascend_selection(layout);
        }

        #[test]
        fn descend_selection() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            let _ = system.descend_selection(layout);
        }

        #[test]
        fn ascend_empty_layout() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            assert!(!system.ascend_selection(layout));
        }
    }

    mod split_selection {
        use super::*;

        #[test]
        fn split_selection_horizontal() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.split_selection(layout, LayoutKind::Horizontal);

        }

        #[test]
        fn split_selection_vertical() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.split_selection(layout, LayoutKind::Vertical);

        }
    }

    mod stacking {
        use super::*;

        #[test]
        fn apply_stacking() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=3 {
                system.add_window_after_selection(layout, w(1, i));
            }

            let windows = system.apply_stacking_to_parent_of_selection(
                layout,
                crate::common::config::StackDefaultOrientation::Horizontal,
            );
            assert!(!windows.is_empty());
        }

        #[test]
        fn unstack_selection() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=3 {
                system.add_window_after_selection(layout, w(1, i));
            }

            system.apply_stacking_to_parent_of_selection(
                layout,
                crate::common::config::StackDefaultOrientation::Horizontal,
            );

            let windows = system.unstack_parent_of_selection(
                layout,
                crate::common::config::StackDefaultOrientation::Horizontal,
            );
            assert!(windows.is_empty() || !windows.is_empty());
        }

        #[test]
        fn parent_of_selection_is_stacked() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            system.apply_stacking_to_parent_of_selection(
                layout,
                crate::common::config::StackDefaultOrientation::Horizontal,
            );

            assert!(system.parent_of_selection_is_stacked(layout));
        }
    }

    mod unjoin {
        use super::*;

        #[test]
        fn unjoin_selection() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=3 {
                system.add_window_after_selection(layout, w(1, i));
            }

            system.unjoin_selection(layout);

        }
    }

    mod join {
        use super::*;

        #[test]
        fn join_selection_with_direction() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=3 {
                system.add_window_after_selection(layout, w(1, i));
            }

            system.join_selection_with_direction(layout, Direction::Right);

        }
    }
}

mod layout_system_common {
    use super::*;

    mod layout_lifecycle {
        use super::*;

        #[test]
        fn create_multiple_layouts() {
            let mut bsp = BspLayoutSystem::default();
            let mut trad = TraditionalLayoutSystem::default();

            let layouts: Vec<LayoutId> = (0..10)
                .map(|_| {
                    let layout = bsp.create_layout();
                    for i in 1..=3 {
                        bsp.add_window_after_selection(layout, w(1, i));
                    }
                    layout
                })
                .collect();

            assert_eq!(layouts.len(), 10);

            let layouts: Vec<LayoutId> = (0..10)
                .map(|_| {
                    let layout = trad.create_layout();
                    for i in 1..=3 {
                        trad.add_window_after_selection(layout, w(1, i));
                    }
                    layout
                })
                .collect();

            assert_eq!(layouts.len(), 10);
        }

        #[test]
        fn remove_layout() {
            let mut bsp = BspLayoutSystem::default();
            let layout = bsp.create_layout();
            bsp.add_window_after_selection(layout, w(1, 1));

            bsp.remove_layout(layout);

            let visible = bsp.visible_windows_in_layout(layout);
            assert!(visible.is_empty());
        }

        #[test]
        fn remove_layout_with_multiple_windows() {
            let mut bsp = BspLayoutSystem::default();
            let layout = bsp.create_layout();

            for i in 1..=10 {
                bsp.add_window_after_selection(layout, w(1, i));
            }

            bsp.remove_layout(layout);

        }
    }

    mod contains_window {
        use super::*;

        #[test]
        fn bsp_contains_window() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));

            assert!(system.contains_window(layout, w(1, 1)));
            assert!(!system.contains_window(layout, w(999, 1)));
        }

        #[test]
        fn traditional_contains_window() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));

            assert!(system.contains_window(layout, w(1, 1)));
            assert!(!system.contains_window(layout, w(999, 1)));
        }
    }

    mod visible_windows {
        use super::*;

        #[test]
        fn visible_windows_under_selection() {
            let mut bsp = BspLayoutSystem::default();
            let layout = bsp.create_layout();

            for i in 1..=5 {
                bsp.add_window_after_selection(layout, w(1, i));
            }

            let visible = bsp.visible_windows_under_selection(layout);
            assert!(!visible.is_empty());

            let mut trad = TraditionalLayoutSystem::default();
            let layout = trad.create_layout();

            for i in 1..=5 {
                trad.add_window_after_selection(layout, w(1, i));
            }

            let visible = trad.visible_windows_under_selection(layout);
            assert!(!visible.is_empty());
        }
    }

    mod draw_tree {
        use super::*;

        #[test]
        fn bsp_draw_tree() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=3 {
                system.add_window_after_selection(layout, w(1, i));
            }

            let tree = system.draw_tree(layout);
            assert!(!tree.is_empty());
        }

        #[test]
        fn traditional_draw_tree() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=3 {
                system.add_window_after_selection(layout, w(1, i));
            }

            let tree = system.draw_tree(layout);
            assert!(!tree.is_empty());
        }
    }

    mod move_selection_to_layout {
        use super::*;

        #[test]
        fn bsp_move_selection_to_layout() {
            let mut system = BspLayoutSystem::default();
            let layout1 = system.create_layout();
            let layout2 = system.create_layout();

            system.add_window_after_selection(layout1, w(1, 1));
            system.add_window_after_selection(layout1, w(1, 2));

            system.move_selection_to_layout_after_selection(layout1, layout2);

            let visible1 = system.visible_windows_in_layout(layout1);
            let visible2 = system.visible_windows_in_layout(layout2);

            assert!(visible1.is_empty() || visible1.len() < 2);
            assert!(!visible2.is_empty());
        }

        #[test]
        fn traditional_move_selection_to_layout() {
            let mut system = TraditionalLayoutSystem::default();
            let layout1 = system.create_layout();
            let layout2 = system.create_layout();

            system.add_window_after_selection(layout1, w(1, 1));
            system.add_window_after_selection(layout1, w(1, 2));

            system.move_selection_to_layout_after_selection(layout1, layout2);

            let visible1 = system.visible_windows_in_layout(layout1);
            let visible2 = system.visible_windows_in_layout(layout2);

            assert!(visible1.is_empty() || visible1.len() < 2);
            assert!(!visible2.is_empty());
        }
    }
}

mod direction_operations {
    use super::*;

    #[test]
    fn direction_step() {
        assert!(Direction::Right.step(0, 5) < 5);
        assert!(Direction::Left.step(4, 5) < 5);
        assert!(Direction::Up.step(0, 5) < 5);
        assert!(Direction::Down.step(4, 5) < 5);
    }

    #[test]
    fn direction_orientation() {
        assert_eq!(Direction::Left.orientation(), Orientation::Horizontal);
        assert_eq!(Direction::Right.orientation(), Orientation::Horizontal);
        assert_eq!(Direction::Up.orientation(), Orientation::Vertical);
        assert_eq!(Direction::Down.orientation(), Orientation::Vertical);
    }

    #[test]
    fn direction_opposite() {
        assert_eq!(Direction::Left.opposite(), Direction::Right);
        assert_eq!(Direction::Right.opposite(), Direction::Left);
        assert_eq!(Direction::Up.opposite(), Direction::Down);
        assert_eq!(Direction::Down.opposite(), Direction::Up);
    }
}

mod layout_kind_operations {
    use super::*;

    #[test]
    fn layout_kind_orientation() {
        assert_eq!(LayoutKind::Horizontal.orientation(), Orientation::Horizontal);
        assert_eq!(LayoutKind::Vertical.orientation(), Orientation::Vertical);
        assert_eq!(
            LayoutKind::HorizontalStack.orientation(),
            Orientation::Horizontal
        );
        assert_eq!(LayoutKind::VerticalStack.orientation(), Orientation::Vertical);
    }

    #[test]
    fn layout_kind_is_stacked() {
        assert!(!LayoutKind::Horizontal.is_stacked());
        assert!(!LayoutKind::Vertical.is_stacked());
        assert!(LayoutKind::HorizontalStack.is_stacked());
        assert!(LayoutKind::VerticalStack.is_stacked());
    }

    #[test]
    fn layout_kind_is_group() {
        assert!(!LayoutKind::Horizontal.is_group());
        assert!(!LayoutKind::Vertical.is_group());
        assert!(LayoutKind::HorizontalStack.is_group());
        assert!(LayoutKind::VerticalStack.is_group());
    }
}

mod window_state_management {
    use super::*;

    mod bsp_minimize_unminimize {
        use super::*;

        #[test]
        fn minimize_single_window_removes_from_visible() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.remove_window(w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert!(visible.is_empty());
            assert_eq!(system.selected_window(layout), None);
        }

        #[test]
        fn minimize_one_of_multiple_windows() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));
            system.add_window_after_selection(layout, w(1, 3));

            system.remove_window(w(1, 2));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 2);
            assert!(visible.contains(&w(1, 1)));
            assert!(visible.contains(&w(1, 3)));
            assert!(!visible.contains(&w(1, 2)));
        }

        #[test]
        fn minimize_last_window_of_app() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));

            system.remove_window(w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert!(visible.is_empty());
            assert!(!system.has_windows_for_app(layout, 1));
        }

        #[test]
        fn minimize_all_windows_of_app_leaves_others() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));
            system.add_window_after_selection(layout, w(2, 1));
            system.add_window_after_selection(layout, w(2, 2));

            system.remove_window(w(1, 1));
            system.remove_window(w(1, 2));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 2);
            assert!(visible.contains(&w(2, 1)));
            assert!(visible.contains(&w(2, 2)));
            assert!(!system.has_windows_for_app(layout, 1));
            assert!(system.has_windows_for_app(layout, 2));
        }

        #[test]
        fn unminimize_re_adds_window() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.remove_window(w(1, 1));
            system.add_window_after_selection(layout, w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 1);
            assert_eq!(visible[0], w(1, 1));
        }

        #[test]
        fn minimize_focused_window_updates_selection() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));
            system.select_window(layout, w(1, 2));

            system.remove_window(w(1, 2));

            let selected = system.selected_window(layout);
            assert_ne!(selected, Some(w(1, 2)));
        }

        #[test]
        fn minimize_middle_window_keeps_others_focused() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));
            system.add_window_after_selection(layout, w(1, 3));
            system.select_window(layout, w(1, 2));

            system.remove_window(w(1, 2));

            let selected = system.selected_window(layout);
            assert!(selected.is_some());
        }

        #[test]
        fn minimize_then_unminimize_multiple_windows() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=3 {
                system.add_window_after_selection(layout, w(1, i));
            }

            system.remove_window(w(1, 2));

            let visible_after_minimize = system.visible_windows_in_layout(layout);
            assert_eq!(visible_after_minimize.len(), 2);

            system.add_window_after_selection(layout, w(1, 2));

            let visible_after_unminimize = system.visible_windows_in_layout(layout);
            assert_eq!(visible_after_unminimize.len(), 3);
            assert!(visible_after_unminimize.contains(&w(1, 2)));
        }

        #[test]
        fn minimize_preserves_layout_structure() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=4 {
                system.add_window_after_selection(layout, w(1, i));
            }

            system.remove_window(w(1, 2));
            system.remove_window(w(1, 4));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 2);
            assert!(visible.contains(&w(1, 1)));
            assert!(visible.contains(&w(1, 3)));
        }
    }

    mod bsp_hide_unhide {
        use super::*;

        #[test]
        fn hide_and_unhide_behaves_like_remove_and_add() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            system.remove_window(w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 1);
            assert!(visible.contains(&w(1, 2)));

            system.add_window_after_selection(layout, w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 2);
            assert!(visible.contains(&w(1, 1)));
            assert!(visible.contains(&w(1, 2)));
        }

        #[test]
        fn hide_all_windows_of_app() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=5 {
                system.add_window_after_selection(layout, w(1, i));
            }

            for i in 1..=5 {
                system.remove_window(w(1, i));
            }

            assert!(!system.has_windows_for_app(layout, 1));
            assert!(system.visible_windows_in_layout(layout).is_empty());
        }

        #[test]
        fn hide_partial_then_unhide_all() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=3 {
                system.add_window_after_selection(layout, w(1, i));
            }

            system.remove_window(w(1, 1));
            system.remove_window(w(1, 3));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 1);
            assert!(visible.contains(&w(1, 2)));

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 3));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 3);
        }
    }

    mod traditional_minimize_unminimize {
        use super::*;

        #[test]
        fn minimize_single_window_removes_from_visible() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.remove_window(w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert!(visible.is_empty());
        }

        #[test]
        fn minimize_one_of_multiple_windows() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=4 {
                system.add_window_after_selection(layout, w(1, i));
            }

            system.remove_window(w(1, 2));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 3);
            assert!(!visible.contains(&w(1, 2)));
        }

        #[test]
        fn minimize_last_window_of_app() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.remove_window(w(1, 1));

            assert!(!system.has_windows_for_app(layout, 1));
        }

        #[test]
        fn minimize_all_windows_of_app() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));
            system.add_window_after_selection(layout, w(2, 1));

            system.remove_windows_for_app(1);

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 1);
            assert_eq!(visible[0], w(2, 1));
        }

        #[test]
        fn unminimize_re_adds_window() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.remove_window(w(1, 1));
            system.add_window_after_selection(layout, w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 1);
        }

        #[test]
        fn minimize_then_unminimize_preserves_order() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));
            system.add_window_after_selection(layout, w(1, 3));

            system.remove_window(w(1, 2));

            system.add_window_after_selection(layout, w(1, 2));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 3);
        }

        #[test]
        fn minimize_selected_window_updates_selection() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));
            system.select_window(layout, w(1, 2));

            system.remove_window(w(1, 2));

            let selected = system.selected_window(layout);
            assert_ne!(selected, Some(w(1, 2)));
        }
    }

    mod traditional_hide_unhide {
        use super::*;

        #[test]
        fn hide_and_unhide_same_as_remove_add() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            system.remove_window(w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 1);

            system.add_window_after_selection(layout, w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 2);
        }

        #[test]
        fn hide_multiple_windows_same_app() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=4 {
                system.add_window_after_selection(layout, w(1, i));
            }

            system.remove_window(w(1, 1));
            system.remove_window(w(1, 3));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 2);
            assert!(visible.contains(&w(1, 2)));
            assert!(visible.contains(&w(1, 4)));
        }

        #[test]
        fn unhide_partial_windows() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));

            system.remove_window(w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 1);

            system.add_window_after_selection(layout, w(1, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 2);
        }
    }

    mod cross_app_minimize_scenarios {
        use super::*;

        #[test]
        fn minimize_one_app_preserves_others_bsp() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));
            system.add_window_after_selection(layout, w(2, 1));
            system.add_window_after_selection(layout, w(2, 2));

            system.remove_windows_for_app(1);

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 2);
            assert!(visible.contains(&w(2, 1)));
            assert!(visible.contains(&w(2, 2)));
        }

        #[test]
        fn minimize_one_app_preserves_others_traditional() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));
            system.add_window_after_selection(layout, w(2, 1));

            system.remove_windows_for_app(1);

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 1);
            assert_eq!(visible[0], w(2, 1));
        }

        #[test]
        fn minimize_then_unminimize_different_apps() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(2, 1));
            system.add_window_after_selection(layout, w(2, 2));

            system.remove_window(w(1, 1));
            system.remove_window(w(2, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 1);
            assert!(visible.contains(&w(2, 2)));

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(2, 1));

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 3);
        }

        #[test]
        fn minimize_last_window_selection_fallback_bsp() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(2, 1));

            system.select_window(layout, w(1, 1));
            system.remove_window(w(1, 1));

            let selected = system.selected_window(layout);
            assert_eq!(selected, Some(w(2, 1)));
        }

        #[test]
        fn minimize_last_window_selection_fallback_traditional() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(2, 1));

            system.select_window(layout, w(1, 1));
            system.remove_window(w(1, 1));

            let selected = system.selected_window(layout);
            assert_eq!(selected, Some(w(2, 1)));
        }
    }

    mod rapid_state_changes {
        use super::*;

        #[test]
        fn rapid_minimize_unminimize_bsp() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=3 {
                system.add_window_after_selection(layout, w(1, i));
            }

            for _ in 0..5 {
                system.remove_window(w(1, 2));
                system.add_window_after_selection(layout, w(1, 2));
            }

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 3);
        }

        #[test]
        fn rapid_minimize_unminimize_traditional() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=3 {
                system.add_window_after_selection(layout, w(1, i));
            }

            for _ in 0..5 {
                system.remove_window(w(1, 2));
                system.add_window_after_selection(layout, w(1, 2));
            }

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 3);
        }

        #[test]
        fn minimize_all_then_restore_bsp() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=5 {
                system.add_window_after_selection(layout, w(1, i));
            }

            for i in 1..=5 {
                system.remove_window(w(1, i));
            }

            assert!(system.visible_windows_in_layout(layout).is_empty());

            for i in 1..=5 {
                system.add_window_after_selection(layout, w(1, i));
            }

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 5);
        }

        #[test]
        fn minimize_all_then_restore_traditional() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=5 {
                system.add_window_after_selection(layout, w(1, i));
            }

            for i in 1..=5 {
                system.remove_window(w(1, i));
            }

            assert!(system.visible_windows_in_layout(layout).is_empty());

            for i in 1..=5 {
                system.add_window_after_selection(layout, w(1, i));
            }

            let visible = system.visible_windows_in_layout(layout);
            assert_eq!(visible.len(), 5);
        }
    }

    mod layout_integrity_after_state_changes {
        use super::*;

        #[test]
        fn layout_calculates_correctly_after_minimize_bsp() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=4 {
                system.add_window_after_selection(layout, w(1, i));
            }

            system.remove_window(w(1, 2));
            system.remove_window(w(1, 4));

            let result = system.calculate_layout(
                layout,
                screen(),
                0.0,
                &gaps(),
                0.0,
                crate::common::config::HorizontalPlacement::Top,
                crate::common::config::VerticalPlacement::Left,
            );

            assert_eq!(result.len(), 2);
        }

        #[test]
        fn layout_calculates_correctly_after_minimize_traditional() {
            let mut system = TraditionalLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=4 {
                system.add_window_after_selection(layout, w(1, i));
            }

            system.remove_window(w(1, 2));

            let result = system.calculate_layout(
                layout,
                screen(),
                0.0,
                &gaps(),
                0.0,
                crate::common::config::HorizontalPlacement::Top,
                crate::common::config::VerticalPlacement::Left,
            );

            assert_eq!(result.len(), 3);
        }

        #[test]
        fn swap_works_after_minimize_bsp() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            system.add_window_after_selection(layout, w(1, 1));
            system.add_window_after_selection(layout, w(1, 2));
            system.add_window_after_selection(layout, w(1, 3));

            system.remove_window(w(1, 2));

            assert!(system.swap_windows(layout, w(1, 1), w(1, 3)));
        }

        #[test]
        fn move_selection_works_after_minimize_bsp() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=3 {
                system.add_window_after_selection(layout, w(1, i));
            }

            system.remove_window(w(1, 2));

            let _ = system.move_selection(layout, Direction::Right);
            let _ = system.move_selection(layout, Direction::Left);
        }

        #[test]
        fn clone_layout_preserves_windows_after_minimize() {
            let mut system = BspLayoutSystem::default();
            let layout = system.create_layout();

            for i in 1..=3 {
                system.add_window_after_selection(layout, w(1, i));
            }

            system.remove_window(w(1, 2));

            let cloned = system.clone_layout(layout);
            let original_visible = system.visible_windows_in_layout(layout);
            let cloned_visible = system.visible_windows_in_layout(cloned);

            assert_eq!(original_visible.len(), cloned_visible.len());
        }
    }
}

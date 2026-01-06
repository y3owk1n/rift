use serde::{Deserialize, Serialize};

use crate::actor::app::{WindowId, pid_t};
use crate::common::collections::{BTreeExt, BTreeSet, HashMap, HashSet};
use crate::sys::screen::SpaceId;

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct FloatingManager {
    floating_windows: BTreeSet<WindowId>,
    #[serde(skip)]
    active_floating_windows: HashMap<SpaceId, HashMap<pid_t, HashSet<WindowId>>>,
    last_floating_focus: Option<WindowId>,
}

impl FloatingManager {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn is_floating(&self, window_id: WindowId) -> bool {
        self.floating_windows.contains(&window_id)
    }

    pub(crate) fn add_floating(&mut self, window_id: WindowId) {
        self.floating_windows.insert(window_id);
    }

    pub(crate) fn remove_floating(&mut self, window_id: WindowId) {
        self.floating_windows.remove(&window_id);

        for space_floating in self.active_floating_windows.values_mut() {
            if let Some(app_set) = space_floating.get_mut(&window_id.pid) {
                app_set.remove(&window_id);
                if app_set.is_empty() {
                    space_floating.remove(&window_id.pid);
                }
            }
        }
        if self.last_floating_focus == Some(window_id) {
            self.last_floating_focus = None;
        }
    }

    pub(crate) fn clear_active_for_app(&mut self, space: SpaceId, pid: pid_t) {
        if let Some(space_map) = self.active_floating_windows.get_mut(&space) {
            space_map.remove(&pid);
        }
    }

    pub(crate) fn add_active(&mut self, space: SpaceId, pid: pid_t, wid: WindowId) {
        self.active_floating_windows
            .entry(space)
            .or_default()
            .entry(pid)
            .or_default()
            .insert(wid);
    }

    pub(crate) fn remove_active(&mut self, space: SpaceId, pid: pid_t, wid: WindowId) {
        if let Some(space_map) = self.active_floating_windows.get_mut(&space)
            && let Some(app_set) = space_map.get_mut(&pid)
        {
            app_set.remove(&wid);
            if app_set.is_empty() {
                space_map.remove(&pid);
            }
        }
    }

    pub(crate) fn active_flat(&self, space: SpaceId) -> Vec<WindowId> {
        self.active_floating_windows
            .get(&space)
            .map(|space_floating| space_floating.values().flatten().copied().collect())
            .unwrap_or_default()
    }

    pub(crate) fn set_last_focus(&mut self, wid: Option<WindowId>) {
        self.last_floating_focus = wid;
    }

    pub(crate) fn last_focus(&self) -> Option<WindowId> {
        self.last_floating_focus
    }

    pub(crate) fn remove_all_for_pid(&mut self, pid: pid_t) {
        let _ = self.floating_windows.remove_all_for_pid(pid);

        for space_map in self.active_floating_windows.values_mut() {
            space_map.remove(&pid);
        }

        if let Some(focus) = self.last_floating_focus
            && focus.pid == pid
        {
            self.last_floating_focus = None;
        }
    }

    pub(crate) fn rebuild_active_for_workspace(
        &mut self,
        space: SpaceId,
        windows_in_workspace: Vec<WindowId>,
    ) {
        let space_map = self.active_floating_windows.entry(space).or_default();
        space_map.clear();
        for wid in windows_in_workspace.into_iter().filter(|&w| self.floating_windows.contains(&w))
        {
            space_map.entry(wid.pid).or_default().insert(wid);
        }
    }

    pub(crate) fn remap_space(&mut self, old_space: SpaceId, new_space: SpaceId) {
        if old_space == new_space {
            return;
        }

        let mut merged = self.active_floating_windows.remove(&new_space).unwrap_or_default();

        if let Some(old) = self.active_floating_windows.remove(&old_space) {
            for (pid, windows) in old {
                merged.entry(pid).or_default().extend(windows);
            }
        }

        if !merged.is_empty() {
            self.active_floating_windows.insert(new_space, merged);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(pid: i32, idx: u32) -> WindowId {
        WindowId::new(pid, idx)
    }

    #[test]
    fn test_floating_manager_is_floating() {
        let mut manager = FloatingManager::new();
        assert!(!manager.is_floating(w(1, 1)));

        manager.add_floating(w(1, 1));
        assert!(manager.is_floating(w(1, 1)));
        assert!(!manager.is_floating(w(1, 2)));
    }

    #[test]
    fn test_floating_manager_add_remove() {
        let mut manager = FloatingManager::new();

        manager.add_floating(w(1, 1));
        manager.add_floating(w(1, 2));
        assert_eq!(manager.floating_windows.len(), 2);

        manager.remove_floating(w(1, 1));
        assert!(!manager.is_floating(w(1, 1)));
        assert!(manager.is_floating(w(1, 2)));
    }

    #[test]
    fn test_floating_manager_active() {
        let mut manager = FloatingManager::new();
        let space = SpaceId::new(1);

        manager.add_floating(w(1, 1));
        manager.add_active(space, 1, w(1, 1));

        let active = manager.active_flat(space);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0], w(1, 1));
    }

    #[test]
    fn test_floating_manager_clear_active_for_app() {
        let mut manager = FloatingManager::new();
        let space = SpaceId::new(1);

        manager.add_floating(w(1, 1));
        manager.add_floating(w(1, 2));
        manager.add_active(space, 1, w(1, 1));
        manager.add_active(space, 1, w(1, 2));

        manager.clear_active_for_app(space, 1);
        let active = manager.active_flat(space);
        assert!(active.is_empty());
    }

    #[test]
    fn test_floating_manager_remove_active() {
        let mut manager = FloatingManager::new();
        let space = SpaceId::new(1);

        manager.add_floating(w(1, 1));
        manager.add_floating(w(1, 2));
        manager.add_active(space, 1, w(1, 1));
        manager.add_active(space, 1, w(1, 2));

        manager.remove_active(space, 1, w(1, 1));
        let active = manager.active_flat(space);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0], w(1, 2));
    }

    #[test]
    fn test_floating_manager_last_focus() {
        let mut manager = FloatingManager::new();

        assert_eq!(manager.last_focus(), None);

        manager.set_last_focus(Some(w(1, 1)));
        assert_eq!(manager.last_focus(), Some(w(1, 1)));

        manager.set_last_focus(None);
        assert_eq!(manager.last_focus(), None);
    }

    #[test]
    fn test_floating_manager_remove_all_for_pid() {
        let mut manager = FloatingManager::new();
        let space = SpaceId::new(1);

        manager.add_floating(w(1, 1));
        manager.add_floating(w(1, 2));
        manager.add_floating(w(2, 1));
        manager.add_active(space, 1, w(1, 1));
        manager.add_active(space, 1, w(1, 2));
        manager.add_active(space, 2, w(2, 1));
        manager.set_last_focus(Some(w(1, 1)));

        manager.remove_all_for_pid(1);

        assert!(!manager.is_floating(w(1, 1)));
        assert!(!manager.is_floating(w(1, 2)));
        assert!(manager.is_floating(w(2, 1)));

        let active = manager.active_flat(space);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0], w(2, 1));

        assert_eq!(manager.last_focus(), None);
    }

    #[test]
    fn test_floating_manager_rebuild_active_for_workspace() {
        let mut manager = FloatingManager::new();
        let space = SpaceId::new(1);

        manager.add_floating(w(1, 1));
        manager.add_floating(w(1, 2));
        manager.add_floating(w(1, 3));

        manager.rebuild_active_for_workspace(space, vec![w(1, 1), w(1, 2), w(1, 4)]);

        let active = manager.active_flat(space);
        assert_eq!(active.len(), 2);
        assert!(active.contains(&w(1, 1)));
        assert!(active.contains(&w(1, 2)));
        assert!(!active.contains(&w(1, 3)));
        assert!(!active.contains(&w(1, 4)));
    }

    #[test]
    fn test_floating_manager_remap_space() {
        let mut manager = FloatingManager::new();
        let space1 = SpaceId::new(1);
        let space2 = SpaceId::new(2);

        manager.add_floating(w(1, 1));
        manager.add_active(space1, 1, w(1, 1));

        manager.remap_space(space1, space2);

        let active1 = manager.active_flat(space1);
        let active2 = manager.active_flat(space2);

        assert!(active1.is_empty());
        assert_eq!(active2.len(), 1);
        assert_eq!(active2[0], w(1, 1));
    }

    #[test]
    fn test_floating_manager_remap_same_space() {
        let mut manager = FloatingManager::new();
        let space = SpaceId::new(1);

        manager.add_floating(w(1, 1));
        manager.add_active(space, 1, w(1, 1));

        manager.remap_space(space, space);

        let active = manager.active_flat(space);
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn test_floating_manager_multiple_spaces() {
        let mut manager = FloatingManager::new();
        let space1 = SpaceId::new(1);
        let space2 = SpaceId::new(2);

        manager.add_floating(w(1, 1));
        manager.add_floating(w(1, 2));
        manager.add_active(space1, 1, w(1, 1));
        manager.add_active(space2, 1, w(1, 2));

        let active1 = manager.active_flat(space1);
        let active2 = manager.active_flat(space2);

        assert_eq!(active1.len(), 1);
        assert_eq!(active2.len(), 1);
        assert_eq!(active1[0], w(1, 1));
        assert_eq!(active2[0], w(1, 2));
    }
}

use dispatchr::queue;
use dispatchr::time::Time;
use std::num::NonZeroU32;
use tracing::{debug, warn};

use crate::actor::app::{pid_t, WindowId};
use crate::actor::raise_manager;
use crate::actor::reactor::{MenuState, Reactor};
use crate::actor::wm_controller::Sender as WmSender;
use crate::sys::dispatch::DispatchExt;
use crate::sys::window_server::WindowServerInfo;

pub struct SystemEventHandler;

impl SystemEventHandler {
    pub fn handle_menu_opened(reactor: &mut Reactor) {
        debug!("menu opened");
        reactor.menu_manager.menu_state = match reactor.menu_manager.menu_state {
            MenuState::Closed => MenuState::Open(1),
            MenuState::Open(depth) => MenuState::Open(depth.saturating_add(1)),
        };
        reactor.update_focus_follows_mouse_state();
    }

    pub fn handle_menu_closed(reactor: &mut Reactor) {
        match reactor.menu_manager.menu_state {
            MenuState::Closed => {
                debug!("menu closed with zero depth");
            }
            MenuState::Open(depth) => {
                let new_depth = depth.saturating_sub(1);
                reactor.menu_manager.menu_state = if new_depth == 0 {
                    MenuState::Closed
                } else {
                    MenuState::Open(new_depth)
                };
                reactor.update_focus_follows_mouse_state();
            }
        }
    }

    pub fn handle_system_woke(reactor: &mut Reactor) {
        debug!("[WAKE] system woke from sleep");

        let reactor_ptr = reactor as *mut Reactor;
        queue::main().after_f_s(
            Time::new_after(Time::NOW, 50_000_000),
            (reactor_ptr,),
            |(reactor_ptr,)| unsafe {
                let reactor = &mut *reactor_ptr;
                debug!("[WAKE] performing post-wake recovery");

                let ws_info = crate::sys::window_server::get_visible_windows_with_layer(None);
                debug!("[WAKE] visible windows from server: {}", ws_info.len());

                if !ws_info.is_empty() {
                    reactor.update_complete_window_server_info(ws_info.clone());
                }

                Self::discover_all_windows_from_server(reactor, &ws_info);

                Self::reassign_all_windows_to_workspaces(reactor);

                if let Err(e) = reactor.update_layout(false, false) {
                    warn!("layout update failed during wake recovery: {}", e);
                }

                reactor.maybe_send_menu_update();

                let ids: Vec<u32> =
                    reactor.window_manager.window_ids.keys().map(|wsid| wsid.as_u32()).collect();
                debug!("[WAKE] final window ids count: {}", ids.len());
                crate::sys::window_notify::update_window_notifications(&ids);
                reactor.notification_manager.last_sls_notification_ids = ids;

                // Emit layout events for all PIDs with discovered windows to ensure
                // windows are properly tiled even if apps haven't reported them yet.
                let pids: std::collections::HashSet<pid_t> = reactor
                    .window_manager
                    .window_ids
                    .values()
                    .map(|wid| wid.pid)
                    .collect();
                debug!("[WAKE] emitting layout events for {} PIDs", pids.len());
                for &pid in &pids {
                    crate::actor::reactor::events::window_discovery::WindowDiscoveryHandler::emit_layout_events(
                        reactor,
                        pid,
                        &Vec::new(),
                        &None,
                    );
                }

                debug!("[WAKE] wake recovery complete");
            },
        );
    }

    fn discover_all_windows_from_server(reactor: &mut Reactor, ws_info: &[WindowServerInfo]) {
        debug!("[WAKE] discovering all windows from window server");

        let mut discovered_count = 0;
        let window_manager = &mut reactor.window_manager;

        for info in ws_info {
            let wsid = info.id;

            if window_manager.window_ids.contains_key(&wsid) {
                continue;
            }

            let idx = NonZeroU32::new(wsid.as_u32()).expect("Window server id was 0");
            let wid = WindowId { pid: info.pid, idx };

            debug!(
                "[WAKE] discovered new window: {:?} (wsid={:?}, pid={})",
                wid, wsid, info.pid
            );

            window_manager.window_ids.insert(wsid, wid);
            discovered_count += 1;
        }

        debug!("[WAKE] discovered {} new windows from server", discovered_count);
    }

    fn reassign_all_windows_to_workspaces(reactor: &mut Reactor) {
        debug!("[WAKE] reassigning all windows to workspaces");

        let vwm = reactor.layout_manager.layout_engine.virtual_workspace_manager_mut();
        let mut reassigned_count = 0;
        let mut skipped_no_info_count = 0;

        for (&wsid, &wid) in &reactor.window_manager.window_ids {
            let server_id = wsid;
            let space = match crate::sys::window_server::window_space(server_id) {
                Some(s) => s,
                None => continue,
            };

            let window_info = reactor.window_manager.windows.get(&wid);
            let app_info = reactor.app_manager.apps.get(&wid.pid).map(|a| a.info.clone());

            if window_info.is_none() {
                debug!(
                    "[WAKE] skipping window {:?} - no WindowInfo yet, will assign when app reports it",
                    wid
                );
                skipped_no_info_count += 1;
                continue;
            }

            let result = vwm.assign_window_with_app_info(
                wid,
                space,
                app_info.as_ref().and_then(|a| a.bundle_id.as_deref()),
                app_info.as_ref().and_then(|a| a.localized_name.as_deref()),
                window_info.and_then(|w| Some(w.title.as_str())),
                window_info.and_then(|w| w.ax_role.as_deref()),
                window_info.and_then(|w| w.ax_subrole.as_deref()),
            );

            if result.is_ok() {
                reassigned_count += 1;
            }
        }

        debug!(
            "[WAKE] reassigned {} windows to workspaces, skipped {} (waiting for WindowInfo)",
            reassigned_count, skipped_no_info_count
        );
    }

    pub fn handle_raise_completed(reactor: &mut Reactor, window_id: WindowId, sequence_id: u64) {
        let msg = raise_manager::Event::RaiseCompleted { window_id, sequence_id };
        _ = reactor.communication_manager.raise_manager_tx.send(msg);
    }

    pub fn handle_raise_timeout(reactor: &mut Reactor, sequence_id: u64) {
        let msg = raise_manager::Event::RaiseTimeout { sequence_id };
        _ = reactor.communication_manager.raise_manager_tx.send(msg);
    }

    pub fn handle_register_wm_sender(reactor: &mut Reactor, sender: WmSender) {
        reactor.communication_manager.wm_sender = Some(sender);
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use crate::actor::reactor::{Reactor, SystemEventHandler};
    use crate::layout_engine::LayoutEngine;

    #[test]
    fn test_system_event_handler_exists() {
        let _handler = SystemEventHandler;
    }

    #[test]
    fn test_handle_system_woke_exists() {
        let mut reactor = Reactor::new_for_test(LayoutEngine::new(
            &crate::common::config::VirtualWorkspaceSettings::default(),
            &crate::common::config::LayoutSettings::default(),
            None,
        ));

        SystemEventHandler::handle_system_woke(&mut reactor);
    }
}

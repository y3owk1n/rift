use tracing::{debug, trace, warn};

use crate::actor::app::{AppInfo, AppThreadHandle, Quiet, WindowId};
use crate::actor::reactor::{AppState, Event, Reactor};
use crate::sys::app::WindowInfo;
use crate::sys::window_server::{self as window_server, WindowServerId, WindowServerInfo};

#[allow(clippy::too_many_arguments)]
pub struct AppEventHandler;

impl AppEventHandler {
#[allow(clippy::too_many_arguments)]
    pub fn handle_application_launched(
        reactor: &mut Reactor,
        pid: i32,
        info: AppInfo,
        handle: AppThreadHandle,
        visible_windows: Vec<(WindowId, WindowInfo)>,
        window_server_info: Vec<WindowServerInfo>,
        _is_frontmost: bool,
        _main_window: Option<WindowId>,
    ) {
        reactor.app_manager.apps.insert(pid, AppState { info: info.clone(), handle });
        reactor.update_partial_window_server_info(window_server_info);
        reactor.on_windows_discovered_with_app_info(pid, visible_windows, vec![], Some(info));
    }

    pub fn handle_apply_app_rules_to_existing_windows(
        reactor: &mut Reactor,
        pid: i32,
        app_info: AppInfo,
        windows: Vec<WindowServerInfo>,
    ) {
        reactor.update_partial_window_server_info(windows.clone());

        let all_windows: Vec<WindowId> = windows
            .iter()
            .filter_map(|info| reactor.window_manager.window_ids.get(&info.id).copied())
            .filter(|wid| {
                reactor
                    .window_manager
                    .windows
                    .get(wid)
                    .is_some_and(|window| window.is_manageable)
            })
            .collect();

        if !all_windows.is_empty() {
            let wsids: Vec<WindowServerId> = windows.iter().map(|w| w.id).collect();
            reactor.app_manager.mark_wsids_recent(wsids);
            reactor.process_windows_for_app_rules(pid, all_windows, app_info);
        }
    }

    pub fn handle_application_terminated(reactor: &mut Reactor, pid: i32) {
        if let Some(app) = reactor.app_manager.apps.get_mut(&pid)
            && let Err(e) = app.handle.send(crate::actor::app::Request::Terminate) {
                warn!("Failed to send Terminate to app {}: {}", pid, e);
            }
    }

    pub fn handle_application_thread_terminated(reactor: &mut Reactor, pid: i32) {
        if let Some(wm) = reactor.communication_manager.wm_sender.as_ref() {
            wm.send(crate::actor::wm_controller::WmEvent::AppThreadTerminated(pid));
        }

        let windows_to_remove: Vec<WindowId> = reactor
            .window_manager
            .windows
            .iter()
            .filter_map(|(&wid, _)| if wid.pid == pid { Some(wid) } else { None })
            .collect();

        for wid in &windows_to_remove {
            reactor.handle_event(Event::WindowDestroyed(*wid));
        }

        reactor.app_manager.apps.remove(&pid);
    }

    pub fn handle_resync_app_for_window(reactor: &mut Reactor, wsid: WindowServerId) {
        if let Some(&wid) = reactor.window_manager.window_ids.get(&wsid) {
            if let Some(app_state) = reactor.app_manager.apps.get(&wid.pid)
                && let Err(e) = app_state
                    .handle
                    .send(crate::actor::app::Request::GetVisibleWindows { force_refresh: true })
                {
                    warn!("Failed to send GetVisibleWindows to app {}: {}", wid.pid, e);
                }
        } else if let Some(info) = reactor
            .window_server_info_manager
            .window_server_info
            .get(&wsid)
            .cloned()
            .or_else(|| window_server::get_window(wsid))
            && let Some(app_state) = reactor.app_manager.apps.get(&info.pid)
                && let Err(e) = app_state
                    .handle
                    .send(crate::actor::app::Request::GetVisibleWindows { force_refresh: true })
                {
                    warn!("Failed to send GetVisibleWindows to app {}: {}", info.pid, e);
                }
    }

    pub fn handle_application_activated(reactor: &mut Reactor, pid: i32, quiet: Quiet) {
        if quiet == Quiet::Yes {
            debug!(
                pid,
                "Skipping auto workspace switch for quiet app activation (initiated by Rift)"
            );
            return;
        }

        let now = std::time::Instant::now();
        if let Some(last_time) = reactor.last_activation_time {
            let cooldown = std::time::Duration::from_millis(50);
            if now.duration_since(last_time) < cooldown {
                trace!(
                    pid,
                    "Skipping duplicate application activation within cooldown period"
                );
                return;
            }
        }
        reactor.last_activation_time = Some(now);

        reactor.handle_app_activation_workspace_switch(pid);
    }

    pub fn handle_windows_discovered(
        reactor: &mut Reactor,
        pid: i32,
        new: Vec<(WindowId, WindowInfo)>,
        known_visible: Vec<WindowId>,
    ) {
        reactor.on_windows_discovered_with_app_info(pid, new, known_visible, None);
    }
}

use log::{debug, error, info};
use std::io;
use std::ptr;
use std::sync::Weak;

use crate::app_state::AppState;
use std::sync::Mutex;

#[cfg(target_os = "windows")]
use winapi::shared::minwindef::LPVOID;
#[cfg(target_os = "windows")]
use winapi::um::winbase::{INFINITE, RegisterWaitForSingleObject, UnregisterWait};
#[cfg(target_os = "windows")]
use winapi::um::winnt::{HANDLE, WT_EXECUTEONLYONCE};

/// Events related to process lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessEvent {
    /// Process has started
    Started,
    /// Process has exited
    Exited,
    /// Process state changed (external detection)
    StateChanged,
    /// Error monitoring the process
    Error,
}

/// Handle for registered process monitors
pub struct ProcessMonitorHandle {
    #[cfg(target_os = "windows")]
    wait_handle: HANDLE,
}

impl Drop for ProcessMonitorHandle {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        unsafe {
            if !self.wait_handle.is_null() {
                UnregisterWait(self.wait_handle);
            }
        }
    }
}

/// Register a callback to be executed when a process exits
#[cfg(target_os = "windows")]
pub fn register_process_exit_monitor(
    process_handle: HANDLE,
    app_state: Weak<Mutex<AppState>>,
    pid: u32,
) -> io::Result<ProcessMonitorHandle> {
    debug!("Registering process exit monitor for PID {}", pid);

    let mut wait_handle: HANDLE = ptr::null_mut();

    // Box the context we need to pass to the callback
    let context = Box::new(ProcessExitContext { app_state, pid });

    let context_ptr = Box::into_raw(context);

    let result = unsafe {
        RegisterWaitForSingleObject(
            &mut wait_handle,
            process_handle,
            Some(process_exit_callback),
            context_ptr as *mut _,
            INFINITE,
            WT_EXECUTEONLYONCE,
        )
    };

    if result == 0 {
        // Clean up on error
        unsafe {
            let _ = Box::from_raw(context_ptr);
        }
        return Err(io::Error::last_os_error());
    }

    Ok(ProcessMonitorHandle { wait_handle })
}

#[cfg(target_os = "windows")]
struct ProcessExitContext {
    app_state: Weak<Mutex<AppState>>,
    pid: u32,
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn process_exit_callback(context: LPVOID, _timed_out: u8) {
    unsafe {
        let context = Box::from_raw(context as *mut ProcessExitContext);
        info!("Process exit detected for PID {}", context.pid);

        // Update app state
        if let Some(app_state) = context.app_state.upgrade() {
            if let Ok(mut state) = app_state.lock() {
                info!("Handling process exit for PID {}", context.pid);
                state.handle_process_exit(context.pid);
            } else {
                error!("Failed to lock app state when handling process exit");
            }
        } else {
            debug!("AppState was dropped, can't handle process exit");
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn register_process_exit_monitor(
    _process_handle: u32,
    _app_state: Weak<Mutex<AppState>>,
    _pid: u32,
) -> io::Result<ProcessMonitorHandle> {
    unimplemented!("Process exit monitoring not implemented for this platform");
}

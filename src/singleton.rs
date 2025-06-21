#[cfg(target_os = "windows")]
pub mod platform {
    use std::ptr;
    use winapi::shared::minwindef::FALSE;
    use winapi::shared::winerror::ERROR_ALREADY_EXISTS;
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::synchapi::CreateMutexW;
    use winapi::um::winnt::LPCWSTR;

    pub struct SingletonGuard {
        _private: (),
    }

    impl SingletonGuard {
        pub fn acquire() -> Option<Self> {
            let mutex_name: Vec<u16> = "Global\\SyncthingersSingletonMutex"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            unsafe {
                let handle = CreateMutexW(ptr::null_mut(), FALSE, mutex_name.as_ptr() as LPCWSTR);
                if handle.is_null() {
                    log::error!(
                        "Failed to create mutex for singleton enforcement: {}",
                        GetLastError()
                    );
                }

                if handle.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
                    let msg = "Another instance of Syncthingers is already running. Exiting.\0";
                    let caption = "Syncthingers Singleton\0";
                    crate::error_handling::show_native_error_dialog(msg, caption);
                    if !handle.is_null() {
                        CloseHandle(handle);
                    }
                    return None;
                }
            }
            Some(SingletonGuard { _private: ()})
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub mod platform {
    pub struct SingletonGuard {
        handle: (),
    }
    impl SingletonGuard {
        pub fn acquire() -> Option<Self> {
            // TODO: Implement file lock or other mechanism for non-Windows platforms
            Some(SingletonGuard { handle: () })
        }
    }
}

#[cfg(target_os = "windows")]
pub mod platform {
    use std::ptr;
    use winapi::um::synchapi::CreateMutexW;
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::winnt::LPCWSTR;
    use winapi::um::winuser::{MessageBoxW, MB_OK};
    use winapi::um::handleapi::CloseHandle;
    use winapi::shared::minwindef::FALSE;
    use winapi::shared::winerror::ERROR_ALREADY_EXISTS;

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
                if handle.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
                    let msg = "Another instance of Syncthingers is already running. Exiting.\0";
                    let caption = "Syncthingers Singleton\0";
                    MessageBoxW(
                        ptr::null_mut(),
                        msg.encode_utf16().collect::<Vec<u16>>().as_ptr(),
                        caption.encode_utf16().collect::<Vec<u16>>().as_ptr(),
                        MB_OK,
                    );
                    if !handle.is_null() {
                        CloseHandle(handle);
                    }
                    return None;
                }
                // Store handle in the struct if you want to release it on drop
            }
            Some(SingletonGuard { _private: () })
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub mod platform {
    pub struct SingletonGuard {
        _private: (),
    }
    impl SingletonGuard {
        pub fn acquire() -> Option<Self> {
            // TODO: Implement file lock or other mechanism for non-Windows platforms
            Some(SingletonGuard { _private: () })
        }
    }
}
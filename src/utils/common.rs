pub fn bytes_to_readable_string(bytes: u64) -> String {
    let kilobytes = 1024.0;
    let megabytes = kilobytes * 1024.0;
    let gigabytes = megabytes * 1024.0;
    let terabytes = gigabytes * 1024.0;

    if bytes as f64 >= terabytes {
        format!("{:.2} TB", bytes as f64 / terabytes)
    } else if bytes as f64 >= gigabytes {
        format!("{:.2} GB", bytes as f64 / gigabytes)
    } else if bytes as f64 >= megabytes {
        format!("{:.2} MB", bytes as f64 / megabytes)
    } else if bytes as f64 >= kilobytes {
        format!("{:.2} KB", bytes as f64 / kilobytes)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(target_os = "windows")]
pub mod memory {
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::psapi::GetProcessMemoryInfo;
    use winapi::um::psapi::PROCESS_MEMORY_COUNTERS;
    use winapi::um::winnt::PROCESS_QUERY_INFORMATION;

    use crate::err;
    use crate::model::error::Result;

    pub fn get_current_memory_usage(pid: Option<u32>) -> Result<usize> {
        let pid = pid.unwrap_or(std::process::id());
        unsafe {
            let process = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
            if process.is_null() {
                return err!("Failed to open process");
            }

            let mut mem_counters = PROCESS_MEMORY_COUNTERS {
                cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
                PageFaultCount: 0,
                PeakWorkingSetSize: 0,
                WorkingSetSize: 0,
                QuotaPeakPagedPoolUsage: 0,
                QuotaPagedPoolUsage: 0,
                QuotaPeakNonPagedPoolUsage: 0,
                QuotaNonPagedPoolUsage: 0,
                PagefileUsage: 0,
                PeakPagefileUsage: 0,
            };

            if GetProcessMemoryInfo(process, &mut mem_counters, mem_counters.cb) == 0 {
                return err!("Failed to get memory info");
            }

            Ok(mem_counters.PagefileUsage as usize)
        }
    }
}

#[cfg(target_os = "macos")]
pub mod memory {
    use sysinfo::{CpuRefreshKind, Pid, RefreshKind, System};

    use crate::model::error::Result;

    pub fn get_current_memory_usage(pid: Option<u32>) -> Result<usize> {
        let pid = pid.unwrap_or(std::process::id());
        let mut sys = System::new();
        sys.refresh_process(Pid::from_u32(pid));
        let self_id = std::process::id();
        if let Some(self_process) = sys.process(Pid::from_u32(self_id)) {
            return Ok(self_process.memory() as usize);
        }
        err!("Failed to get memory usage")
    }
}

#[cfg(target_os = "linux")]
pub mod memory {
    use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

    use crate::model::error::Result;

    pub fn get_current_memory_usage(pid: Option<u32>) -> Result<usize> {
        let pid = pid.unwrap_or(std::process::id());
        let mut sys = System::new_with_specifics(
            RefreshKind::new().with_processes(ProcessRefreshKind::new().with_memory()),
        );

        let process = sys.process(Pid::from_u32(pid));
        Ok(process.unwrap().memory() as usize)
    }
}

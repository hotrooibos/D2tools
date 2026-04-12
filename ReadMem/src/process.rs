#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(windows)]
use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;
#[cfg(windows)]
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, Process32FirstW, Process32NextW,
    TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS, PROCESSENTRY32W, MODULEENTRY32W,
};
#[cfg(windows)]
use windows_sys::Win32::System::Memory::{VirtualQueryEx, MEMORY_BASIC_INFORMATION};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::OpenProcess;

const PROCESS_VM_READ: u32 = 0x0010;
const PROCESS_QUERY_INFORMATION: u32 = 0x0400;

#[cfg(windows)]
pub struct Process {
    handle: HANDLE,
    pid: u32,
    base_address: usize,
    d2client_addr: usize,
}

#[cfg(windows)]
impl Process {
    pub fn open(process_name: &str) -> Option<Self> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == 0 {
                return None;
            }

            let mut entry: PROCESSENTRY32W = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

            if Process32FirstW(snapshot, &mut entry) == 0 {
                return None;
            }

            loop {
                let name = std::slice::from_raw_parts(
                    entry.szExeFile.as_ptr(),
                    entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(260),
                );
                let name_str = String::from_utf16_lossy(name);

                if name_str.to_lowercase().contains(&process_name.to_lowercase()) {
                    let pid = entry.th32ProcessID;
                    let access = PROCESS_VM_READ | PROCESS_QUERY_INFORMATION;
                    let handle = OpenProcess(access, 0, pid);
                    if handle != 0 {
                        let base_address = find_module_base(pid, "D2R.exe")
                            .or_else(|| find_module_base(pid, "D2R"))
                            .or_else(|| find_module_base(pid, "Diablo II Resurrected.exe"));
                        let d2client_addr = find_module_base(pid, "D2Client.dll")
                            .or_else(|| find_module_base(pid, "D2Client64.dll"));
                        
                        eprintln!("DEBUG: Found D2R modules - base: {:?}, d2client: {:?}", base_address, d2client_addr);
                        
                        return Some(Self {
                            handle,
                            pid,
                            base_address: base_address.unwrap_or(0),
                            d2client_addr: d2client_addr.unwrap_or(0),
                        });
                    }
                }

                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
            None
        }
    }

    pub fn read_u32(&self, address: usize) -> Option<u32> {
        let mut buffer: u32 = 0;
        let mut bytes_read: usize = 0;

        let result = unsafe {
            ReadProcessMemory(
                self.handle,
                address as *const _,
                &mut buffer as *mut _ as *mut _,
                4,
                &mut bytes_read,
            )
        };

        if result != 0 && bytes_read == 4 {
            Some(buffer)
        } else {
            None
        }
    }

    pub fn read_u8(&self, address: usize) -> Option<u8> {
        let mut buffer: u8 = 0;
        let mut bytes_read: usize = 0;

        let result = unsafe {
            ReadProcessMemory(
                self.handle,
                address as *const _,
                &mut buffer as *mut _ as *mut _,
                1,
                &mut bytes_read,
            )
        };

        if result != 0 && bytes_read == 1 {
            Some(buffer)
        } else {
            None
        }
    }

    pub fn read_u16(&self, address: usize) -> Option<u16> {
        let mut buffer: u16 = 0;
        let mut bytes_read: usize = 0;

        let result = unsafe {
            ReadProcessMemory(
                self.handle,
                address as *const _,
                &mut buffer as *mut _ as *mut _,
                2,
                &mut bytes_read,
            )
        };

        if result != 0 && bytes_read == 2 {
            Some(buffer)
        } else {
            None
        }
    }

    pub fn read_u64(&self, address: usize) -> Option<u64> {
        let mut buffer: u64 = 0;
        let mut bytes_read: usize = 0;

        let result = unsafe {
            ReadProcessMemory(
                self.handle,
                address as *const _,
                &mut buffer as *mut _ as *mut _,
                8,
                &mut bytes_read,
            )
        };

        if result != 0 && bytes_read == 8 {
            Some(buffer)
        } else {
            None
        }
    }

    pub fn read_i32(&self, address: usize) -> Option<i32> {
        self.read_u32(address).map(|v| v as i32)
    }

    pub fn read_array(&self, address: usize, size: usize) -> Option<Vec<u8>> {
        let mut buffer = vec![0u8; size];
        let mut bytes_read: usize = 0;

        let result = unsafe {
            ReadProcessMemory(
                self.handle,
                address as *const _,
                buffer.as_mut_ptr() as *mut _,
                size,
                &mut bytes_read,
            )
        };

        if result != 0 {
            buffer.truncate(bytes_read);
            Some(buffer)
        } else {
            None
        }
    }

    pub fn get_base_address(&self) -> usize {
        self.base_address
    }

    pub fn get_d2client_address(&self) -> usize {
        self.d2client_addr
    }

    pub fn is_valid(&self) -> bool {
        self.handle != 0
    }

    pub fn enumerate_memory_regions(&self) -> Vec<(usize, usize)> {
        let mut regions = Vec::new();
        
        #[repr(C)]
        struct MEMORY_BASIC_INFORMATION64 {
            base_address: u64,
            allocation_base: u64,
            allocation_protect: u32,
            alignment1: u32,
            region_size: u64,
            state: u32,
            protect: u32,
            type_: u32,
            alignment2: u32,
        }
        
        let mut addr: usize = 0;
        loop {
            let mut mbi: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
            let result = unsafe {
                VirtualQueryEx(
                    self.handle,
                    addr as *const _,
                    &mut mbi,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };
            
            if result == 0 {
                break;
            }
            
            let state = mbi.State;
            let protect = mbi.Protect;
            let region_size = mbi.RegionSize;
            
            // Only include committed, readable memory
            if state == 0x1000 && protect != 0 && protect != 0x01 { // MEM_COMMIT and not PAGE_NOACCESS
                let base = mbi.BaseAddress as usize;
                let size = mbi.RegionSize as usize;
                
                // Skip very small regions and system memory
                if size > 4096 && base < 0x7FFFFFFFFFFF {
                    regions.push((base, size));
                }
            }
            
            // Move to next region
            if region_size == 0 {
                break;
            }
            addr = (mbi.BaseAddress as usize) + (region_size as usize);
            
            // Safety limit
            if addr > 0x7FFFFFFFFFFF {
                break;
            }
        }
        
        regions
    }

    pub fn find_all_modules(&self) -> Vec<(String, usize)> {
        let mut modules = Vec::new();
        
        if self.pid == 0 {
            return modules;
        }
        
        unsafe {
            // Enumerate modules from the target process
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, self.pid);
            if snapshot == 0 {
                eprintln!("DEBUG: Failed to create module snapshot for PID {}", self.pid);
                return modules;
            }
            
            let mut entry: MODULEENTRY32W = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<MODULEENTRY32W>() as u32;
            
            if Module32FirstW(snapshot, &mut entry) == 0 {
                eprintln!("DEBUG: No modules found for PID {}", self.pid);
                return modules;
            }
            
            loop {
                let name = std::slice::from_raw_parts(
                    entry.szModule.as_ptr(),
                    entry.szModule.iter().position(|&c| c == 0).unwrap_or(256),
                );
                let name_str = String::from_utf16_lossy(name).to_string();
                let addr = entry.modBaseAddr as usize;
                
                eprintln!("DEBUG: Module '{}' at 0x{:016X}", name_str, addr);
                modules.push((name_str, addr));
                
                if Module32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        
        modules
    }
}

#[cfg(windows)]
impl Drop for Process {
    fn drop(&mut self) {
        if self.handle != 0 {
            unsafe {
                CloseHandle(self.handle);
            }
        }
    }
}

#[cfg(windows)]
fn find_module_base(process_id: u32, module_name: &str) -> Option<usize> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, process_id);
        if snapshot == 0 {
            return None;
        }

        let mut entry: MODULEENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<MODULEENTRY32W>() as u32;

        if Module32FirstW(snapshot, &mut entry) == 0 {
            return None;
        }

        loop {
            let name = std::slice::from_raw_parts(
                entry.szModule.as_ptr(),
                entry.szModule.iter().position(|&c| c == 0).unwrap_or(256),
            );
            let name_str = String::from_utf16_lossy(name);

            if name_str.to_lowercase() == module_name.to_lowercase() {
                return Some(entry.modBaseAddr as usize);
            }

            if Module32NextW(snapshot, &mut entry) == 0 {
                break;
            }
        }
        None
    }
}

#[cfg(windows)]
pub fn wait_for_process(process_name: &str, timeout_secs: Option<u64>) -> Option<Process> {
    use std::time::{Duration, Instant};

    let start = Instant::now();
    let timeout = timeout_secs.map(|s| Duration::from_secs(s));

    loop {
        if let Some(process) = Process::open(process_name) {
            return Some(process);
        }

        if let Some(timeout) = timeout {
            if start.elapsed() > timeout {
                return None;
            }
        }

        std::thread::sleep(Duration::from_millis(500));
    }
}

#[cfg(not(windows))]
pub struct Process;

#[cfg(not(windows))]
impl Process {
    pub fn open(_process_name: &str) -> Option<Self> { None }
    pub fn read_u32(&self, _address: usize) -> Option<u32> { None }
    pub fn read_u16(&self, _address: usize) -> Option<u16> { None }
    pub fn read_u64(&self, _address: usize) -> Option<u64> { None }
    pub fn read_i32(&self, _address: usize) -> Option<i32> { None }
    pub fn read_array(&self, _address: usize, _size: usize) -> Option<Vec<u8>> { None }
    pub fn get_base_address(&self) -> usize { 0 }
    pub fn get_d2client_address(&self) -> usize { 0 }
    pub fn is_valid(&self) -> bool { false }
}

#[cfg(not(windows))]
pub fn wait_for_process(_process_name: &str, _timeout_secs: Option<u64>) -> Option<Process> { None }

//! Process management syscalls
use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE},
    mm::{translated_byte_buffer, MapPermission, VirtAddr, VirtPageNum},
    task::{
        alloc_framed_area, change_program_brk, current_user_token, exit_current_and_run_next,
        get_current_task_fst_time, get_current_task_status, get_current_task_syscall_times,
        get_pte_by_vpn, suspend_current_and_run_next, unmap_sequence_area, TaskStatus,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let mut dest_ptr = translated_byte_buffer(
        current_user_token(),
        _ts as *const u8,
        core::mem::size_of::<TimeVal>(),
    );
    let src_ptr = &time_val as *const TimeVal as *const u8;
    unsafe {
        core::ptr::copy_nonoverlapping(
            src_ptr,
            dest_ptr[0].as_mut_ptr(),
            core::mem::size_of::<TimeVal>(),
        );
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let mut dest_ptr = translated_byte_buffer(
        current_user_token(),
        _ti as *const u8,
        core::mem::size_of::<TaskInfo>(),
    );
    let task_info = TaskInfo {
        status: get_current_task_status(),
        syscall_times: get_current_task_syscall_times(),
        time: (get_time_us() / 1000) - get_current_task_fst_time(),
    };
    let src_ptr = &task_info as *const TaskInfo as *const u8;
    unsafe {
        core::ptr::copy_nonoverlapping(
            src_ptr,
            dest_ptr[0].as_mut_ptr(),
            core::mem::size_of::<TaskInfo>(),
        );
    }
    0
}

// YOUR JOB: Implement mmap.
/// Port: X | W | R ;len = 3
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if _start % PAGE_SIZE != 0
        || _port & !0x7 != 0
        || _port & 0x7 == 0
        || _start >= usize::MAX
        || _start + _len >= usize::MAX
    {
        return -1;
    }
    let start_vpn = VirtAddr::from(_start).floor();
    let end_vpn = VirtAddr::from(_start + _len).ceil();
    //左闭右开
    for i in start_vpn.0..end_vpn.0 {
        if let Some(pte) = get_pte_by_vpn(VirtPageNum(i)) {
            if pte.is_valid() {
                return -1;
            }
        };
    }
    let permission = MapPermission::from_bits_truncate((_port << 1) as u8) | MapPermission::U;
    alloc_framed_area(start_vpn.into(), end_vpn.into(), permission);
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if _start % PAGE_SIZE != 0 || _start >= usize::MAX || _start + _len >= usize::MAX {
        return -1;
    }
    let start_vpn = VirtAddr::from(_start).floor();
    let end_vpn = VirtAddr::from(_start + _len).ceil();
    //左闭右开
    for i in start_vpn.0..end_vpn.0 {
        if let Some(pte) = get_pte_by_vpn(VirtPageNum(i)) {
            if !pte.is_valid() {
                return -1;
            }
        } else {
            return -1;
        };
    }
    unmap_sequence_area(start_vpn, end_vpn);
    0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

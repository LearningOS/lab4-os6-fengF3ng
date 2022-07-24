//! File and filesystem-related syscalls

use crate::mm::translated_byte_buffer;
use crate::mm::translated_str;
use crate::mm::translated_refmut;
use crate::task::current_user_token;
use crate::task::current_task;
use crate::fs::open_file;
use crate::fs::OpenFlags;
use crate::fs::Stat;
use crate::mm::UserBuffer;
use alloc::sync::Arc;
use crate::fs::{linkat, unlinkat};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.read(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(
        path.as_str(),
        OpenFlags::from_bits(flags).unwrap()
    ) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

// YOUR JOB: 扩展 easy-fs 和内核以实现以下三个 syscall

/*
功能：获取文件状态。

Ｃ接口： int fstat(int fd, struct Stat* st)

Rust 接口： fn fstat(fd: i32, st: *mut Stat) -> i32

参数：
        fd: 文件描述符
        st: 文件状态结构体
*/
pub fn sys_fstat(_fd: usize, _st: *mut Stat) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if _fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[_fd] {
        let _st: *mut Stat = translated_refmut(token, _st);
        file.info(_st);
        0
    } else {
        -1
    }
}

/*
功能：创建一个文件的一个硬链接， linkat标准接口 。

Ｃ接口： int linkat(int olddirfd, char* oldpath, int newdirfd, char* newpath, unsigned int flags)

Rust 接口： fn linkat(olddirfd: i32, oldpath: *const u8, newdirfd: i32, newpath: *const u8, flags: u32) -> i32

参数：
        olddirfd，newdirfd: 仅为了兼容性考虑，本次实验中始终为 AT_FDCWD (-100)，可以忽略。
        flags: 仅为了兼容性考虑，本次实验中始终为 0，可以忽略。
        oldpath：原有文件路径
        newpath: 新的链接文件路径。

说明：
        为了方便，不考虑新文件路径已经存在的情况（属于未定义行为），除非链接同名文件。
        返回值：如果出现了错误则返回 -1，否则返回 0。

可能的错误

        链接同名文件。
*/

pub fn sys_linkat(_old_name: *const u8, _new_name: *const u8) -> isize {
    let token = current_user_token();
    let old_name = translated_str(token, _old_name);
    let old_name = old_name.as_str();
    let new_name = translated_str(token, _new_name);
    let new_name = new_name.as_str();
    if old_name == new_name {
        -1
    } else {
        linkat(old_name, new_name);
        0
    }
}

/*
功能：取消一个文件路径到文件的链接, unlinkat标准接口 。

Ｃ接口： int unlinkat(int dirfd, char* path, unsigned int flags)

Rust 接口： fn unlinkat(dirfd: i32, path: *const u8, flags: u32) -> i32

参数：
        dirfd: 仅为了兼容性考虑，本次实验中始终为 AT_FDCWD (-100)，可以忽略。
        flags: 仅为了兼容性考虑，本次实验中始终为 0，可以忽略。
        path：文件路径。

说明：
        注意考虑使用 unlink 彻底删除文件的情况，此时需要回收inode以及它对应的数据块。

返回值：如果出现了错误则返回 -1，否则返回 0。

可能的错误
        文件不存在。
*/
pub fn sys_unlinkat(_name: *const u8) -> isize {
    let token = current_user_token();
    let name = translated_str(token, _name);
    let name = name.as_str();
    unlinkat(name)
}

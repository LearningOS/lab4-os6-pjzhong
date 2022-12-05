//! File and filesystem-related syscalls


use crate::fs::{open_file, OpenFlags, Stat, StatMode, ROOT_INODE};
use crate::mm::{translated_byte_buffer, translated_refmut, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};

// YOUR JOB: 修改 sys_write 使之通过测试
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner.inner.borrow_mut();
    if fd >= inner.fd_table.len() {
        return -1;
    }

    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        drop(inner);
        drop(task);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        println!("WriteError");
        return -1;
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let task = current_task().unwrap();
    let inner = task.inner.inner.borrow_mut();
    if fd >= inner.fd_table.len() {
        return -1;
    }

    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        drop(inner);
        drop(task);
        file.read(UserBuffer::new(translated_byte_buffer(
            current_user_token(),
            buf,
            len,
        ))) as isize
    } else {
        return -1;
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let flags = if let Some(flags) = OpenFlags::from_bits(flags) {
        flags
    } else {
        return -1;
    };


    let token = current_user_token();
    let path = translated_str(token, path);
    if let (Some(task), Some(inode)) = (current_task(), open_file(path.as_str(), flags)) {
        let mut inner = task.inner.inner.borrow_mut();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        //实在是神奇了.....究竟哪里搞错了。。。。。
        //去掉就报错了...
        //println!("openError:{:?}-{:?}", path, flags);
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner.inner.borrow_mut();
    if fd >= inner.fd_table.len() || inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

// YOUR JOB: 扩展 easy-fs 和内核以实现以下三个 syscall
pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize {
    let task = current_task().unwrap();
    let inner = task.inner.inner.borrow_mut();
    if fd >= inner.fd_table.len() || inner.fd_table[fd].is_none() {
        return -1;
    }

    return if let (Some(fd), Some(ts)) = (
        inner.fd_table[fd].clone(),
        translated_refmut(inner.get_user_token(), st),
    ) {
        ts.ino = fd.inode_id().map_or(0u64, |id| id as u64);
        //单层结构，只能是文件
        ts.mode = StatMode::FILE;
        if 0 < ts.ino {
            ts.nlink = ROOT_INODE.calc_hard_links(ts.ino as u32) as u32
        }
        0
    } else {
        -1
    };
}

pub fn sys_linkat(old_name: *const u8, new_name: *const u8) -> isize {
    let token = current_user_token();
    let old_name = translated_str(token, old_name);
    let new_name = translated_str(token, new_name);

    if old_name.eq(&new_name) {
        return -1;
    }

    if ROOT_INODE.link(&new_name, &old_name).is_some() {
        0
    } else {
        -1
    }
}

pub fn sys_unlinkat(name: *const u8) -> isize {
    let token = current_user_token();
    let name = translated_str(token, name);

    if ROOT_INODE.unlink(&name).is_some() {
        0
    } else {
        -1
    }
}
use alloc::{sync::Arc, vec::Vec};
use easy_fs::{EasyFileSystem, Inode};
use lazy_static::lazy_static;

use crate::{drivers::BLOCK_DEVICE, mm::UserBuffer, sync::UPSafeCell};

use super::File;

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}

impl OpenFlags {
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}

lazy_static! {
    pub static ref ROOT_INODE: Arc<Inode> = {
        let efs = EasyFileSystem::open(BLOCK_DEVICE.clone());
        Arc::new(EasyFileSystem::root_inode(&efs))
    };
}

pub struct OSInode {
    inode_id: u32,
    readable: bool,
    writable: bool,
    inner: UPSafeCell<OSINodeInner>,
}

impl File for OSInode {
    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.inner.borrow_mut();
        let mut total_read_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let read_size = inner.inode.read_at(inner.offset, *slice);
            if read_size == 0 {
                break;
            }

            inner.offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }

    fn write(&self, buf: crate::mm::UserBuffer) -> usize {
        let mut inner = self.inner.inner.borrow_mut();
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            let write_size = inner.inode.write_at(inner.offset, *&slice);
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }

    fn readable(&self) -> bool {
        self.readable
    }

    fn writable(&self) -> bool {
        self.writable
    }

    fn inode_id(&self) -> Option<u32> {
        Some(self.inode_id)
    }
}

pub struct OSINodeInner {
    offset: usize,
    inode: Arc<Inode>,
}

impl OSInode {
    pub fn new(inode_id: u32, readable: bool, writable: bool, inode: Arc<Inode>) -> Self {
        Self {
            inode_id,
            readable,
            writable,
            inner: UPSafeCell::new(OSINodeInner { offset: 0, inode }),
        }
    }

    pub fn read_all(&self) -> Vec<u8> {
        let mut inner = self.inner.inner.borrow_mut();
        let mut buffer = [0u8; 512];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = inner.inode.read_at(inner.offset, &mut buffer);
            if len == 0 {
                break;
            }

            inner.offset += len;
            v.extend_from_slice(&buffer[..len]);
        }
        v
    }
}

pub fn list_apps() {
    println!("/**** APPS ****");
    for app in ROOT_INODE.ls() {
        println!("{}", app);
    }
    println!("***************/")
}

pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    let (readable, writable) = flags.read_write();
    if flags.contains(OpenFlags::CREATE) {
        if let Some((inode_id, inode)) = ROOT_INODE.find_node(name) {
            inode.clear();
            Some(Arc::new(OSInode::new(inode_id, readable, writable, inode)))
        } else {
            ROOT_INODE.create_inode(name).map(|(inode_id, inode)| {
                Arc::new(OSInode::new(inode_id, readable, writable, inode))
            })
        }
    } else {
        ROOT_INODE.find_node(name).map(|(inode_id, inode)| {
            if flags.contains(OpenFlags::TRUNC) {
                inode.clear();
            }

            Arc::new(OSInode::new(inode_id, readable, writable, inode))
        })
    }
}

use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs::File,
    io::Read,
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
    slice::SliceIndex,
    time::{Duration, UNIX_EPOCH},
};

use fuser::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, ReplyOpen,
    ReplyStatfs, ReplyXattr, Request,
};
use index::Index;
use libc::ENOENT;
use tar::{Archive, Entry};

mod arena;
mod index;

const TTL: Duration = Duration::from_secs(1);

pub struct TarFs {
    path: &'static str,
    index: index::Index,
}

impl TarFs {
    pub fn new(path: &'static str) -> Self {
        let file = File::open(path).unwrap();
        let index = Index::build(file);
        for (i, v) in index.arena.arena.iter().enumerate() {
            println!(
                "inode {:?}, parent {:?} path {:?}, index {:?}",
                v.inode, v.parent, v.entry.path, i
            );
        }
        TarFs { path, index }
    }
}

impl<'a> Filesystem for TarFs {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        println!("lookup name {:?}, parent {}", name, parent);
        for e in &self.index.arena.arena {
            if e.parent == parent && e.entry.path == name {
                reply.entry(&TTL, &e.entry.attr, 0);
                return;
            }
        }

        reply.error(ENOENT);
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr ino {}", ino);
        match self.index.arena.get((ino - 1) as usize) {
            Some(e) => {
                reply.attr(&TTL, &e.entry.attr);
            }
            None => reply.error(ENOENT),
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: ReplyData,
    ) {
        println!("read!");
        let mut archive = Archive::new(File::open(self.path).unwrap());

        for (i, entry) in archive.entries().unwrap().enumerate() {
            if ino == i as u64 {
                let mut entry = entry.unwrap();
                let mut buf: Vec<u8> = vec![0; entry.header().size().unwrap() as usize];
                entry.read_exact(&mut buf).unwrap();
                reply.data(&buf);
                return;
            }
        }
        reply.error(ENOENT);
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        println!("readdir ino {}, offset {}", ino, offset);
        let root = self.index.arena.get((ino - 1) as usize).unwrap();
        let mut entries: Vec<(u64, FileType, &OsStr)> = vec![];
        if ino == 1 {
            println!("add .");
            entries.push((ino, FileType::Directory, OsStr::from_bytes(b".")));
            entries.push((ino, FileType::Directory, OsStr::from_bytes(b"..")));
        }

        for e in &root.children {
            let entry = self.index.arena.get((e - 1) as usize).unwrap();
            println!("entry {:?}", entry);
            entries.push((
                entry.inode,
                entry.entry.attr.kind,
                entry.entry.path.as_os_str(),
            ));
        }

        println!("skipping {} entries", offset);
        for (i, e) in entries.iter().enumerate().skip((offset) as usize) {
            let path = PathBuf::from(e.2);
            // Remove trailing slash for some reason
            let path = path.components().last().unwrap().as_os_str();

            println!(
                "ino {}, offset {}, kind {:?}, path {:?}",
                e.0,
                (i as i64 + 1) as i64,
                e.1,
                path
            );
            if reply.add(e.0, (i as i64 + 1) as i64, e.1, path) {
                println!("buffer full");
                break;
            }
        }

        reply.ok();
        println!("");
        println!("");
        println!("");
        println!("");
    }

    // fn opendir(&mut self, req: &Request, inode: u64, flags: i32, reply: ReplyOpen) {
    //     println!("opendir inode {}", inode);
    // }

    fn statfs(&mut self, _req: &Request, _ino: u64, reply: ReplyStatfs) {
        println!("statfs");
    }

    fn listxattr(&mut self, _req: &Request<'_>, inode: u64, size: u32, reply: ReplyXattr) {
        println!("kistxattr");
    }
    fn readlink(&mut self, _req: &Request<'_>, _ino: u64, reply: ReplyData) {
        println!("Readlink!");
        reply.error(libc::ENOSYS);
    }
}

use std::{ffi::OsStr, fs::File, os::unix::prelude::OsStrExt, path::PathBuf, time::Duration};

use fuser::{FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request};
use index::Index;
use libc::ENOENT;

mod arena;
mod index;

const TTL: Duration = Duration::from_secs(1);

pub struct TarFs {
    path: String,
    index: index::Index,
}

impl TarFs {
    pub fn new(path: String) -> Self {
        let file = File::open(&path).unwrap();
        let index = Index::build(file);

        TarFs { path, index }
    }
}

impl<'a> Filesystem for TarFs {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        match self.index.lookup_child(parent, name.to_str().unwrap()) {
            Some(e) => {
                reply.entry(&TTL, &e.entry.attr, 0);
            }
            None => {
                reply.error(ENOENT);
            }
        }
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
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let root = self.index.arena.get((ino - 1) as usize).unwrap();
        let mut entries: Vec<(u64, FileType, &OsStr)> = vec![];
        if ino == 1 {
            entries.push((ino, FileType::Directory, OsStr::from_bytes(b".")));
            entries.push((ino, FileType::Directory, OsStr::from_bytes(b"..")));
        }

        for e in &root.children {
            let entry = self.index.arena.get((e - 1) as usize).unwrap();
            entries.push((
                entry.inode,
                entry.entry.attr.kind,
                entry.entry.path.as_os_str(),
            ));
        }

        for (i, e) in entries.iter().enumerate().skip((offset) as usize) {
            let path = PathBuf::from(e.2);

            // Remove trailing slash for some reason
            let path = path.components().last().unwrap().as_os_str();

            if reply.add(e.0, (i as i64 + 1) as i64, e.1, path) {
                println!("buffer full");
                break;
            }
        }

        reply.ok();
    }
}

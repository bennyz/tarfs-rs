use std::{
    collections::{btree_map::Entry as MapEntry, BTreeMap},
    ffi::OsStr,
    fs::File,
    path::PathBuf,
    time::SystemTime,
};

use crate::arena;
use chrono::TimeZone;
use fuser::{FileAttr, FileType};
use tar::{Archive, Entry, EntryType};

#[derive(Debug, Clone)]
pub struct TarEntry {
    pub path: PathBuf,
    pub attr: FileAttr,
}

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub inode: u64,
    pub parent: u64,
    pub children: Vec<u64>,
    pub entry: TarEntry,
}

#[derive(Debug, Clone)]
pub struct Index {
    pub arena: arena::ArenaTree<IndexEntry>,
}

impl Index {
    pub fn build(file: File) -> Self {
        let mut archive = Archive::new(file);
        let mut lookup_map = BTreeMap::new();
        let mut arena = arena::ArenaTree::new();
        let root_entry = create_root_tar_entry();
        let root_index = create_root_index(root_entry.clone());
        arena.insert(root_index.clone(), 0);
        lookup_map.insert(root_entry.path.clone(), root_index);
        let mut next_inode = 2;
        for (i, e) in archive.entries().unwrap().enumerate() {
            let entry = e.unwrap();
            let full_path = entry.path().unwrap().to_path_buf();
            let tar_entry = TarEntry::from(entry);

            let mut clone = lookup_map.clone();
            let index_entry = clone.entry(tar_entry.clone().path).or_insert(IndexEntry {
                inode: next_inode,
                parent: 1, // Assign root by default
                children: vec![],
                entry: tar_entry.clone(),
            });

            let parent_path = PathBuf::from(
                full_path
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap_or(OsStr::new("")),
            );

            match lookup_map.entry(parent_path.clone()) {
                MapEntry::Occupied(mut e) => {
                    let parent_index = e.get_mut();
                    index_entry.parent = parent_index.inode;
                    arena
                        .get_mut((parent_index.inode - 1) as usize)
                        .unwrap()
                        .children
                        .push(next_inode);
                }
                MapEntry::Vacant(_) => {
                    let parent_index_entry = IndexEntry {
                        inode: next_inode as u64,
                        parent: 1,
                        children: vec![next_inode + 1],
                        entry: root_entry.clone(),
                    };
                    lookup_map.insert(parent_path.clone(), parent_index_entry.clone());
                    index_entry.parent = parent_index_entry.inode;
                    if parent_path != PathBuf::from(".") {
                        arena.push(parent_index_entry.clone());
                    }
                    next_inode += 1;
                    index_entry.inode = next_inode;
                }
            }

            let key = tar_entry.clone().path;
            let key = key.file_name().unwrap();
            lookup_map.insert(PathBuf::from(key), index_entry.clone());
            index_entry.entry.attr.ino = next_inode;
            arena.push(index_entry.clone());
            next_inode += 1;
        }

        Index { arena }
    }

    pub fn lookup_child(&self, parent_inode: u64, name: &str) -> Option<&IndexEntry> {
        let parent = self.arena.get(parent_inode as usize - 1).unwrap();
        for c in &parent.children {
            let child = self.arena.get(*c as usize - 1).unwrap();
            if child.entry.path.file_name().unwrap() == name {
                return Some(child);
            }
        }

        None
    }
}

fn create_root_tar_entry() -> TarEntry {
    TarEntry {
        path: PathBuf::from(""),
        attr: FileAttr {
            ino: 1,
            size: 13,
            blocks: 1,
            atime: SystemTime::now(),
            mtime: SystemTime::now(),
            ctime: SystemTime::now(),
            crtime: SystemTime::now(),
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: 1000,
            gid: 1000,
            rdev: 0,
            flags: 0,
            blksize: 512,
        },
    }
}

fn create_root_index(root: TarEntry) -> IndexEntry {
    IndexEntry {
        inode: 1,
        parent: 0,
        children: vec![],
        entry: root,
    }
}

impl Default for TarEntry {
    fn default() -> Self {
        Self {
            path: Default::default(),
            attr: FileAttr {
                ino: 0,
                size: 512,
                blocks: 1,
                atime: SystemTime::now(),
                mtime: SystemTime::now(),
                ctime: SystemTime::now(),
                crtime: SystemTime::now(),
                kind: FileType::Directory,
                perm: 0o755,
                nlink: 0,
                uid: 0,
                gid: 0,
                rdev: 0,
                blksize: 512,
                flags: 0,
            },
        }
    }
}

impl<'a> From<Entry<'a, File>> for TarEntry {
    fn from(entry: Entry<File>) -> Self {
        let kind = match entry.header().entry_type() {
            EntryType::Regular => FileType::RegularFile,
            EntryType::Directory => FileType::Directory,
            EntryType::Link => todo!(),
            EntryType::Symlink => FileType::Symlink,
            EntryType::Char => FileType::CharDevice,
            EntryType::Block => FileType::BlockDevice,
            EntryType::Fifo => FileType::NamedPipe,
            EntryType::Continuous => todo!(),
            EntryType::GNULongName => todo!(),
            EntryType::GNULongLink => todo!(),
            EntryType::GNUSparse => todo!(),
            EntryType::XGlobalHeader => todo!(),
            EntryType::XHeader => todo!(),
            EntryType::__Nonexhaustive(_) => todo!(),
        };

        let mtime: SystemTime = chrono::Utc
            .timestamp(entry.header().mtime().unwrap_or(0) as i64, 0)
            .into();
        let atime = SystemTime::now();
        let ctime = SystemTime::now();

        let attr = FileAttr {
            ino: 0,
            size: entry.header().size().unwrap(),
            blocks: 1,
            atime,
            mtime,
            ctime,
            crtime: SystemTime::now(),
            kind,
            perm: entry.header().mode().unwrap() as u16,
            nlink: 0,
            uid: entry.header().uid().unwrap() as u32,
            gid: entry.header().gid().unwrap() as u32,
            rdev: 0,
            blksize: 512,
            flags: 0,
        };
        let path = PathBuf::from(entry.path().unwrap().file_name().unwrap());
        TarEntry { path, attr }
    }
}

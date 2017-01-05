// InodeTable :: a bi-directional map of paths to inodes.
//
// Copyright (c) 2016 by William R. Fraser
//

use std::collections::BTreeMap;
use std::collections::btree_map::Entry::*;
use std::sync::Arc;
use std::path::{Path, PathBuf};

pub type Inode = u64;

#[derive(Debug)]
pub struct InodeTable {
    path_to_inode: BTreeMap<Arc<PathBuf>, Inode>,
    inode_to_path: BTreeMap<Inode, Arc<PathBuf>>,
    next_inode: u64,
}

impl InodeTable {
    pub fn new() -> InodeTable {
        InodeTable {
            path_to_inode: BTreeMap::new(),
            inode_to_path: BTreeMap::new(),
            next_inode: 1,
        }
    }

    pub fn add(&mut self, path: Arc<PathBuf>) -> Inode {
        let inode = self.next_inode;
        self.next_inode += 1;
        match self.path_to_inode.insert(path.clone(), inode) {
            Some(_) => {
                debug!("inode table: {:?}", self);
                debug!("added path: {:?}", path);
                panic!("duplicate path inserted into inode table!");
            },
            None => ()
        }
        self.inode_to_path.insert(inode, path.clone());
        inode
    }

    pub fn add_or_get(&mut self, path: Arc<PathBuf>) -> Inode {
        match self.path_to_inode.entry(path.clone()) {
            Vacant(entry) => {
                let inode = self.next_inode;
                self.next_inode += 1;
                entry.insert(inode);
                self.inode_to_path.insert(inode, path.clone());
                inode
            },
            Occupied(entry) => {
                *entry.get()
            }
        }
    }

    pub fn get_path(&self, inode: Inode) -> Option<Arc<PathBuf>> {
        match self.inode_to_path.get(&inode) {
            Some(rc) => Some(rc.clone()),
            None     => None
        }
    }

    pub fn get_inode(&self, path: &Path) -> Option<Inode> {
        match self.path_to_inode.get(Pathish::new(path)) {
            Some(inode) => Some(*inode),
            None        => None
        }
    }

    pub fn rename(&mut self, oldpath: &Path, newpath: Arc<PathBuf>) {
        let ino = self.path_to_inode.remove(Pathish::new(oldpath)).unwrap();
        self.inode_to_path.remove(&ino);

        self.path_to_inode.insert(newpath.clone(), ino).unwrap(); // this can replace a path with a new inode
        self.inode_to_path.insert(ino, newpath);
    }

    pub fn unlink(&mut self, path: &Path) {
        // The inode is now unreachable by this name, but the inode->path mapping remains.
        self.path_to_inode.remove(Pathish::new(path));
    }
}

// Facilitates comparing Rc<PathBuf> and &Path
struct Pathish {
    inner: Path,
}

impl Pathish {
    pub fn new(p: &Path) -> &Pathish {
        unsafe { ::std::mem::transmute(p) }
    }
}

impl ::std::borrow::Borrow<Pathish> for Arc<PathBuf> {
    fn borrow(&self) -> &Pathish {
        Pathish::new(self.as_path())
    }
}

impl ::std::cmp::Ord for Pathish {
    fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl ::std::cmp::PartialOrd for Pathish {
    fn partial_cmp(&self, other: &Pathish) -> Option<::std::cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl ::std::cmp::Eq for Pathish {
}

impl ::std::cmp::PartialEq for Pathish {
    fn eq(&self, other: &Pathish) -> bool {
        self.inner.eq(&other.inner)
    }
}

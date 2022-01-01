#[derive(Debug, Clone)]
pub struct ArenaTree<T> {
    arena: Vec<T>,
}

impl<T> ArenaTree<T> {
    pub fn new() -> Self {
        ArenaTree { arena: Vec::new() }
    }

    pub fn insert(&mut self, val: T, index: usize) {
        self.arena.insert(index, val);
    }

    pub fn push(&mut self, val: T) {
        self.arena.push(val);
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.arena.get_mut(index)
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.arena.get(index)
    }
}

use super::{fault, PeleError};
use std::sync::Arc;
pub(super) const PAGE: usize = 1024;
const DIRECTORY: usize = 256;
/// Two-level COW directory. No ancestor epochs or mutable shared allocations.
#[derive(Clone)]
pub(super) struct Pages<T: Copy + Default + PartialEq> {
    directories: Vec<Arc<Vec<Arc<Vec<T>>>>>,
    length: usize,
}
impl<T: Copy + Default + PartialEq> Default for Pages<T> {
    fn default() -> Self {
        Self {
            directories: Vec::new(),
            length: 0,
        }
    }
}
impl<T: Copy + Default + PartialEq> Pages<T> {
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn get(&self, index: usize) -> Result<T, PeleError> {
        if index >= self.length {
            return Err(fault());
        }
        Ok(self.directories[index / PAGE / DIRECTORY][index / PAGE % DIRECTORY][index % PAGE])
    }
    pub fn set(&mut self, index: usize, value: T) -> Result<(), PeleError> {
        if index >= self.length {
            return Err(fault());
        }
        if self.get(index)? == value {
            return Ok(());
        }
        let directory = Arc::make_mut(&mut self.directories[index / PAGE / DIRECTORY]);
        Arc::make_mut(&mut directory[index / PAGE % DIRECTORY])[index % PAGE] = value;
        Ok(())
    }
    pub fn push(&mut self, value: T) -> Result<usize, PeleError> {
        let index = self.length;
        self.length.checked_add(1).ok_or_else(fault)?;
        let page = index / PAGE;
        if page / DIRECTORY == self.directories.len() {
            self.directories.push(Arc::new(Vec::new()));
        }
        let directory = Arc::make_mut(&mut self.directories[page / DIRECTORY]);
        if page % DIRECTORY == directory.len() {
            directory.push(Arc::new(Vec::new()));
        }
        let values = Arc::make_mut(&mut directory[page % DIRECTORY]);
        if values.capacity() == 0 {
            values.reserve(PAGE);
        }
        values.push(value);
        self.length += 1;
        Ok(index)
    }
    pub fn reserve_span(&mut self, capacity: usize) -> Result<usize, PeleError> {
        let start = self.length;
        start
            .checked_add(capacity)
            .filter(|n| *n <= u32::MAX as usize)
            .ok_or_else(fault)?;
        for _ in 0..capacity {
            self.push(T::default())?;
        }
        Ok(start)
    }
    pub fn truncate(&mut self, length: usize) {
        if length >= self.length {
            return;
        }
        if length == 0 {
            self.directories.clear();
            self.length = 0;
            return;
        }
        let last = (length - 1) / PAGE;
        self.directories.truncate(last / DIRECTORY + 1);
        let directory = Arc::make_mut(&mut self.directories[last / DIRECTORY]);
        directory.truncate(last % DIRECTORY + 1);
        let final_length = (length - 1) % PAGE + 1;
        if directory[last % DIRECTORY].len() != final_length {
            Arc::make_mut(&mut directory[last % DIRECTORY]).truncate(final_length);
        }
        self.length = length;
    }
    pub fn visit(&self, mut visit: impl FnMut(usize, &[T])) {
        for (di, directory) in self.directories.iter().enumerate() {
            for (pi, page) in directory.iter().enumerate() {
                visit(di * DIRECTORY + pi, page);
            }
        }
    }
    pub fn equal_range(
        &self,
        start: usize,
        other: &Self,
        other_start: usize,
        count: usize,
    ) -> Result<bool, PeleError> {
        start
            .checked_add(count)
            .filter(|end| *end <= self.length)
            .ok_or_else(fault)?;
        other_start
            .checked_add(count)
            .filter(|end| *end <= other.length)
            .ok_or_else(fault)?;
        let mut offset = 0;
        while offset < count {
            let a = start + offset;
            let b = other_start + offset;
            let pa = &self.directories[a / PAGE / DIRECTORY][a / PAGE % DIRECTORY];
            let pb = &other.directories[b / PAGE / DIRECTORY][b / PAGE % DIRECTORY];
            let n = (count - offset)
                .min(pa.len() - a % PAGE)
                .min(pb.len() - b % PAGE);
            if !(a % PAGE == b % PAGE && Arc::ptr_eq(pa, pb))
                && pa[a % PAGE..a % PAGE + n] != pb[b % PAGE..b % PAGE + n]
            {
                return Ok(false);
            }
            offset += n;
        }
        Ok(true)
    }
    pub fn chunks(
        &self,
        start: usize,
        count: usize,
        mut visit: impl FnMut(&[T]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        let end = start
            .checked_add(count)
            .filter(|end| *end <= self.length)
            .ok_or_else(fault)?;
        let mut at = start;
        while at < end {
            let page = &self.directories[at / PAGE / DIRECTORY][at / PAGE % DIRECTORY];
            let n = (end - at).min(page.len() - at % PAGE);
            visit(&page[at % PAGE..at % PAGE + n])?;
            at += n;
        }
        Ok(())
    }
}

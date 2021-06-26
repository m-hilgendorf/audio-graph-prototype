//! # `audio_graph::matrix`
use std::cmp::Ordering;
/// The direction of an edge in a matrix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Incoming,
    Outgoing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    row: usize,
    col: usize,
    dir: Dir,
}

/// An adjacency matrix.
pub struct AdjMatrix {
    entries: Vec<Entry>,
}

impl AdjMatrix {
    /// Lookup algorithm.
    ///
    /// If the matrix contains an entry at (row, col), then Ok() is returned
    /// with the index of the entry.
    ///
    /// If the matrix does not contain an entry at (row, col), then Err() is
    /// returned with the index where it may be inserted.
    ///
    /// Potential improvement: the vector is always sorted, so a binary search
    /// may be used instead of linear.
    fn lookup(&self, row: usize, col: usize) -> Result<usize, usize> {
        let mut idx = 0;
        let mut found_row = false;
        while idx < self.entries.len() && self.entries[idx].row <= row {
            if found_row {
                match self.entries[idx].col.cmp(&col) {
                    Ordering::Equal => return Ok(idx),
                    Ordering::Greater => return Err(idx),
                    Ordering::Less => idx += 1,
                }
            } else {
                match self.entries[idx].row.cmp(&row) {
                    Ordering::Equal => found_row = true,
                    Ordering::Greater => return Err(idx),
                    Ordering::Less => idx += 1,
                }
            }
        }
        Err(idx)
    }

    /// Insert an entry into the matrix.
    fn insert(&mut self, entry: Entry) {
        let index = match self.lookup(entry.row, entry.col) {
            Ok(i) => i,
            Err(i) => i,
        };
        self.entries.insert(index, entry);
    }

    /// Remove an entry from the matrix.
    fn remove(&mut self, row: usize, col: usize) {
        if let Ok(idx) = self.lookup(row, col) {
            self.entries.remove(idx);
        }
    }

    /// Remove all entries corresponding to a row or column (in other words, delete the
    /// row and column corresponding to `idx`)
    fn remove_all(&mut self, idx: usize) {
        self.entries = self
            .entries
            .iter()
            .filter(|e| e.row != idx && e.col != idx)
            .copied()
            .collect();
    }

    /// Return the adjacencies of an index.
    pub fn adjacent<'a>(&'a self, node: usize) -> impl Iterator<Item = (usize, Dir)> + 'a {
        (node..)
            .take_while(move |i| self.entries[*i].row == self.entries[node].row)
            .map(move |idx| {
                let Entry { row, col, dir } = self.entries[idx];
                (col, dir)
            })
    }

    /// Return an iterator of the incoming edges to the node
    pub fn incoming<'a> (&'a self, node: usize) -> impl Iterator<Item = usize> + 'a {
        self.adjacent(node).filter_map(|(n, d)| match d { Dir::Incoming => Some(n), Dir::Outgoing => None })
    }

    /// Return an iterator of the outgoing edges from the node
    pub fn outgoing<'a> (&'a self, node: usize) -> impl Iterator<Item = usize> + 'a {
        self.adjacent(node).filter_map(|(n, d)| match d { Dir::Outgoing => Some(n), Dir::Incoming => None })
    }

    /// Connect two nodes in the graph
    pub fn connect (&mut self, src:usize, dst:usize) {
        self.insert(Entry { row: src, col: dst, dir: Dir::Outgoing });
        self.insert(Entry { row: dst, col: src, dir: Dir::Incoming });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matrix_lookup() {
        let matrix = AdjMatrix {
            entries: vec![
                Entry {
                    row: 0,
                    col: 1,
                    dir: Dir::Outgoing,
                },
                Entry {
                    row: 0,
                    col: 3,
                    dir: Dir::Outgoing,
                },
                Entry {
                    row: 1,
                    col: 0,
                    dir: Dir::Incoming,
                },
                Entry {
                    row: 1,
                    col: 3,
                    dir: Dir::Outgoing,
                },
                Entry {
                    row: 3,
                    col: 0,
                    dir: Dir::Incoming,
                },
                Entry {
                    row: 3,
                    col: 1,
                    dir: Dir::Incoming,
                },
            ],
        };
        assert_eq!(matrix.lookup(0, 1), Ok(0));
        assert_eq!(matrix.lookup(0, 3), Ok(1));
        assert_eq!(matrix.lookup(1, 0), Ok(2));
        assert_eq!(matrix.lookup(1, 3), Ok(3));
        assert_eq!(matrix.lookup(3, 0), Ok(4));
        assert_eq!(matrix.lookup(3, 1), Ok(5));
        assert_eq!(matrix.lookup(0, 4), Err(2));
        assert_eq!(matrix.lookup(3, 3), Err(6));
        assert_eq!(matrix.lookup(2, 1), Err(4));
    }

    #[test]
    fn matrix_insertion() {
        let mut matrix = AdjMatrix { entries: vec![] };
        matrix.insert(Entry {
            row: 0,
            col: 1,
            dir: Dir::Outgoing,
        });
        matrix.insert(Entry {
            row: 1,
            col: 0,
            dir: Dir::Incoming,
        });
        matrix.insert(Entry {
            row: 0,
            col: 3,
            dir: Dir::Outgoing,
        });
        matrix.insert(Entry {
            row: 3,
            col: 0,
            dir: Dir::Incoming,
        });
        assert_eq!(matrix.lookup(0, 1), Ok(0));
        assert_eq!(matrix.lookup(0, 3), Ok(1));
        assert_eq!(matrix.lookup(1, 0), Ok(2));
        assert_eq!(matrix.lookup(3, 0), Ok(3));
    }

    #[test]
    fn matrix_removal() {
        let mut matrix = AdjMatrix { entries: vec![] };
        matrix.insert(Entry {
            row: 0,
            col: 1,
            dir: Dir::Outgoing,
        });
        matrix.insert(Entry {
            row: 0,
            col: 2,
            dir: Dir::Outgoing,
        });
        matrix.insert(Entry {
            row: 0,
            col: 3,
            dir: Dir::Outgoing,
        });
        matrix.insert(Entry {
            row: 1,
            col: 0,
            dir: Dir::Incoming,
        });
        matrix.insert(Entry {
            row: 3,
            col: 0,
            dir: Dir::Incoming,
        });
        assert_eq!(matrix.lookup(0, 1), Ok(0));
        assert_eq!(matrix.lookup(0, 3), Ok(1));
        assert_eq!(matrix.lookup(1, 0), Ok(2));
        assert_eq!(matrix.lookup(3, 0), Ok(3));
    }
}

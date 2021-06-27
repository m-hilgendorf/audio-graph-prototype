//! # `audio_graph::matrix`
//!
//! An adjacency matrix for nodes with multiple ports.
//!
//! ## Usage
//! ```
//! // create a new graph
//! use audio_graph::matrix::AdjMatrix;
//!
//! let mut matrix = AdjMatrix::default();
//!
//! // connect some nodes to ports
//! matrix.connect((0, 0), (2, 0)); // connect node 0 port 0 to node 2 port 0
//! matrix.connect((0, 1), (2, 1)); // connect node 0 port 1 to node 2 port 1
//!
//! for (node, port) in matrix.outgoing(0, 0) {
//!     println!("0 is connected to {}.{}", node, port);    
//! }
//!
//! for (node, port) in matrix.incoming(2, 0) {
//!     println!("2.0 is connected to {}.{}", node, port);
//! }
//! ```

use std::cmp::Ordering;

/// The direction of an edge in a matrix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Incoming,
    Outgoing,
}

impl std::ops::Neg for Dir {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Dir::Incoming => Dir::Outgoing,
            Dir::Outgoing => Dir::Incoming
        }
    }
}

///! An entry into the matrix
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Entry {
    row: usize,
    col: usize,
    port: usize,
    dir: Dir,
}

/// An adjacency matrix.
#[derive(Clone, Debug, Default)]
pub struct AdjMatrix {
    entries: Vec<Entry>,
}

impl Entry {
    fn new(row: usize, col: usize, port: usize, dir: Dir) -> Self {
        Self {
            row,
            col,
            port,
            dir,
        }
    }
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
    fn lookup(&self, row: usize, col: usize, port: usize) -> Result<usize, usize> {
        //FIXME: this loop can be cleaned up.
        //       algorithm:
        //       - find the first element of the matrix with entry.row == row
        //       - if no entries are found, return Err(index) where index = index of first entry.row > row.
        //       - find the first element of the row with entry.col == col
        //       - if no entry is found, return Err(index) where index = index of the first entry.col > col.
        //       - return Ok(index) of the first element of the column with entry.port == port.
        //       - if no entry is found, return Err(index) where index = index of the first entry.port > port.
        let mut idx = 0;
        let mut found_row = false;
        let mut found_col = false;

        while idx < self.entries.len() && self.entries[idx].row <= row {
            if found_row {
                if found_col {
                    match self.entries[idx].port.cmp(&port) {
                        Ordering::Equal => return Ok(idx),
                        Ordering::Greater => return Err(idx),
                        Ordering::Less => idx += 1,
                    }
                } else {
                    match self.entries[idx].col.cmp(&col) {
                        Ordering::Equal => found_col = true,
                        Ordering::Greater => return Err(idx),
                        Ordering::Less => idx += 1,
                    }
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
        let index = match self.lookup(entry.row, entry.col, entry.port) {
            Ok(i) => i,
            Err(i) => i,
        };
        self.entries.insert(index, entry);
    }

    /// Remove an entry from the matrix.
    fn remove(&mut self, row: usize, col: usize, port: usize) {
        if let Ok(idx) = self.lookup(row, col, port) {
            self.entries.remove(idx);
        }
    }

    /// Remove all entries corresponding to a row or column (in other words, delete the
    /// row and column corresponding to `idx`).
    fn remove_all(&mut self, idx: usize) {
        self.entries = self
            .entries
            .iter()
            .filter(|e| e.row != idx && e.col != idx)
            .copied()
            .collect();
    }

    /// Return the entries in the adjacency matrix for a node.
    fn entries<'a>(&'a self, node: usize) -> impl Iterator<Item = Entry> + 'a {
        (node..self.entries.len())
            .take_while(move |i| self.entries[*i].row == self.entries[node].row)
            .map(move |i| self.entries[i])
    }

    /// Return the adjacent entries to a node given a port.
    fn adjacent_entries<'a>(
        &'a self,
        node: usize,
        port: usize,
    ) -> impl Iterator<Item = Entry> + 'a {
        self.entries(node).filter(move |e| e.port == port)
    }

    fn dir_entries<'a> (
        &'a self, 
        node: usize,
        port: usize,
        dir: Dir
    ) -> impl Iterator<Item = Entry> + 'a {
        self.adjacent_entries(node, port).filter(move |e| e.dir == dir)
    }

    /// Return the adjacent incoming entries to a node given a port.
    fn incoming_entries<'a>(
        &'a self,
        node: usize,
        port: usize,
    ) -> impl Iterator<Item = Entry> + 'a {
        self.dir_entries(node, port, Dir::Incoming)
    }

    /// Return the adjecent outgoing entries to a node given a port.
    fn outgoing_entries<'a>(
        &'a self,
        node: usize,
        port: usize,
    ) -> impl Iterator<Item = Entry> + 'a {
        self.dir_entries(node, port, Dir::Outgoing)
    }

    /// Return the adjacencies of an index.
    pub fn adjacent<'a>(
        &'a self,
        node: usize,
        port: usize,
    ) -> impl Iterator<Item = (usize, usize, Dir)> + 'a {
        self.entries(node).filter_map(move |e| {
            if e.port == port {
                Some((e.col, e.port, e.dir))
            } else {
                None
            }
        })
    }

    /// Return an iterator of the incoming edges to the node.
    pub fn incoming<'a>(
        &'a self,
        node: usize,
        port: usize,
    ) -> impl Iterator<Item = (usize, usize)> + 'a {
        self.adjacent(node, port)
            .filter_map(|(node, port, dir)| match dir {
                Dir::Incoming => Some((node, port)),
                _ => None,
            })
    }

    /// Return an iterator of the outgoing edges from the node.
    pub fn outgoing<'a>(
        &'a self,
        node: usize,
        port: usize,
    ) -> impl Iterator<Item = (usize, usize)> + 'a {
        self.adjacent(node, port)
            .filter_map(|(node, port, dir)| match dir {
                Dir::Outgoing => Some((node, port)),
                _ => None,
            })
    }

    /// Connect two nodes in the graph, `(src, port) -> (dst, port)`.
    pub fn connect(&mut self, src: (usize, usize), dst: (usize, usize)) {
        self.insert(Entry::new(src.0, dst.0, src.1, Dir::Outgoing));
        self.insert(Entry::new(dst.0, src.0, dst.1, Dir::Incoming));
    }

    /// Disconnect two nodes in the graph, if they are connected.
    pub fn disconnect(&mut self, src: (usize, usize), dst: (usize, usize)) {
        match (
            self.lookup(src.0, dst.0, src.1),
            self.lookup(dst.0, src.0, dst.1),
        ) {
            (Ok(src), Ok(dst)) => {
                self.entries.remove(src);
                self.entries.remove(dst - 1);
            }
            _ => (), // FIXME: Error here
        }
    }

    /// Return the indegree (number of incoming edges) to a node in the graph
    pub fn indegree(&self, node: usize) -> usize {
        self.degree(node, Dir::Incoming)
    }

    /// Return the outdegree (number of outgoing edges) to a node in the graph
    pub fn outdegree(&self, node: usize) -> usize {
        self.degree(node, Dir::Outgoing)
    }

    /// Return the number of nodes that this graph is aware of.
    pub fn num_nodes(&self) -> usize {
        self.entries.last().map(|e| e.row).unwrap_or(0)
    }

    /// Returns the number of ports that this graph is aware of for a given node.
    pub fn num_ports(&self, node:usize) -> usize {
        self.entries(node).fold(0, |count, e| count.max(e.port))
    }

    /// Internal. Returns the indegree or outdegree of a node.
    fn degree(&self, node: usize, dir: Dir) -> usize {
        self.entries(node)
            .fold(0, |count, e| count + if e.dir == dir { 1 } else { 0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matrix_lookup() {
        let matrix = AdjMatrix {
            entries: vec![
                Entry::new(0, 1, 0, Dir::Outgoing),
                Entry::new(0, 3, 0, Dir::Outgoing),
                Entry::new(1, 0, 0, Dir::Incoming),
                Entry::new(1, 3, 0, Dir::Outgoing),
                Entry::new(3, 0, 0, Dir::Incoming),
                Entry::new(3, 1, 0, Dir::Incoming),
            ],
        };
        assert_eq!(matrix.lookup(0, 1, 0), Ok(0));
        assert_eq!(matrix.lookup(0, 3, 0), Ok(1));
        assert_eq!(matrix.lookup(1, 0, 0), Ok(2));
        assert_eq!(matrix.lookup(1, 3, 0), Ok(3));
        assert_eq!(matrix.lookup(3, 0, 0), Ok(4));
        assert_eq!(matrix.lookup(3, 1, 0), Ok(5));
        assert_eq!(matrix.lookup(0, 4, 0), Err(2));
        assert_eq!(matrix.lookup(3, 3, 0), Err(6));
        assert_eq!(matrix.lookup(2, 1, 0), Err(4));
    }

    #[test]
    fn matrix_insertion() {
        let mut matrix = AdjMatrix::default();
        matrix.insert(Entry::new(0, 1, 0, Dir::Outgoing));
        matrix.insert(Entry::new(0, 3, 0, Dir::Outgoing));
        matrix.insert(Entry::new(1, 0, 0, Dir::Incoming));
        matrix.insert(Entry::new(1, 3, 0, Dir::Outgoing));
        assert_eq!(matrix.lookup(0, 1, 0), Ok(0));
        assert_eq!(matrix.lookup(0, 3, 0), Ok(1));
        assert_eq!(matrix.lookup(1, 0, 0), Ok(2));
        assert_eq!(matrix.lookup(1, 3, 0), Ok(3));
    }

    #[test]
    fn matrix_removal() {
        let mut matrix = AdjMatrix::default();
        matrix.insert(Entry::new(0, 1, 0, Dir::Outgoing));
        matrix.insert(Entry::new(0, 3, 0, Dir::Outgoing));
        matrix.insert(Entry::new(1, 0, 0, Dir::Incoming));
        matrix.insert(Entry::new(1, 3, 0, Dir::Outgoing));
        assert_eq!(matrix.lookup(0, 1, 0), Ok(0));
        assert_eq!(matrix.lookup(0, 3, 0), Ok(1));
        assert_eq!(matrix.lookup(1, 0, 0), Ok(2));
        assert_eq!(matrix.lookup(1, 3, 0), Ok(3));
        matrix.remove(0, 3, 0);
        assert_eq!(matrix.lookup(1, 0, 0), Ok(1));
        assert_eq!(matrix.lookup(1, 3, 0), Ok(2));
    }

    #[test]
    fn matrix_connection() {
        let mut matrix = AdjMatrix::default();
        matrix.connect((0, 0), (1, 0));
        assert_eq!(matrix.lookup(0, 1, 0), Ok(0));
        assert_eq!(matrix.lookup(1, 0, 0), Ok(1));
    }
}

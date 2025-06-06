// This file is part of the FlatGeobuf project.
// Much of this code is derived from the `packed_r_tree.rs` file of the FlatGeobuf project:
// https://github.com/flatgeobuf/flatgeobuf/blob/master/src/rust/src/packed_r_tree.rs
// Copyright (c) 2018, Björn Harrtell, Postnummer Stockholm AB (BSD 2-Clause License)
//
// See LICENSE file for full license text.
//! Create and read a [packed Hilbert R-Tree](https://en.wikipedia.org/wiki/Hilbert_R-tree#Packed_Hilbert_R-trees)
//! to enable fast bounding box spatial filtering.

// ... rest of the code ...
//! Create and read a [packed Hilbert R-Tree](https://en.wikipedia.org/wiki/Hilbert_R-tree#Packed_Hilbert_R-trees)
//! to enable fast bounding box spatial filtering.

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use core::f64;
pub use error::Error;
#[cfg(feature = "http")]
use http_range_client::{AsyncBufferedHttpRangeClient, AsyncHttpRangeClient};
use std::cmp::min;
use std::collections::VecDeque;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::mem::size_of;
use std::ops::Range;

mod error;

// This implementation was derived from FlatGeobuf's implemenation.

#[derive(Clone, PartialEq, Debug)]
#[repr(C)]
/// R-Tree node
pub struct NodeItem {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
    /// Byte offset in feature data section
    pub offset: u64,
}

impl NodeItem {
    #[deprecated(
        note = "Use NodeItem::bounds instead if you're only using the node item for bounds checking"
    )]
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> NodeItem {
        Self::bounds(min_x, min_y, max_x, max_y)
    }
    pub fn bounds(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> NodeItem {
        NodeItem {
            min_x,
            min_y,
            max_x,
            max_y,
            offset: 0,
        }
    }

    pub fn create(offset: u64) -> NodeItem {
        NodeItem {
            min_x: f64::INFINITY,
            min_y: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            max_y: f64::NEG_INFINITY,
            offset,
        }
    }

    pub fn from_reader(mut rdr: impl Read) -> Result<Self, Error> {
        Ok(NodeItem {
            min_x: rdr.read_f64::<LittleEndian>()?,
            min_y: rdr.read_f64::<LittleEndian>()?,
            max_x: rdr.read_f64::<LittleEndian>()?,
            max_y: rdr.read_f64::<LittleEndian>()?,
            offset: rdr.read_u64::<LittleEndian>()?,
        })
    }

    fn from_bytes(raw: &[u8]) -> Result<Self, Error> {
        Self::from_reader(&mut Cursor::new(raw))
    }

    pub fn write<W: Write>(&self, wtr: &mut W) -> std::io::Result<()> {
        wtr.write_f64::<LittleEndian>(self.min_x)?;
        wtr.write_f64::<LittleEndian>(self.min_y)?;
        wtr.write_f64::<LittleEndian>(self.max_x)?;
        wtr.write_f64::<LittleEndian>(self.max_y)?;
        wtr.write_u64::<LittleEndian>(self.offset)?;
        Ok(())
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    pub fn sum(mut a: NodeItem, b: &NodeItem) -> NodeItem {
        a.expand(b);
        a
    }

    pub fn expand(&mut self, r: &NodeItem) {
        if r.min_x < self.min_x {
            self.min_x = r.min_x;
        }
        if r.min_y < self.min_y {
            self.min_y = r.min_y;
        }
        if r.max_x > self.max_x {
            self.max_x = r.max_x;
        }
        if r.max_y > self.max_y {
            self.max_y = r.max_y;
        }
    }

    pub fn expand_xy(&mut self, x: f64, y: f64) {
        if x < self.min_x {
            self.min_x = x;
        }
        if y < self.min_y {
            self.min_y = y;
        }
        if x > self.max_x {
            self.max_x = x;
        }
        if y > self.max_y {
            self.max_y = y;
        }
    }

    pub fn intersects(&self, r: &NodeItem) -> bool {
        if self.max_x < r.min_x {
            return false;
        }
        if self.max_y < r.min_y {
            return false;
        }
        if self.min_x > r.max_x {
            return false;
        }
        if self.min_y > r.max_y {
            return false;
        }
        true
    }

    /// Check if a point is inside or on the boundary of this NodeItem's bounding box
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    /// Calculate the squared Euclidean distance from a point to the centroid of this NodeItem
    pub fn centroid_distance_squared(&self, x: f64, y: f64) -> f64 {
        let centroid_x = (self.min_x + self.max_x) / 2.0;
        let centroid_y = (self.min_y + self.max_y) / 2.0;
        let dx = x - centroid_x;
        let dy = y - centroid_y;
        dx * dx + dy * dy
    }

    /// Calculate the minimum squared distance from a point to this NodeItem's bounding box
    /// Returns 0 if the point is inside the bbox
    pub fn min_distance_squared(&self, x: f64, y: f64) -> f64 {
        if self.contains_point(x, y) {
            return 0.0;
        }

        // Calculate closest point on the bbox
        let closest_x = x.clamp(self.min_x, self.max_x);
        let closest_y = y.clamp(self.min_y, self.max_y);

        // Calculate squared distance
        let dx = x - closest_x;
        let dy = y - closest_y;
        dx * dx + dy * dy
    }
}

/// Read full capacity of vec from data stream
fn read_node_vec(node_items: &mut Vec<NodeItem>, mut data: impl Read) -> Result<(), Error> {
    node_items.clear();
    for _ in 0..node_items.capacity() {
        node_items.push(NodeItem::from_reader(&mut data)?);
    }
    Ok(())
}

/// Read partial item vec from data stream
fn read_node_items<R: Read + Seek>(
    data: &mut R,
    base: u64,
    node_index: usize,
    length: usize,
) -> Result<Vec<NodeItem>, Error> {
    let mut node_items = Vec::with_capacity(length);
    data.seek(SeekFrom::Start(
        base + (node_index * size_of::<NodeItem>()) as u64,
    ))?;
    read_node_vec(&mut node_items, data)?;
    Ok(node_items)
}

/// Read partial item vec from http
#[cfg(feature = "http")]
async fn read_http_node_items<T: AsyncHttpRangeClient>(
    client: &mut AsyncBufferedHttpRangeClient<T>,
    base: usize,
    node_ids: &Range<usize>,
) -> Result<Vec<NodeItem>, Error> {
    let begin = base + node_ids.start * size_of::<NodeItem>();
    let length = node_ids.len() * size_of::<NodeItem>();
    let bytes = client
        // we've  already determined precisely which nodes to fetch - no need for extra.
        .min_req_size(0)
        .get_range(begin, length)
        .await?;

    let mut node_items = Vec::with_capacity(node_ids.len());
    debug_assert_eq!(bytes.len(), length);
    for node_item_bytes in bytes.chunks(size_of::<NodeItem>()) {
        node_items.push(NodeItem::from_bytes(node_item_bytes)?);
    }
    Ok(node_items)
}

#[derive(Debug, Clone, Copy)]
pub enum Query {
    BBox(f64, f64, f64, f64),
    PointIntersects(f64, f64),
    PointNearest(f64, f64),
}

#[derive(Debug)]
/// Bbox filter search result
pub struct SearchResultItem {
    /// Byte offset in feature data section
    pub offset: usize,
    /// Feature number
    pub index: usize,
}

const HILBERT_MAX: u32 = (1 << 16) - 1;

// Based on public domain code at https://github.com/rawrunprotected/hilbert_curves
fn hilbert(x: u32, y: u32) -> u32 {
    let mut a = x ^ y;
    let mut b = 0xFFFF ^ a;
    let mut c = 0xFFFF ^ (x | y);
    let mut d = x & (y ^ 0xFFFF);

    let mut aa = a | (b >> 1);
    let mut bb = (a >> 1) ^ a;
    let mut cc = ((c >> 1) ^ (b & (d >> 1))) ^ c;
    let mut dd = ((a & (c >> 1)) ^ (d >> 1)) ^ d;

    a = aa;
    b = bb;
    c = cc;
    d = dd;
    aa = (a & (a >> 2)) ^ (b & (b >> 2));
    bb = (a & (b >> 2)) ^ (b & ((a ^ b) >> 2));
    cc ^= (a & (c >> 2)) ^ (b & (d >> 2));
    dd ^= (b & (c >> 2)) ^ ((a ^ b) & (d >> 2));

    a = aa;
    b = bb;
    c = cc;
    d = dd;
    aa = (a & (a >> 4)) ^ (b & (b >> 4));
    bb = (a & (b >> 4)) ^ (b & ((a ^ b) >> 4));
    cc ^= (a & (c >> 4)) ^ (b & (d >> 4));
    dd ^= (b & (c >> 4)) ^ ((a ^ b) & (d >> 4));

    a = aa;
    b = bb;
    c = cc;
    d = dd;
    cc ^= (a & (c >> 8)) ^ (b & (d >> 8));
    dd ^= (b & (c >> 8)) ^ ((a ^ b) & (d >> 8));

    a = cc ^ (cc >> 1);
    b = dd ^ (dd >> 1);

    let mut i0 = x ^ y;
    let mut i1 = b | (0xFFFF ^ (i0 | a));

    i0 = (i0 | (i0 << 8)) & 0x00FF00FF;
    i0 = (i0 | (i0 << 4)) & 0x0F0F0F0F;
    i0 = (i0 | (i0 << 2)) & 0x33333333;
    i0 = (i0 | (i0 << 1)) & 0x55555555;

    i1 = (i1 | (i1 << 8)) & 0x00FF00FF;
    i1 = (i1 | (i1 << 4)) & 0x0F0F0F0F;
    i1 = (i1 | (i1 << 2)) & 0x33333333;
    i1 = (i1 | (i1 << 1)) & 0x55555555;

    (i1 << 1) | i0
}

fn hilbert_bbox(r: &NodeItem, hilbert_max: u32, extent: &NodeItem) -> u32 {
    // calculate bbox center and scale to hilbert_max
    let x = (hilbert_max as f64 * ((r.min_x + r.max_x) / 2.0 - extent.min_x) / extent.width())
        .floor() as u32;
    let y = (hilbert_max as f64 * ((r.min_y + r.max_y) / 2.0 - extent.min_y) / extent.height())
        .floor() as u32;
    hilbert(x, y)
}

pub fn hilbert_sort(items: &mut [NodeItem], extent: &NodeItem) {
    items.sort_by(|a, b| {
        let ha = hilbert_bbox(a, HILBERT_MAX, extent);
        let hb = hilbert_bbox(b, HILBERT_MAX, extent);
        hb.partial_cmp(&ha).unwrap() // ha > hb
    });
}

pub fn calc_extent(nodes: &[NodeItem]) -> NodeItem {
    nodes.iter().fold(NodeItem::create(0), |mut a, b| {
        a.expand(b);
        a
    })
}

/// Packed Hilbert R-Tree
pub struct PackedRTree {
    extent: NodeItem,
    node_items: Vec<NodeItem>,
    num_leaf_nodes: usize,
    branching_factor: u16,
    level_bounds: Vec<Range<usize>>,
}

impl PackedRTree {
    pub const DEFAULT_NODE_SIZE: u16 = 16;

    fn init(&mut self, node_size: u16) -> Result<(), Error> {
        assert!(node_size >= 2, "Node size must be at least 2");
        assert!(self.num_leaf_nodes > 0, "Cannot create empty tree");
        self.branching_factor = node_size.clamp(2u16, 65535u16);
        self.level_bounds =
            PackedRTree::generate_level_bounds(self.num_leaf_nodes, self.branching_factor);
        let num_nodes = self
            .level_bounds
            .first()
            .expect("RTree has at least one level when node_size >= 2 and num_items > 0")
            .end;
        self.node_items = vec![NodeItem::create(0); num_nodes]; // Quite slow!
        Ok(())
    }

    fn generate_level_bounds(num_items: usize, node_size: u16) -> Vec<Range<usize>> {
        assert!(node_size >= 2, "Node size must be at least 2");
        assert!(num_items > 0, "Cannot create empty tree");
        assert!(
            num_items <= usize::MAX - ((num_items / node_size as usize) * 2),
            "Number of items too large"
        );

        // number of nodes per level in bottom-up order
        let mut level_num_nodes: Vec<usize> = Vec::new();
        let mut n = num_items;
        let mut num_nodes = n;
        level_num_nodes.push(n);
        loop {
            n = n.div_ceil(node_size as usize);
            num_nodes += n;
            level_num_nodes.push(n);
            if n == 1 {
                break;
            }
        }
        // bounds per level in reversed storage order (top-down)
        let mut level_offsets: Vec<usize> = Vec::with_capacity(level_num_nodes.len());
        n = num_nodes;
        for size in &level_num_nodes {
            level_offsets.push(n - size);
            n -= size;
        }
        let mut level_bounds = Vec::with_capacity(level_num_nodes.len());
        for i in 0..level_num_nodes.len() {
            level_bounds.push(level_offsets[i]..level_offsets[i] + level_num_nodes[i]);
        }
        level_bounds
    }

    fn generate_nodes(&mut self) {
        for level in 0..self.level_bounds.len() - 1 {
            let children_level = &self.level_bounds[level];
            let parent_level = &self.level_bounds[level + 1];

            let mut parent_idx = parent_level.start;
            let mut child_idx = children_level.start;
            while child_idx < children_level.end {
                let mut parent_node = NodeItem::create(child_idx as u64);
                for _j in 0..self.branching_factor {
                    if child_idx >= children_level.end {
                        break;
                    }
                    parent_node.expand(&self.node_items[child_idx]);
                    child_idx += 1;
                }
                self.node_items[parent_idx] = parent_node;
                parent_idx += 1;
            }
        }
    }

    fn read_data(&mut self, data: impl Read) -> Result<(), Error> {
        read_node_vec(&mut self.node_items, data)?;
        for node in &self.node_items {
            self.extent.expand(node)
        }
        Ok(())
    }

    #[cfg(feature = "http")]
    async fn read_http<T: AsyncHttpRangeClient>(
        &mut self,
        client: &mut AsyncBufferedHttpRangeClient<T>,
        index_begin: usize,
    ) -> Result<(), Error> {
        let min_req_size = self.size(); // read full index at once
        let mut pos = index_begin;
        for i in 0..self.num_nodes() {
            let bytes = client
                .min_req_size(min_req_size)
                .get_range(pos, size_of::<NodeItem>())
                .await?;
            let n = NodeItem::from_bytes(bytes)?;
            self.extent.expand(&n);
            self.node_items[i] = n;
            pos += size_of::<NodeItem>();
        }
        Ok(())
    }

    fn num_nodes(&self) -> usize {
        self.node_items.len()
    }

    pub fn build(
        nodes: &[NodeItem],
        extent: &NodeItem,
        node_size: u16,
    ) -> Result<PackedRTree, Error> {
        let mut tree = PackedRTree {
            extent: extent.clone(),
            node_items: Vec::new(),
            num_leaf_nodes: nodes.len(),
            branching_factor: 0,
            level_bounds: Vec::new(),
        };
        tree.init(node_size)?;
        let num_nodes = tree.num_nodes();
        for (i, node) in nodes.iter().take(tree.num_leaf_nodes).cloned().enumerate() {
            tree.node_items[num_nodes - tree.num_leaf_nodes + i] = node;
        }
        tree.generate_nodes();
        Ok(tree)
    }

    pub fn from_buf(
        data: impl Read,
        num_items: usize,
        node_size: u16,
    ) -> Result<PackedRTree, Error> {
        let node_size = node_size.clamp(2u16, 65535u16);
        let level_bounds = PackedRTree::generate_level_bounds(num_items, node_size);
        let num_nodes = level_bounds
            .first()
            .expect("RTree has at least one level when node_size >= 2 and num_items > 0")
            .end;
        let mut tree = PackedRTree {
            extent: NodeItem::create(0),
            node_items: Vec::with_capacity(num_nodes),
            num_leaf_nodes: num_items,
            branching_factor: node_size,
            level_bounds,
        };
        tree.read_data(data)?;
        Ok(tree)
    }

    #[cfg(feature = "http")]
    pub async fn from_http<T: AsyncHttpRangeClient>(
        client: &mut AsyncBufferedHttpRangeClient<T>,
        index_begin: usize,
        num_items: usize,
        node_size: u16,
    ) -> Result<PackedRTree, Error> {
        let mut tree = PackedRTree {
            extent: NodeItem::create(0),
            node_items: Vec::new(),
            num_leaf_nodes: num_items,
            branching_factor: 0,
            level_bounds: Vec::new(),
        };
        tree.init(node_size)?;
        tree.read_http(client, index_begin).await?;
        Ok(tree)
    }

    /// Search the R-Tree using a specific query type
    pub fn search(&self, query: Query) -> Result<Vec<SearchResultItem>, Error> {
        let leaf_nodes_offset = self
            .level_bounds
            .first()
            .expect("RTree has at least one level when node_size >= 2 and num_items > 0")
            .start;

        match query {
            Query::BBox(min_x, min_y, max_x, max_y) => {
                // Standard bounding box query
                let bounds = NodeItem::bounds(min_x, min_y, max_x, max_y);
                let mut results = Vec::new();
                let mut queue = VecDeque::new();
                queue.push_back((0, self.level_bounds.len() - 1));

                while let Some(next) = queue.pop_front() {
                    let node_index = next.0;
                    let level = next.1;
                    let is_leaf_node = node_index >= self.num_nodes() - self.num_leaf_nodes;
                    // find the end index of the node
                    let end = min(
                        node_index + self.branching_factor as usize,
                        self.level_bounds[level].end,
                    );
                    // search through child nodes
                    for pos in node_index..end {
                        let node_item = &self.node_items[pos];
                        if !bounds.intersects(node_item) {
                            continue;
                        }
                        if is_leaf_node {
                            results.push(SearchResultItem {
                                offset: node_item.offset as usize,
                                index: pos - leaf_nodes_offset,
                            });
                        } else {
                            queue.push_back((node_item.offset as usize, level - 1));
                        }
                    }
                }
                Ok(results)
            }
            Query::PointIntersects(x, y) => {
                // Point intersection query - find all bboxes that contain the point
                // Create a point as a degenerate bbox
                let point_bounds = NodeItem::bounds(x, y, x, y);
                let mut results = Vec::new();
                let mut queue = VecDeque::new();
                queue.push_back((0, self.level_bounds.len() - 1));

                while let Some(next) = queue.pop_front() {
                    let node_index = next.0;
                    let level = next.1;
                    let is_leaf_node = node_index >= self.num_nodes() - self.num_leaf_nodes;
                    // find the end index of the node
                    let end = min(
                        node_index + self.branching_factor as usize,
                        self.level_bounds[level].end,
                    );
                    // search through child nodes
                    for pos in node_index..end {
                        let node_item = &self.node_items[pos];
                        if !node_item.contains_point(x, y) {
                            continue;
                        }
                        if is_leaf_node {
                            results.push(SearchResultItem {
                                offset: node_item.offset as usize,
                                index: pos - leaf_nodes_offset,
                            });
                        } else {
                            queue.push_back((node_item.offset as usize, level - 1));
                        }
                    }
                }
                Ok(results)
            }
            Query::PointNearest(x, y) => {
                // Nearest neighbor query
                // We use a priority queue to visit nodes in order of minimum distance
                use std::cmp::Reverse;
                use std::collections::BinaryHeap;

                #[derive(PartialEq)]
                struct QueueItem {
                    distance: f64,
                    node_index: usize,
                    level: usize,
                }

                impl Eq for QueueItem {}

                impl PartialOrd for QueueItem {
                    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                        self.distance.partial_cmp(&other.distance)
                    }
                }

                impl Ord for QueueItem {
                    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
                    }
                }

                let mut nearest: Option<(f64, SearchResultItem)> = None;
                let mut queue = BinaryHeap::new();

                // Start with the root node
                queue.push(Reverse(QueueItem {
                    distance: 0.0,
                    node_index: 0,
                    level: self.level_bounds.len() - 1,
                }));

                while let Some(Reverse(next)) = queue.pop() {
                    // If we have a nearest item and its distance is less than this node's,
                    // we can stop searching
                    if let Some((best_dist, _)) = nearest {
                        if next.distance > best_dist {
                            break;
                        }
                    }

                    let node_index = next.node_index;
                    let level = next.level;
                    let is_leaf_node = node_index >= self.num_nodes() - self.num_leaf_nodes;
                    // find the end index of the node
                    let end = min(
                        node_index + self.branching_factor as usize,
                        self.level_bounds[level].end,
                    );

                    // search through child nodes
                    for pos in node_index..end {
                        let node_item = &self.node_items[pos];
                        let dist = node_item.min_distance_squared(x, y);

                        // If we have a nearest item, only consider this node if it's closer
                        if let Some((best_dist, _)) = nearest {
                            if dist >= best_dist {
                                continue;
                            }
                        }

                        if is_leaf_node {
                            // Update nearest if this leaf node is closer
                            let result = SearchResultItem {
                                offset: node_item.offset as usize,
                                index: pos - leaf_nodes_offset,
                            };

                            // For leaf nodes, use centroid distance as the final measure
                            let centroid_dist = node_item.centroid_distance_squared(x, y);

                            match nearest {
                                None => nearest = Some((centroid_dist, result)),
                                Some((best_dist, _)) if centroid_dist < best_dist => {
                                    nearest = Some((centroid_dist, result))
                                }
                                _ => {}
                            }
                        } else {
                            // Add this node to the queue with its minimum distance
                            queue.push(Reverse(QueueItem {
                                distance: dist,
                                node_index: node_item.offset as usize,
                                level: level - 1,
                            }));
                        }
                    }
                }

                // Return the nearest item, or empty vector if none found
                Ok(nearest.map(|(_, item)| vec![item]).unwrap_or_default())
            }
        }
    }

    /// Search the R-Tree using a specific query type with streaming
    pub fn stream_search<R: Read + Seek>(
        data: &mut R,
        num_items: usize,
        node_size: u16,
        query: Query,
    ) -> Result<Vec<SearchResultItem>, Error> {
        let level_bounds = PackedRTree::generate_level_bounds(num_items, node_size);
        let Range {
            start: leaf_nodes_offset,
            end: num_nodes,
        } = level_bounds
            .first()
            .expect("RTree has at least one level when node_size >= 2 and num_items > 0");

        // current position must be start of index
        let index_base = data.stream_position()?;

        match query {
            Query::BBox(min_x, min_y, max_x, max_y) => {
                let bounds = NodeItem::bounds(min_x, min_y, max_x, max_y);

                // use ordered search queue to make index traversal in sequential order
                let mut queue = VecDeque::new();
                queue.push_back((0, level_bounds.len() - 1));
                let mut results = Vec::new();

                while let Some(next) = queue.pop_front() {
                    let node_index = next.0;
                    let level = next.1;
                    let is_leaf_node = node_index >= num_nodes - num_items;
                    // find the end index of the node
                    let end = min(node_index + node_size as usize, level_bounds[level].end);
                    let length = end - node_index;
                    let node_items = read_node_items(data, index_base, node_index, length)?;
                    // search through child nodes
                    for pos in node_index..end {
                        let node_pos = pos - node_index;
                        let node_item = &node_items[node_pos];
                        if !bounds.intersects(node_item) {
                            continue;
                        }
                        if is_leaf_node {
                            let index = pos - leaf_nodes_offset;
                            let offset = node_item.offset as usize;
                            results.push(SearchResultItem { offset, index });
                        } else {
                            let offset = node_item.offset as usize;
                            let prev_level = level - 1;
                            queue.push_back((offset, prev_level));
                        }
                    }
                }

                // Skip rest of index
                data.seek(SeekFrom::Start(
                    index_base + (num_nodes * size_of::<NodeItem>()) as u64,
                ))?;
                Ok(results)
            }
            Query::PointIntersects(x, y) => {
                // use ordered search queue to make index traversal in sequential order
                let mut queue = VecDeque::new();
                queue.push_back((0, level_bounds.len() - 1));
                let mut results = Vec::new();

                while let Some(next) = queue.pop_front() {
                    let node_index = next.0;
                    let level = next.1;
                    let is_leaf_node = node_index >= num_nodes - num_items;
                    // find the end index of the node
                    let end = min(node_index + node_size as usize, level_bounds[level].end);
                    let length = end - node_index;
                    let node_items = read_node_items(data, index_base, node_index, length)?;
                    // search through child nodes
                    for pos in node_index..end {
                        let node_pos = pos - node_index;
                        let node_item = &node_items[node_pos];
                        if !node_item.contains_point(x, y) {
                            continue;
                        }
                        if is_leaf_node {
                            let index = pos - leaf_nodes_offset;
                            let offset = node_item.offset as usize;
                            results.push(SearchResultItem { offset, index });
                        } else {
                            let offset = node_item.offset as usize;
                            let prev_level = level - 1;
                            queue.push_back((offset, prev_level));
                        }
                    }
                }

                // Skip rest of index
                data.seek(SeekFrom::Start(
                    index_base + (num_nodes * size_of::<NodeItem>()) as u64,
                ))?;
                Ok(results)
            }
            Query::PointNearest(x, y) => {
                use std::cmp::Reverse;
                use std::collections::BinaryHeap;

                #[derive(PartialEq)]
                struct QueueItem {
                    distance: f64,
                    node_index: usize,
                    level: usize,
                }

                impl Eq for QueueItem {}

                impl PartialOrd for QueueItem {
                    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                        self.distance.partial_cmp(&other.distance)
                    }
                }

                impl Ord for QueueItem {
                    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
                    }
                }

                let mut nearest: Option<(f64, SearchResultItem)> = None;
                let mut queue = BinaryHeap::new();

                // Start with the root node
                queue.push(Reverse(QueueItem {
                    distance: 0.0,
                    node_index: 0,
                    level: level_bounds.len() - 1,
                }));

                while let Some(Reverse(next)) = queue.pop() {
                    // If we have a nearest item and its distance is less than this node's,
                    // we can stop searching
                    if let Some((best_dist, _)) = nearest {
                        if next.distance > best_dist {
                            break;
                        }
                    }

                    let node_index = next.node_index;
                    let level = next.level;
                    let is_leaf_node = node_index >= num_nodes - num_items;

                    // Get the node items
                    let end = min(node_index + node_size as usize, level_bounds[level].end);
                    let length = end - node_index;
                    let node_items = read_node_items(data, index_base, node_index, length)?;

                    // search through child nodes
                    for pos in node_index..end {
                        let node_pos = pos - node_index;
                        let node_item = &node_items[node_pos];
                        let dist = node_item.min_distance_squared(x, y);

                        // If we have a nearest item, only consider this node if it's closer
                        if let Some((best_dist, _)) = nearest {
                            if dist >= best_dist {
                                continue;
                            }
                        }

                        if is_leaf_node {
                            // Update nearest if this leaf node is closer
                            let index = pos - leaf_nodes_offset;
                            let offset = node_item.offset as usize;
                            let result = SearchResultItem { offset, index };

                            // For leaf nodes, use centroid distance as the final measure
                            let centroid_dist = node_item.centroid_distance_squared(x, y);

                            match nearest {
                                None => nearest = Some((centroid_dist, result)),
                                Some((best_dist, _)) if centroid_dist < best_dist => {
                                    nearest = Some((centroid_dist, result))
                                }
                                _ => {}
                            }
                        } else {
                            // Add this node to the queue with its minimum distance
                            queue.push(Reverse(QueueItem {
                                distance: dist,
                                node_index: node_item.offset as usize,
                                level: level - 1,
                            }));
                        }
                    }
                }

                // Skip rest of index
                data.seek(SeekFrom::Start(
                    index_base + (num_nodes * size_of::<NodeItem>()) as u64,
                ))?;

                // Return the nearest item, or empty vector if none found
                Ok(nearest.map(|(_, item)| vec![item]).unwrap_or_default())
            }
        }
    }

    pub fn size(&self) -> usize {
        self.num_nodes() * size_of::<NodeItem>()
    }

    pub fn index_size(num_items: usize, node_size: u16) -> usize {
        assert!(node_size >= 2, "Node size must be at least 2");
        assert!(num_items > 0, "Cannot create empty tree");
        let node_size_min = node_size.clamp(2, 65535) as usize;
        // limit so that resulting size in bytes can be represented by uint64_t
        // assert!(
        //     num_items <= 1 << 56,
        //     "Number of items must be less than 2^56"
        // );
        let mut n = num_items;
        let mut num_nodes = n;
        loop {
            n = n.div_ceil(node_size_min);
            num_nodes += n;
            if n == 1 {
                break;
            }
        }
        num_nodes * size_of::<NodeItem>()
    }

    /// Write all index nodes
    pub fn stream_write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        for item in &self.node_items {
            item.write(out)?;
        }
        Ok(())
    }

    pub fn extent(&self) -> NodeItem {
        self.extent.clone()
    }

    #[cfg(feature = "http")]
    #[allow(clippy::too_many_arguments)]
    pub async fn http_stream_search<T: AsyncHttpRangeClient>(
        client: &mut AsyncBufferedHttpRangeClient<T>,
        index_begin: usize,
        attr_index_size: usize,
        num_items: usize,
        branching_factor: u16,
        query: Query,
        combine_request_threshold: usize,
    ) -> Result<Vec<HttpSearchResultItem>, Error> {
        use tracing::debug;

        if num_items == 0 {
            return Ok(vec![]);
        }

        let level_bounds = PackedRTree::generate_level_bounds(num_items, branching_factor);
        let feature_begin =
            index_begin + attr_index_size + PackedRTree::index_size(num_items, branching_factor);

        match query {
            Query::BBox(min_x, min_y, max_x, max_y) => {
                debug!("http_stream_search - index_begin: {index_begin}, feature_begin: {feature_begin} num_items: {num_items}, branching_factor: {branching_factor}, level_bounds: {level_bounds:?}, GPS bounds:[({min_x}, {min_y}), ({max_x},{max_y})]");

                let bounds = NodeItem::bounds(min_x, min_y, max_x, max_y);

                #[derive(Debug, PartialEq, Eq)]
                struct NodeRange {
                    level: usize,
                    nodes: Range<usize>,
                }

                let mut queue = VecDeque::new();
                queue.push_back(NodeRange {
                    nodes: 0..1,
                    level: level_bounds.len() - 1,
                });
                let mut results = Vec::new();

                while let Some(node_range) = queue.pop_front() {
                    debug!("next: {node_range:?}. {} items left in queue", queue.len());
                    let node_items =
                        read_http_node_items(client, index_begin, &node_range.nodes).await?;
                    for (node_pos, node_item) in node_items.iter().enumerate() {
                        if !bounds.intersects(node_item) {
                            continue;
                        }

                        if node_range.level == 0 {
                            // leaf node
                            let start = feature_begin + node_item.offset as usize;
                            if let Some(next_node_item) = &node_items.get(node_pos + 1) {
                                let end = feature_begin + next_node_item.offset as usize;
                                results.push(HttpSearchResultItem {
                                    range: HttpRange::Range(start..end),
                                });
                            } else {
                                debug_assert_eq!(node_pos, num_items - 1);
                                results.push(HttpSearchResultItem {
                                    range: HttpRange::RangeFrom(start..),
                                });
                            }
                        } else {
                            let children_level = node_range.level - 1;
                            let mut children_nodes = node_item.offset as usize
                                ..(node_item.offset + branching_factor as u64) as usize;
                            if children_level == 0 {
                                // These children are leaf nodes.
                                //
                                // We can right-size our feature requests if we know the size of each feature.
                                //
                                // To infer the length of *this* feature, we need the start of the *next*
                                // feature, so we get an extra node here.
                                children_nodes.end += 1;
                            }
                            // always stay within level's bounds
                            children_nodes.end =
                                min(children_nodes.end, level_bounds[children_level].end);

                            let children_range = NodeRange {
                                nodes: children_nodes,
                                level: children_level,
                            };

                            let Some(tail) = queue.back_mut() else {
                                debug!("Adding new request onto empty queue: {children_range:?}");
                                queue.push_back(children_range);
                                continue;
                            };

                            if tail.level != children_level {
                                debug!("Adding new request for new level: {children_range:?} (existing queue tail: {tail:?})");
                                queue.push_back(children_range);
                                continue;
                            }

                            let wasted_bytes = {
                                if children_range.nodes.start >= tail.nodes.end {
                                    (children_range.nodes.start - tail.nodes.end)
                                        * size_of::<NodeItem>()
                                } else {
                                    // To compute feature size, we fetch an extra leaf node, but computing
                                    // wasted_bytes for adjacent ranges will overflow in that case, so
                                    // we skip that computation.
                                    //
                                    // But let's make sure we're in the state we think we are:
                                    debug_assert_eq!(
                                        children_range.nodes.start + 1,
                                        tail.nodes.end,
                                        "we only ever fetch one extra node"
                                    );
                                    debug_assert_eq!(
                                        children_level, 0,
                                        "extra node fetching only happens with leaf nodes"
                                    );
                                    0
                                }
                            };
                            if wasted_bytes > combine_request_threshold {
                                debug!("Adding new request for: {children_range:?} rather than merging with distant NodeRange: {tail:?} (would waste {wasted_bytes} bytes)");
                                queue.push_back(children_range);
                                continue;
                            }

                            // Merge the ranges to avoid an extra request
                            debug!("Extending existing request {tail:?} with nearby children: {:?} (wastes {wasted_bytes} bytes)", &children_range.nodes);
                            tail.nodes.end = children_range.nodes.end;
                        }
                    }
                }
                Ok(results)
            }
            Query::PointIntersects(x, y) => {
                debug!("http_stream_search point intersects - index_begin: {index_begin}, feature_begin: {feature_begin} num_items: {num_items}, branching_factor: {branching_factor}, level_bounds: {level_bounds:?}, point: ({x}, {y})");

                #[derive(Debug, PartialEq, Eq)]
                struct NodeRange {
                    level: usize,
                    nodes: Range<usize>,
                }

                let mut queue = VecDeque::new();
                queue.push_back(NodeRange {
                    nodes: 0..1,
                    level: level_bounds.len() - 1,
                });
                let mut results = Vec::new();

                while let Some(node_range) = queue.pop_front() {
                    debug!("next: {node_range:?}. {} items left in queue", queue.len());
                    let node_items =
                        read_http_node_items(client, index_begin, &node_range.nodes).await?;
                    for (node_pos, node_item) in node_items.iter().enumerate() {
                        if !node_item.contains_point(x, y) {
                            continue;
                        }

                        if node_range.level == 0 {
                            // leaf node
                            let start = feature_begin + node_item.offset as usize;
                            if let Some(next_node_item) = &node_items.get(node_pos + 1) {
                                let end = feature_begin + next_node_item.offset as usize;
                                results.push(HttpSearchResultItem {
                                    range: HttpRange::Range(start..end),
                                });
                            } else {
                                debug_assert_eq!(node_pos, num_items - 1);
                                results.push(HttpSearchResultItem {
                                    range: HttpRange::RangeFrom(start..),
                                });
                            }
                        } else {
                            let children_level = node_range.level - 1;
                            let mut children_nodes = node_item.offset as usize
                                ..(node_item.offset + branching_factor as u64) as usize;
                            if children_level == 0 {
                                children_nodes.end += 1;
                            }
                            children_nodes.end =
                                min(children_nodes.end, level_bounds[children_level].end);

                            let children_range = NodeRange {
                                nodes: children_nodes,
                                level: children_level,
                            };

                            let Some(tail) = queue.back_mut() else {
                                debug!("Adding new request onto empty queue: {children_range:?}");
                                queue.push_back(children_range);
                                continue;
                            };

                            if tail.level != children_level {
                                debug!("Adding new request for new level: {children_range:?} (existing queue tail: {tail:?})");
                                queue.push_back(children_range);
                                continue;
                            }

                            let wasted_bytes = {
                                if children_range.nodes.start >= tail.nodes.end {
                                    (children_range.nodes.start - tail.nodes.end)
                                        * size_of::<NodeItem>()
                                } else {
                                    debug_assert_eq!(
                                        children_range.nodes.start + 1,
                                        tail.nodes.end,
                                        "we only ever fetch one extra node"
                                    );
                                    debug_assert_eq!(
                                        children_level, 0,
                                        "extra node fetching only happens with leaf nodes"
                                    );
                                    0
                                }
                            };
                            if wasted_bytes > combine_request_threshold {
                                debug!("Adding new request for: {children_range:?} rather than merging with distant NodeRange: {tail:?} (would waste {wasted_bytes} bytes)");
                                queue.push_back(children_range);
                                continue;
                            }

                            tail.nodes.end = children_range.nodes.end;
                        }
                    }
                }
                Ok(results)
            }
            Query::PointNearest(x, y) => {
                debug!("http_stream_search nearest neighbor - index_begin: {index_begin}, feature_begin: {feature_begin} num_items: {num_items}, branching_factor: {branching_factor}, level_bounds: {level_bounds:?}, point: ({x}, {y})");

                use std::cmp::Reverse;
                use std::collections::BinaryHeap;

                #[derive(PartialEq)]
                struct QueueItem {
                    distance: f64,
                    level: usize,
                    nodes: Range<usize>,
                }

                impl Eq for QueueItem {}

                impl PartialOrd for QueueItem {
                    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                        self.distance.partial_cmp(&other.distance)
                    }
                }

                impl Ord for QueueItem {
                    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
                    }
                }

                let mut nearest: Option<(f64, HttpSearchResultItem)> = None;
                let mut queue = BinaryHeap::new();

                // Start with the root node
                queue.push(Reverse(QueueItem {
                    distance: 0.0,
                    nodes: 0..1,
                    level: level_bounds.len() - 1,
                }));

                while let Some(Reverse(next)) = queue.pop() {
                    // If we have a nearest item and its distance is less than this node's,
                    // we can stop searching
                    if let Some((best_dist, _)) = nearest {
                        if next.distance > best_dist {
                            break;
                        }
                    }

                    debug!(
                        "next: node dist: {}, level: {}, nodes: {:?}, {} items left in queue",
                        next.distance,
                        next.level,
                        next.nodes,
                        queue.len()
                    );
                    let node_items = read_http_node_items(client, index_begin, &next.nodes).await?;

                    for (node_pos, node_item) in node_items.iter().enumerate() {
                        let dist = node_item.min_distance_squared(x, y);

                        // If we have a nearest item, only consider this node if it's closer
                        if let Some((best_dist, _)) = nearest {
                            if dist >= best_dist {
                                continue;
                            }
                        }

                        if next.level == 0 {
                            // Leaf node - use centroid distance as the final measure
                            let centroid_dist = node_item.centroid_distance_squared(x, y);

                            // Create range for the result
                            let start = feature_begin + node_item.offset as usize;
                            let result = if let Some(next_node_item) = &node_items.get(node_pos + 1)
                            {
                                let end = feature_begin + next_node_item.offset as usize;
                                HttpSearchResultItem {
                                    range: HttpRange::Range(start..end),
                                }
                            } else {
                                debug_assert_eq!(node_pos, num_items - 1);
                                HttpSearchResultItem {
                                    range: HttpRange::RangeFrom(start..),
                                }
                            };

                            match nearest {
                                None => nearest = Some((centroid_dist, result)),
                                Some((best_dist, _)) if centroid_dist < best_dist => {
                                    nearest = Some((centroid_dist, result))
                                }
                                _ => {}
                            }
                        } else {
                            // Not a leaf node - add children to the queue
                            let children_level = next.level - 1;
                            let mut children_nodes = node_item.offset as usize
                                ..(node_item.offset + branching_factor as u64) as usize;

                            if children_level == 0 {
                                children_nodes.end += 1;
                            }

                            // Always stay within level bounds
                            children_nodes.end =
                                min(children_nodes.end, level_bounds[children_level].end);

                            queue.push(Reverse(QueueItem {
                                distance: dist,
                                nodes: children_nodes,
                                level: children_level,
                            }));
                        }
                    }
                }

                // Return the nearest item, or empty vector if none found
                Ok(nearest.map(|(_, item)| vec![item]).unwrap_or_default())
            }
        }
    }
}

#[cfg(feature = "http")]
pub mod http {
    use std::ops::{Range, RangeFrom};

    /// Byte range within a file. Suitable for an HTTP Range request.
    #[derive(Debug, Clone)]
    pub enum HttpRange {
        Range(Range<usize>),
        RangeFrom(RangeFrom<usize>),
    }

    impl HttpRange {
        pub fn start(&self) -> usize {
            match self {
                Self::Range(range) => range.start,
                Self::RangeFrom(range) => range.start,
            }
        }

        pub fn end(&self) -> Option<usize> {
            match self {
                Self::Range(range) => Some(range.end),
                Self::RangeFrom(_) => None,
            }
        }

        pub fn with_end(self, end: Option<usize>) -> Self {
            match end {
                Some(end) => Self::Range(self.start()..end),
                None => Self::RangeFrom(self.start()..),
            }
        }

        pub fn length(&self) -> Option<usize> {
            match self {
                Self::Range(range) => Some(range.end - range.start),
                Self::RangeFrom(_) => None,
            }
        }
    }

    #[derive(Debug)]
    /// Bbox filter search result
    pub struct HttpSearchResultItem {
        /// Byte offset in feature data section
        pub range: HttpRange,
    }
}
#[cfg(feature = "http")]
pub(crate) use http::*;

#[cfg(test)]
mod tests {
    use super::*;
    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn tree_2items() -> Result<()> {
        let mut nodes = Vec::new();
        nodes.push(NodeItem::bounds(0.0, 0.0, 1.0, 1.0));
        nodes.push(NodeItem::bounds(2.0, 2.0, 3.0, 3.0));
        let extent = calc_extent(&nodes);
        assert_eq!(extent, NodeItem::bounds(0.0, 0.0, 3.0, 3.0));
        assert!(nodes[0].intersects(&NodeItem::bounds(0.0, 0.0, 1.0, 1.0)));
        assert!(nodes[1].intersects(&NodeItem::bounds(2.0, 2.0, 3.0, 3.0)));
        hilbert_sort(&mut nodes, &extent);
        let mut offset = 0;
        for node in &mut nodes {
            node.offset = offset;
            offset += size_of::<NodeItem>() as u64;
        }
        assert!(nodes[1].intersects(&NodeItem::bounds(0.0, 0.0, 1.0, 1.0)));
        assert!(nodes[0].intersects(&NodeItem::bounds(2.0, 2.0, 3.0, 3.0)));
        let tree = PackedRTree::build(&nodes, &extent, PackedRTree::DEFAULT_NODE_SIZE)?;
        let list = tree.search(Query::BBox(0.0, 0.0, 1.0, 1.0))?;
        assert_eq!(list.len(), 1);
        assert!(nodes[list[0].index].intersects(&NodeItem::bounds(0.0, 0.0, 1.0, 1.0)));
        Ok(())
    }

    #[test]
    fn test_point_intersects_query() -> Result<()> {
        // Create a simple tree with some test nodes
        let mut nodes = Vec::new();
        nodes.push(NodeItem::bounds(0.0, 0.0, 1.0, 1.0)); // Node 0: Small box at origin
        nodes.push(NodeItem::bounds(2.0, 2.0, 3.0, 3.0)); // Node 1: Small box at (2,2)
        nodes.push(NodeItem::bounds(5.0, 5.0, 10.0, 10.0)); // Node 2: Larger box
        nodes.push(NodeItem::bounds(-10.0, -10.0, 10.0, 10.0)); // Node 3: Large box covering others

        let extent = calc_extent(&nodes);
        hilbert_sort(&mut nodes, &extent);

        // Set offsets to match node indices
        let mut offset = 0;
        for node in &mut nodes {
            node.offset = offset;
            offset += size_of::<NodeItem>() as u64;
        }

        let tree = PackedRTree::build(&nodes, &extent, PackedRTree::DEFAULT_NODE_SIZE)?;

        // Test point (0.5, 0.5) should be in the first box and the large box
        let results = tree.search(Query::PointIntersects(0.5, 0.5))?;
        assert_eq!(results.len(), 2, "Point (0.5, 0.5) should be in 2 bboxes");

        // Test point at corner (1.0, 1.0) should be on the boundary of the first box and the large box
        let results = tree.search(Query::PointIntersects(1.0, 1.0))?;
        assert_eq!(results.len(), 2, "Point (1.0, 1.0) should be in 2 bboxes");

        // Test point (7.5, 7.5) should be in the larger box and the large box
        let results = tree.search(Query::PointIntersects(7.5, 7.5))?;
        assert_eq!(results.len(), 2, "Point (7.5, 7.5) should be in 2 bboxes");

        // Test point (20.0, 20.0) should be in no boxes
        let results = tree.search(Query::PointIntersects(20.0, 20.0))?;
        assert_eq!(results.len(), 0, "Point (20.0, 20.0) should be in 0 bboxes");

        // Test with streaming query
        let mut tree_data: Vec<u8> = Vec::new();
        tree.stream_write(&mut tree_data)?;

        let mut reader = Cursor::new(&tree_data);
        let results = PackedRTree::stream_search(
            &mut reader,
            nodes.len(),
            PackedRTree::DEFAULT_NODE_SIZE,
            Query::PointIntersects(0.5, 0.5),
        )?;
        assert_eq!(
            results.len(),
            2,
            "Stream query: Point (0.5, 0.5) should be in 2 bboxes"
        );

        Ok(())
    }

    #[test]
    fn test_nearest_neighbor_query() -> Result<()> {
        // Create a simple tree with some test nodes
        let mut nodes = Vec::new();
        nodes.push(NodeItem::bounds(0.0, 0.0, 1.0, 1.0)); // Node 0: Small box at origin
        nodes.push(NodeItem::bounds(2.0, 2.0, 3.0, 3.0)); // Node 1: Small box at (2,2)
        nodes.push(NodeItem::bounds(5.0, 5.0, 10.0, 10.0)); // Node 2: Larger box
        nodes.push(NodeItem::bounds(-10.0, -10.0, -5.0, -5.0)); // Node 3: Box in negative quadrant

        let extent = calc_extent(&nodes);
        hilbert_sort(&mut nodes, &extent);

        // Set offsets to match node indices
        let mut offset = 0;
        for node in &mut nodes {
            node.offset = offset;
            offset += size_of::<NodeItem>() as u64;
        }

        let tree = PackedRTree::build(&nodes, &extent, PackedRTree::DEFAULT_NODE_SIZE)?;

        // Test nearest to point (0.0, 0.0) should be the box at origin
        let results = tree.search(Query::PointNearest(0.0, 0.0))?;
        assert_eq!(results.len(), 1, "Should find exactly one nearest node");

        // The node at (0.0, 0.0) has the smallest distance to (0.0, 0.0)
        let node0_centroid = (0.5, 0.5); // Center of node 0
        let node3_centroid = (-7.5, -7.5); // Center of node 3

        let dist_to_node0 = (node0_centroid.0 - 0.0) * (node0_centroid.0 - 0.0)
            + (node0_centroid.1 - 0.0) * (node0_centroid.1 - 0.0);
        let dist_to_node3 = (node3_centroid.0 - 0.0) * (node3_centroid.0 - 0.0)
            + (node3_centroid.1 - 0.0) * (node3_centroid.1 - 0.0);

        assert!(
            dist_to_node0 < dist_to_node3,
            "Node 0 should be closer than Node 3"
        );

        // Test nearest to point (4.0, 4.0) should be the box at (2.0, 2.0, 3.0, 3.0) or (5.0, 5.0, 10.0, 10.0)
        // Let's calculate which one should be closer:
        let node1_centroid = (2.5, 2.5); // Center of node 1
        let node2_centroid = (7.5, 7.5); // Center of node 2

        let dist_to_node1: f64 = (node1_centroid.0 - 4.0) * (node1_centroid.0 - 4.0)
            + (node1_centroid.1 - 4.0) * (node1_centroid.1 - 4.0);
        let dist_to_node2: f64 = (node2_centroid.0 - 4.0) * (node2_centroid.0 - 4.0)
            + (node2_centroid.1 - 4.0) * (node2_centroid.1 - 4.0);

        let expected_closest_distance = dist_to_node1.min(dist_to_node2);

        // Get the result
        let results = tree.search(Query::PointNearest(4.0, 4.0))?;
        assert_eq!(results.len(), 1, "Should find exactly one nearest node");

        // Test with streaming query
        let mut tree_data: Vec<u8> = Vec::new();
        tree.stream_write(&mut tree_data)?;

        let mut reader = Cursor::new(&tree_data);
        let results = PackedRTree::stream_search(
            &mut reader,
            nodes.len(),
            PackedRTree::DEFAULT_NODE_SIZE,
            Query::PointNearest(0.0, 0.0),
        )?;
        assert_eq!(
            results.len(),
            1,
            "Stream query: Should find exactly one nearest node"
        );

        Ok(())
    }

    #[test]
    fn test_node_item_helper_methods() -> Result<()> {
        // Test contains_point
        let node = NodeItem::bounds(0.0, 0.0, 5.0, 5.0);

        assert!(
            node.contains_point(0.0, 0.0),
            "Origin point should be contained (boundary)"
        );
        assert!(
            node.contains_point(5.0, 5.0),
            "Corner point should be contained (boundary)"
        );
        assert!(
            node.contains_point(2.5, 2.5),
            "Center point should be contained"
        );
        assert!(
            !node.contains_point(-1.0, 2.5),
            "Point outside should not be contained"
        );
        assert!(
            !node.contains_point(6.0, 2.5),
            "Point outside should not be contained"
        );

        // Test min_distance_squared
        assert_eq!(
            node.min_distance_squared(2.5, 2.5),
            0.0,
            "Point inside should have zero distance"
        );
        assert_eq!(
            node.min_distance_squared(0.0, 0.0),
            0.0,
            "Point on boundary should have zero distance"
        );

        let dist_to_outside_point = node.min_distance_squared(7.0, 8.0);
        let expected_dist = (7.0 - 5.0) * (7.0 - 5.0) + (8.0 - 5.0) * (8.0 - 5.0); // Distance to closest point (5,5)
        assert_eq!(
            dist_to_outside_point, expected_dist,
            "Distance to outside point"
        );

        // Test centroid_distance_squared
        let centroid_x = 2.5; // (0.0 + 5.0) / 2.0
        let centroid_y = 2.5; // (0.0 + 5.0) / 2.0

        let dist_to_point = node.centroid_distance_squared(0.0, 0.0);
        let expected_dist =
            (0.0 - centroid_x) * (0.0 - centroid_x) + (0.0 - centroid_y) * (0.0 - centroid_y);
        assert_eq!(
            dist_to_point, expected_dist,
            "Centroid distance calculation"
        );

        Ok(())
    }

    #[test]
    fn tree_19items_roundtrip_stream_search() -> Result<()> {
        let mut nodes = vec![
            NodeItem::bounds(0.0, 0.0, 1.0, 1.0),
            NodeItem::bounds(2.0, 2.0, 3.0, 3.0),
            NodeItem::bounds(100.0, 100.0, 110.0, 110.0),
            NodeItem::bounds(101.0, 101.0, 111.0, 111.0),
            NodeItem::bounds(102.0, 102.0, 112.0, 112.0),
            NodeItem::bounds(103.0, 103.0, 113.0, 113.0),
            NodeItem::bounds(104.0, 104.0, 114.0, 114.0),
            NodeItem::bounds(10010.0, 10010.0, 10110.0, 10110.0),
            NodeItem::bounds(10010.0, 10010.0, 10110.0, 10110.0),
            NodeItem::bounds(10010.0, 10010.0, 10110.0, 10110.0),
            NodeItem::bounds(10010.0, 10010.0, 10110.0, 10110.0),
            NodeItem::bounds(10010.0, 10010.0, 10110.0, 10110.0),
            NodeItem::bounds(10010.0, 10010.0, 10110.0, 10110.0),
            NodeItem::bounds(10010.0, 10010.0, 10110.0, 10110.0),
            NodeItem::bounds(10010.0, 10010.0, 10110.0, 10110.0),
            NodeItem::bounds(10010.0, 10010.0, 10110.0, 10110.0),
            NodeItem::bounds(10010.0, 10010.0, 10110.0, 10110.0),
            NodeItem::bounds(10010.0, 10010.0, 10110.0, 10110.0),
            NodeItem::bounds(10010.0, 10010.0, 10110.0, 10110.0),
        ];

        let extent = calc_extent(&nodes);
        hilbert_sort(&mut nodes, &extent);
        let mut offset = 0;
        for node in &mut nodes {
            node.offset = offset;
            offset += size_of::<NodeItem>() as u64;
        }
        let tree = PackedRTree::build(&nodes, &extent, PackedRTree::DEFAULT_NODE_SIZE)?;
        let list = tree.search(Query::BBox(102.0, 102.0, 103.0, 103.0))?;
        assert_eq!(list.len(), 4);

        let indexes: Vec<usize> = list.iter().map(|item| item.index).collect();
        let expected: Vec<usize> = vec![13, 14, 15, 16];
        assert_eq!(indexes, expected);

        let mut tree_data: Vec<u8> = Vec::new();
        let res = tree.stream_write(&mut tree_data);
        assert!(res.is_ok());
        assert_eq!(tree_data.len(), (nodes.len() + 3) * size_of::<NodeItem>());
        assert_eq!(size_of::<NodeItem>(), 40);

        let tree2 = PackedRTree::from_buf(
            &mut &tree_data[..],
            nodes.len(),
            PackedRTree::DEFAULT_NODE_SIZE,
        )?;
        let list = tree2.search(Query::BBox(102.0, 102.0, 103.0, 103.0))?;
        assert_eq!(list.len(), 4);

        let indexes: Vec<usize> = list.iter().map(|item| item.index).collect();
        let expected: Vec<usize> = vec![13, 14, 15, 16];
        assert_eq!(indexes, expected);

        let mut reader = Cursor::new(&tree_data);
        let list = PackedRTree::stream_search(
            &mut reader,
            nodes.len(),
            PackedRTree::DEFAULT_NODE_SIZE,
            Query::BBox(102.0, 102.0, 103.0, 103.0),
        )?;
        assert_eq!(list.len(), 4);

        let indexes: Vec<usize> = list.iter().map(|item| item.index).collect();
        let expected: Vec<usize> = vec![13, 14, 15, 16];
        assert_eq!(indexes, expected);

        Ok(())
    }

    #[test]
    fn tree_100_000_items_in_denmark() -> Result<()> {
        use rand::distributions::{Distribution, Uniform};

        let unifx = Uniform::from(466379..708929);
        let unify = Uniform::from(6096801..6322352);
        let mut rng = rand::thread_rng();

        let mut nodes = Vec::new();
        for _ in 0..100000 {
            let x = unifx.sample(&mut rng) as f64;
            let y = unify.sample(&mut rng) as f64;
            nodes.push(NodeItem::bounds(x, y, x, y));
        }

        let extent = calc_extent(&nodes);
        hilbert_sort(&mut nodes, &extent);
        let tree = PackedRTree::build(&nodes, &extent, PackedRTree::DEFAULT_NODE_SIZE)?;
        let list = tree.search(Query::BBox(690407.0, 6063692.0, 811682.0, 6176467.0))?;

        for i in 0..list.len() {
            assert!(nodes[list[i].index]
                .intersects(&NodeItem::bounds(690407.0, 6063692.0, 811682.0, 6176467.0)));
        }

        let mut tree_data: Vec<u8> = Vec::new();
        let res = tree.stream_write(&mut tree_data);
        assert!(res.is_ok());

        let mut reader = Cursor::new(&tree_data);
        let list2 = PackedRTree::stream_search(
            &mut reader,
            nodes.len(),
            PackedRTree::DEFAULT_NODE_SIZE,
            Query::BBox(690407.0, 6063692.0, 811682.0, 6176467.0),
        )?;
        assert_eq!(list2.len(), list.len());
        for i in 0..list2.len() {
            assert!(nodes[list2[i].index]
                .intersects(&NodeItem::bounds(690407.0, 6063692.0, 811682.0, 6176467.0)));
        }
        Ok(())
    }
}

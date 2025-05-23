# Static B+Tree (S+Tree) Crate Overview

**Project:** Implement the `static-btree` Rust Crate
**Goal:** Create a Rust crate for a Static B+Tree (S+Tree) optimized for read performance.

## 1. Introduction

This document provides a comprehensive overview of the static-btree crate, detailing its internal structure, modules, and workflows for tree construction and search operations.

The implementation follows the principles described in the [Algorithmica S+Tree article](https://en.algorithmica.org/hpc/data-structures/s-tree/), utilizing an implicit Eytzinger layout for node addressing and storing the entire tree structure contiguously. The goal is to create a highly performant, read-optimized B+Tree suitable for large, static datasets, emphasizing cache efficiency and minimal memory usage during queries.

## 2. Core Concepts

* **Static:** Built once, read many times. No modifications after build.
* **Implicit Layout (Eytzinger):** Nodes stored contiguously, often level-by-level.
* **Packed Nodes:** Nodes are fully utilized (except potentially the last one per level) for better space and cache efficiency.
* **Read Optimization:** Designed for fast lookups and range scans by minimizing I/O (reading only needed nodes).
* **`Read + Seek` Abstraction:** Operates on standard Rust I/O traits, enabling use with files, memory, etc.

## 3. File Structure and Module Overview

The static-btree crate is organized into the following key modules:

| File/Module | Description |
|-------------|-------------|
| `lib.rs` | Entry point for the crate, re-exports public modules and types |
| `stree.rs` | Core implementation of the Static B+Tree (S+Tree) structure |
| `error.rs` | Error types and handling for the crate |
| `entry.rs` | Implementation of node entries (`NodeItem<K>` / `Entry<K>`) |
| `key.rs` | Trait definitions and implementations for key types |
| `mocked_http_range_client.rs` | Mock HTTP client for testing HTTP-based operations |
| `query/` | Query interface for higher-level operations (described in section 10) |

### Key Components

* **`Entry<K>`**: Represents an item in a tree node, containing a key and an offset.
* **`Stree<K>`**: The core tree structure, managing node items and tree layout.
* **`Key` trait**: Defines operations required for key types (comparison, serialization).
* **`HttpSearchResultItem`**: Represents search results when using HTTP-based querying.
* **`SearchResultItem`**: Represents search results when using direct memory or file access.

## 4. Terminology & Symbols

Before diving into the implementation details, let's define the key terminology and symbols used throughout the crate:

| Symbol | Meaning                          |
|--------|----------------------------------|
| `B`    | The number of pointers to children in a node. The number of items in a node is `B-1`. |
| `N`    | Total number of entries          |
| `H`    | Height of the tree               |
| `K`    | Key type implementing `Key`      |
| `O`    | Offset (`u64`) – value payload. This is the offset of the value in the payload. |
| `payload` | The buffer where actual data offsets are stored. When there are duplicates, the payload is the array of offsets of the duplicate values. |

## 5. Tree Structure

### 5.1 Node Layout

1. **Leaf Nodes**
   * Store **up to `B`-1 keys** followed by their offsets.
   * No explicit *next‑leaf* pointer is required – leaves are stored **contiguously**; the next leaf is `index + 1` except for the right‑most leaf.

2. **Internal Nodes**
   * Store keys and offsets. The offset is the offset index of the first (leftmost) item in the child node.
   * For the parent, its children nodes have smaller keys than the parent. All node items in the child node can be retrieved by the offset and the number of items in the child node (B-1).
   * Each key `i` equals the **smallest key in the `(i+1)`‑th child subtree** (standard B+ invariant).
   * No padding is required for the last child node. Tree is packed.

3. **Layer Packing**
   * Layers are written **top‑down (root → leaves)** in both memory & on‑disk representation.
   * Items in each layer are stored as single array. e.g. the root node is the first B-1 items, the next level is the next B items, etc. `[root node item1, root node item2, ..., root node itemB-1, child node item1, child node item2, ..., child node itemB-1]`.

## 6. Tree Construction Workflow

The static B+Tree construction follows a sequential process designed for efficient read-only operations:

### 6.1. Data Preparation

1. **Collection of Entries**:
   * Starts with a collection of `Entry<K>` items containing keys and their corresponding offsets.
   * Each entry represents a key-value pair, where the value is referenced by an offset.

2. **Duplicate Handling**:
   * Identify and handle duplicate keys.
   * For duplicate keys, offsets are stored in a separate payload area.
   * The tree itself stores only unique keys, with references to the payload for duplicates.

### 6.2. Tree Structure Calculation

1. **Level Bounds Calculation** (`generate_level_bounds`):
   * Calculates the number of nodes per level based on:
     * Total number of entries
     * Branching factor (B)
   * Creates a vector of ranges where each range represents a level in the tree.
   * First range represents leaf nodes, last range represents the root.

2. **Node Structure Calculation**:
   * B-1 keys per node (where B is the branching factor).
   * Nodes are stored contiguously in a level-by-level layout.
   * Level bounds store the start and end index of each level.

### 6.3. Tree Building Process (`generate_nodes`)

1. **Bottom-up Construction**:
   * Starts with leaf nodes (already prepared).
   * Builds internal nodes level by level, moving upward toward the root.

2. **Internal Node Generation**:
   * For each parent node slot, selects the minimum key from the right subtree.
   * Sets the offset to point to the corresponding child node's position.

3. **Packing Strategy**:
   * Tree is built with maximum packing density.
   * Last nodes at each level may be partially filled, but most nodes are fully utilized.
   * Follows the Eytzinger layout to optimize cache efficiency.

### 6.4. Payload Encoding

1. **Standard Entries**:
   * For entries with unique keys, the offset directly points to the value.

2. **Duplicate Key Handling**:
   * When duplicate keys exist, a special bit pattern in the offset (`PAYLOAD_TAG`) indicates it points to the payload area.
   * The actual offset is masked with `PAYLOAD_MASK` to get the position in the payload area.
   * Payload entries store multiple offsets for the duplicate keys.
   * `PayloadEntry` structure contains a list of offsets for duplicate key values.

### 6.5. Serialization (`stream_write`)

1. **Data Layout**:
   * Tree is serialized level by level, starting from the root downward.
   * Nodes and their entries are stored contiguously.
   * Payload area is appended at the end if duplicates exist.

### 6.6 Complete Construction Algorithm

The following steps summarize the complete construction process:

1. **Copy leaves** – Create subset of entries with unique keys, handling duplicates through the payload area.
2. **Tree layout** – Calculate level bounds based on branching factor and number of items.
3. **Build internal layers** – Generate internal nodes bottom-up from the leaf level.
4. **Serialize** – Write the structure to an output stream.
5. **Deserialize** – Read the structure back when needed for queries.

**Complexities**:

* **Build**: `O(N)` time, `O(N)` space.
* **Search**: `O(log_{B} N)` node touches (each touch requires reading a single node from the underlying reader, not the whole tree).

## 7. Search Operations

The crate provides various search operations optimized for different access patterns and data sources:

### 7.1. Direct Memory Search Operations

| Function | Description |
|----------|-------------|
| `find_exact` | Finds entries with an exact key match in memory-loaded tree |
| `find_range` | Finds entries with keys in a specified range in memory-loaded tree |
| `find_partition` | Finds the index where a key would be inserted (useful for determining positions) |

### 7.2. Stream-based Search Operations

| Function | Description |
|----------|-------------|
| `stream_find_exact` | Finds exact matches by reading only necessary nodes from a `Read + Seek` source |
| `stream_find_range` | Finds range matches by efficiently reading nodes within the range from a `Read + Seek` source |
| `stream_find_partition` | Finds insertion position by navigating the tree through streaming |

### 7.3. HTTP-based Search Operations

| Function | Description |
|----------|-------------|
| `http_stream_find_exact` | Performs exact match search using HTTP range requests to fetch only needed nodes |
| `http_stream_find_range` | Performs range search using HTTP range requests, optimizing data transfer |
| `http_stream_find_partition` | Determines insertion position through HTTP requests, useful for binary search |

### 7.4. Search Algorithm

The search process follows these general steps:

1. **Tree Navigation**:
   * Start at the root node.
   * At each internal node, use binary search to find the child node to descend to.
   * Continue until reaching a leaf node.

2. **Leaf Processing**:
   * For exact search: Use binary search to find the exact key.
   * For range search: Scan leaf nodes within the range boundaries.

3. **Result Collection**:
   * For each matching key, determine if it points to a direct value or payload area.
   * If direct, return the offset as a result.
   * If pointing to payload, read all offsets from the payload area and return multiple results.

4. **HTTP Optimization**:
   * HTTP-based operations use range requests to fetch only needed nodes.
   * Implements request batching when adjacent nodes need to be read to reduce HTTP round trips.
   * Uses `AsyncHttpRangeClient` to perform asynchronous HTTP operations.

### 7.5. Payload Handling Optimizations

The implementation includes advanced techniques for efficient payload handling:

#### 7.5.1. Payload Prefetching

Instead of making individual HTTP requests for each payload entry, a prefetching mechanism significantly reduces HTTP overhead:

1. **Initial Prefetch**:
   * Before starting a search, a chunk of the payload section is prefetched.
   * The size is computed based on the characteristics of the tree (number of entries, estimated duplicate rate).
   * A `PayloadCache` stores and manages the prefetched data.

2. **Adaptive Sizing**:
   * The prefetch size is adapted based on the tree characteristics:
     * For small trees: A minimum prefetch size ensures basic efficiency (16KB)
     * For medium trees: Prefetch size grows linearly with tree size
     * For large trees: Maximum caps prevent excessive memory usage (1MB by default)
   * Prefetch factor can be adjusted for specific workloads (higher for range queries, lower for exact matches)

3. **Cache Management**:
   * The `PayloadCache` maintains the prefetched data and provides fast offset lookup.
   * Subsequent payload lookups check the cache before making HTTP requests.
   * The cache is query-scoped to avoid growing memory usage over time.

#### 7.5.2. Batch Payload Resolution

When a search identifies multiple keys that reference payload entries, batch resolution combines them:

1. **Reference Collection**:
   * During tree traversal, references to payload entries are collected rather than immediately resolved.
   * Both direct offsets and payload references are tracked in a unified collection.

2. **Deduplication and Grouping**:
   * Duplicate payload references are removed to prevent redundant fetches.
   * Adjacent payload entries are grouped based on proximity (within 4KB).
   * This reduces the number of HTTP requests while avoiding excessive unused data transfer.

3. **Consolidated Fetching**:
   * A single HTTP request is made for each group of adjacent payload entries.
   * Fetched data is processed in memory to extract all payload entries.
   * Results from direct offsets and payload entries are combined in the final result set.

This batch approach can reduce HTTP requests by orders of magnitude when dealing with queries that return many results or datasets with many duplicate keys.

## 8. Performance Considerations

1. **Cache Efficiency**:
   * Uses Eytzinger layout for better cache locality.
   * Nodes at the same level are stored contiguously.

2. **Minimal I/O**:
   * Only reads nodes necessary for the search path.
   * Uses binary search within nodes to minimize comparisons.

3. **HTTP Optimizations**:
   * Batches adjacent node requests to reduce HTTP overhead.
   * Uses range requests to fetch only needed data.
   * Buffers HTTP responses for better performance.
   * Implements payload prefetching and batch resolution to minimize HTTP requests.
   * Adaptively sizes prefetch requests based on tree characteristics.
   * Groups adjacent payload references to maximize HTTP efficiency.

4. **Memory Efficiency**:
   * Static structure allows for precise memory allocation.
   * No need for dynamic node allocations or pointer indirection.
   * Compact representation with minimal overhead.
   * Payload cache balances memory usage with HTTP request reduction.

5. **Request Efficiency for HTTP**:
   * For typical queries, the number of HTTP requests is reduced by up to 90% through:
     * Proactive payload prefetching
     * Batch payload resolution
     * Adaptive grouping of nearby offsets
     * Cache-first lookup approach
   * These optimizations are especially effective for range queries or datasets with many duplicate keys.
   * The implementation automatically adapts to different network conditions and dataset characteristics.

## 9. Safety & Error Handling

* **No `unsafe`** is required; index math uses `usize` and is bounds‑checked in debug builds.
* All fallible functions use `crate::error::Result` (`std::result::Result<T, Error>`).
* `Error::IoError` is propagated verbatim from underlying I/O operations.

## 10. Query Interface

The static-btree crate provides a higher-level query interface designed to work with multiple indices and support complex query operations. This interface has been fully implemented and tested.

### 10.1 Query Module Structure

```
static-btree/
├── src/
│   ├── query/
│   │   ├── mod.rs         // Re-exports from submodules
│   │   ├── types.rs       // Query types and traits (KeyType, Operator, TypedQueryCondition)
│   │   ├── memory.rs      // In-memory index implementation (MemoryIndex, MemoryMultiIndex)
│   │   ├── stream.rs      // Stream-based index implementation (StreamIndex, StreamMultiIndex)
│   │   └── http.rs        // HTTP-based index implementation (feature-gated)
```

### 10.2 Core Query Types

The query module defines several core types to support high-level query operations:

1. **Operator Enum**: Defines comparison operators:
   * `Eq` - Equal
   * `Ne` - Not equal
   * `Gt` - Greater than
   * `Lt` - Less than
   * `Ge` - Greater than or equal
   * `Le` - Less than or equal

2. **KeyType Enum**: Heterogeneous key type support:
   * Integers: `Int32`, `Int64`, `UInt32`, `UInt64`
   * Floats: `Float32`, `Float64` (using OrderedFloat)
   * `Bool`: Boolean values
   * `DateTime`: Timestamps with UTC timezone
   * String keys: `StringKey20`, `StringKey50`, `StringKey100` (fixed-length strings)

3. **TypedQueryCondition**: Represents a single condition with:
   * Field name (string)
   * Operator (from the Operator enum)
   * Key value (from the KeyType enum)

### 10.3 Index Abstractions

The query module implements several index abstractions:

1. **Basic Index Types**:
   * **MemoryIndex\<K\>**: In-memory index for fast local operations
   * **StreamIndex\<K\>**: File-based index using Read+Seek for minimal memory usage
   * **HttpIndex\<K\>**: Remote index accessed via HTTP range requests

2. **Multi-Index Types**:
   * **MemoryMultiIndex**: Collection of in-memory indices with heterogeneous key types
   * **StreamMultiIndex**: Collection of stream-based indices with heterogeneous key types
   * **HttpMultiIndex\<T\>**: Collection of HTTP-based indices with heterogeneous key types

3. **Supporting Traits**:
   * **TypedSearchIndex**: For memory-based query execution
   * **TypedStreamSearchIndex**: For stream-based query execution
   * **TypedHttpSearchIndex\<T\>**: For HTTP-based query execution with AsyncHttpRangeClient

### 10.4 HTTP Index Implementation

The HTTP index implementation (`query/http.rs`) provides several key features:

1. **HttpIndex\<K\>**:
   * Implements exact and range queries over HTTP
   * Uses `AsyncHttpRangeClient` for network operations
   * Supports request batching to minimize HTTP round trips
   * Properly handles all key types through the `Key` trait

2. **TypedHttpSearchIndex\<T\> Trait**:
   * Provides a common interface for different key types
   * Implemented for all supported key types via a macro
   * Supports all comparison operators with proper type checking

3. **HttpMultiIndex\<T\>**:
   * Container for multiple HTTP indices of different key types
   * Query execution combines results from multiple conditions
   * Properly handles error cases and type mismatches

4. **Query Execution**:
   * Supports complex queries with multiple conditions
   * Implements result set intersection for AND logic
   * Properly expands payload references for duplicate keys

### 10.5 Query Execution Process

The query execution process in the static-btree crate follows these steps:

1. **Condition Evaluation**:
   * Each condition is evaluated against the appropriate index
   * For HTTP indices, conditions are translated to async HTTP operations
   * Results are collected as sets of offsets

2. **Result Combination**:
   * Results from different conditions are intersected (AND logic)
   * For HTTP queries, this happens after all async operations complete
   * The final set contains offsets that match all conditions

3. **Type Safety**:
   * The implementation provides compile-time type safety through generics
   * Runtime type checking is used for heterogeneous operations
   * Proper error messages are generated for type mismatches

### 10.6 Integration Testing

The query module includes comprehensive end-to-end tests:

1. **Memory Index Tests**:
   * Verify exact and range queries with various key types
   * Test all comparison operators and edge cases
   * Ensure proper handling of duplicate keys

2. **Stream Index Tests**:
   * Test serialization and deserialization
   * Verify correct operation with file-like sources
   * Ensure minimal memory usage during operations

3. **HTTP Index Tests**:
   * Use a mock HTTP client for deterministic testing
   * Verify that HTTP range requests fetch the correct data
   * Test complex queries with multiple conditions
   * Ensure proper handling of network errors

### 10.7 Current Status and Next Steps

All query module components have been fully implemented and tested, including:

* Memory-based indices
* Stream-based indices
* HTTP-based indices (feature-gated)
* End-to-end tests for all index types

The next phase is integration with fcb_core as outlined in the implementation_integrate_w_flatcitybuf.md document.

## 11. Limitations and Constraints

1. **Read-Only Structure**:
   * Tree is immutable after construction.
   * No support for insertions, deletions, or updates.

2. **Construction Cost**:
   * One-time O(N) build cost in both time and space.
   * Must rebuild the entire tree to incorporate new data.

3. **Memory Requirements**:
   * Full tree needs to be loaded in memory during construction.
   * Stream-based operations require minimal memory during queries.

## 12. Future Work

1. **Prefetch hints** – Explore prefetching for sequential range scans to improve performance.
2. **Compression** – Investigate node-level compression techniques for reduced storage requirements.
3. **Parallel construction** – Optimize tree building for multi-core systems.
4. **fcb_core Integration** – Create compatibility layer for smooth integration with fcb_core.

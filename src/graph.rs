use hashbrown::HashMap;
use std::hash::Hash;

/// A trait for nodes in the graph, which allows obtaining a key to identify them by.
pub trait HasKey {
    /// The key type. This should be small and suitable for use as a hashmap key.
    type Key: Hash + Eq + Clone;

    /// Get the key for this node. This should be a cheap operation - ideally just attribute access.
    fn key(&self) -> Self::Key;
}

/// An undirected graph with weighted edges.
///
/// The graph is implemented using a hashmap of nodes and a nested hashmap of edges. This type has
/// two type parameters:
/// - `N`: The value stored for each node. There are no requirements on this type other than that
///   it implements `HasKey<K>` to derive a key from it.
/// - `W`: The weight of each edge. In this graph type, every pair of nodes is connected by an edge,
///   initially with a weight of `W::default()`. This type must also implement `Clone` because edges
///   are stored twice, once for each endpoint. Note that `Option<T>` implements `Default`, so you
///   can use it to represent the concept of edges that may or may not exist.
#[derive(Clone, Debug)]
pub struct Graph<N: HasKey, W: Clone + Default> {
    // Nodes indexed by their key.
    nodes: HashMap<N::Key, N>,
    // Edges are stored as a nested hashmap, where the first key is the key of one node and the
    // second key is the other - the value is the weight. Each edge is stored twice, once for each
    // endpoint. These two should always have the same weight.
    edges: HashMap<N::Key, HashMap<N::Key, W>>,
}

impl<N: HasKey, W: Clone + Default> Default for Graph<N, W> {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }
}

impl<N: HasKey, W: Clone + Default> Graph<N, W> {
    /// Create a new, empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    /// Add a new node to the graph.
    ///
    /// If a node with the same key already exists, it will be replaced.
    pub fn add_node(&mut self, node: N) {
        self.nodes.insert(node.key(), node);
    }

    /// Get a reference to a node in the graph.
    pub fn get_node(&self, key: &N::Key) -> Option<&N> {
        self.nodes.get(key)
    }

    /// Get a mutable reference to a node in the graph.
    pub fn get_node_mut(&mut self, key: &N::Key) -> Option<&mut N> {
        self.nodes.get_mut(key)
    }

    /// Iterate over all nodes in the graph.
    pub fn nodes(&self) -> impl Iterator<Item = &N> {
        self.nodes.values()
    }

    /// Iterate over all nodes in the graph, mutably.
    pub fn nodes_mut(&mut self) -> impl Iterator<Item = &mut N> {
        self.nodes.values_mut()
    }

    /// Get the total number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Set the weight of an edge.
    ///
    /// Edges are undirected, so the weight of the edge from `to` to `from` will also be set.
    ///
    /// # Panics
    ///
    /// Panics if either of the nodes does not exist in the graph.
    pub fn set_weight(&mut self, from: N::Key, to: N::Key, weight: W) -> W {
        assert!(self.nodes.contains_key(&from));
        assert!(self.nodes.contains_key(&to));
        self.edges
            .entry(from.clone())
            .or_default()
            .insert(to.clone(), weight.clone());
        self.edges
            .entry(to)
            .or_default()
            .insert(from, weight)
            .unwrap_or_default()
    }

    /// Get the weight of an edge.
    ///
    /// Every pair of nodes is connected by an edge, so this always returns a value, even if that
    /// value is the default weight.
    pub fn get_weight(&self, from: &N::Key, to: &N::Key) -> W {
        self.edges
            .get(from)
            .and_then(|m| m.get(to))
            .cloned()
            .unwrap_or_default()
    }

    /// Iterate over the every edge of a given node.
    ///
    /// Since all nodes are connected, this will give one edge for every other node in the graph.
    pub fn edges(&self, key: N::Key) -> impl Iterator<Item = (&N, W)> {
        let siblings = self.edges.get(&key);
        self.nodes
            .values()
            .filter(move |node| node.key() != key)
            .map(move |node| {
                let weight = siblings.and_then(|m| m.get(&node.key()));
                (node, weight.cloned().unwrap_or_default())
            })
    }

    /// Remove a node from the graph, and return it if it existed.
    ///
    /// This will also remove all edges connected to the node.
    pub fn remove_node(&mut self, key: &N::Key) -> Option<N> {
        let node = self.nodes.remove(key);
        if node.is_some() {
            if let Some(siblings) = self.edges.remove(key) {
                for (sibling, _) in siblings {
                    self.edges.get_mut(&sibling).unwrap().remove(key);
                }
            }
        }
        node
    }
}

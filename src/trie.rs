use core::fmt;
use std::{collections::HashMap, io};

#[derive(Clone)]
struct Node {
    children: HashMap<String, usize>,
    count: u64,
}

impl Node {
    fn new() -> Self {
        Node {
            children: HashMap::<String, usize>::new(),
            count: 0,
        }
    }
}

pub struct Trie {
    nodes: Vec<Node>,
}

impl Trie {
    pub fn new() -> Self {
        Self {
            nodes: vec![Node::new()],
        }
    }

    fn get_node(&self, index: usize) -> Result<&Node, TrieErr> {
        self.nodes.get(index).ok_or(TrieErr::missing_node(index))
    }

    fn get_mut_node(&mut self, index: usize) -> Result<&mut Node, TrieErr> {
        self.nodes
            .get_mut(index)
            .ok_or(TrieErr::missing_node(index))
    }

    fn add_node<'a>(&mut self, parent_index: usize, prefix: &'a str) -> Result<usize, TrieErr> {
        let index = self.nodes.len();
        let parent = self.get_mut_node(parent_index)?;

        if let Some(index) = parent.children.get(prefix) {
            return Ok(*index);
        }

        parent.children.insert(prefix.to_string(), index);
        self.nodes.push(Node::new());
        Ok(index)
    }

    pub fn insert<'a>(&mut self, word: &'a str) -> Result<&mut Self, TrieErr> {
        let mut node_index = 0usize;

        for char in word.chars() {
            let node = self.get_mut_node(node_index)?;
            node.count += 1;

            let prefix = char.to_string();

            if let Some(index) = node.children.get(&prefix) {
                node_index = *index;
                continue;
            }

            node_index = self.add_node(node_index, &prefix)?;
        }
        self.get_mut_node(node_index)?.count += 1;
        Ok(self)
    }

    fn get_node_info(&self) -> (Vec<String>, Vec<usize>) {
        let mut parents = vec![0usize; self.nodes.len()];
        let mut prefixes = vec!["".to_string(); self.nodes.len()];

        for (index, node) in self.nodes.iter().enumerate() {
            for (cprefix, cindex) in node.children.iter() {
                parents[*cindex] = index;
                prefixes[*cindex] = cprefix.to_string();
            }
        }

        (prefixes, parents)
    }

    pub fn compress(&self) -> Result<Self, TrieErr> {
        let (mut prefixes, mut parents) = self.get_node_info();
        let mut new_nodes = vec![self.nodes[0].clone()];
        let mut stack = vec![0usize];

        while let Some(index) = stack.pop() {
            if index != 0 && new_nodes[index].children.len() == 1 {
                let cprefix = new_nodes[index].children.keys().nth(0).unwrap().clone();
                let cindex = new_nodes[index].children.get(&cprefix).unwrap().clone();
                let child = &self.nodes[cindex];

                if new_nodes[index].count == child.count {
                    // the is redundant, replace it with its only child
                    let mut prefix = prefixes[index].clone();
                    let parent = &mut new_nodes[parents[index]];

                    parent.children.remove_entry(&prefix);
                    prefix += &cprefix;

                    prefixes[index] = prefix.clone();
                    parent.children.insert(prefix, index);

                    new_nodes[index] = child.clone();
                    stack.push(index);

                    continue;
                }
            }

            // just copy the children, updating their indices
            let node = new_nodes[index].clone();

            for (cprefix, cindex) in node.children.iter() {
                let new_index = new_nodes.len();
                new_nodes[index]
                    .children
                    .get_mut(cprefix)
                    .map(|valref| *valref = new_index);

                new_nodes.push(self.nodes[*cindex].clone());
                stack.push(new_index);
                parents[new_index] = index;
                prefixes[new_index] = cprefix.clone();
            }
        }

        Ok(Self { nodes: new_nodes })
    }

    pub fn num_words(&self) -> u64 {
        self.get_node(0).map_or(0, |node| node.count)
    }

    pub fn sample(&self, mut id: u64) -> Result<String, TrieErr> {
        let mut node = self.get_node(0)?;
        if node.count == 0 {
            return Err(TrieErr::empty_trie());
        }

        let mut word = "".to_string();

        // expect `id < node.count` but wrap the id in case it's too big
        id = id % node.count;

        loop {
            let mut should_stop = true;

            for (prefix, index) in node.children.iter() {
                let child = self.get_node(*index)?;
                if id < child.count {
                    word += prefix;
                    node = child;
                    should_stop = false;
                    break;
                } else {
                    id -= child.count;
                }
            }

            if should_stop {
                break;
            }
        }

        Ok(word)
    }

    fn preorder_iter(&self) -> impl Iterator<Item = (&str, usize, usize)> {
        let mut stack = vec![("", 0usize, 0usize)];

        std::iter::from_fn(move || {
            let (prefix, index, depth) = stack.pop()?;
            let node = self.get_node(index).ok()?;

            for (cprefix, cindex) in node.children.iter() {
                stack.push((cprefix, *cindex, depth + 1));
            }

            Some((prefix, index, depth))
        })
    }
}

impl std::fmt::Display for Trie {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (mut prefix, index, depth) in self.preorder_iter() {
            if index == 0 {
                prefix = "root";
            }
            let count = self.get_node(index).ok().map_or(0, |node| node.count);
            let _ = write!(
                f,
                "{}{} (count={}, index={})\n",
                "    ".repeat(depth),
                prefix,
                count,
                index
            )?;
        }
        writeln!(f, "Num. nodes = {}", self.nodes.len())
    }
}

pub struct TrieErr {
    msg: String,
}

impl TrieErr {
    fn missing_node(index: usize) -> Self {
        TrieErr {
            msg: format!("Could not get node at index {}", index),
        }
    }

    fn empty_trie() -> Self {
        TrieErr {
            msg: "Cannot sample from an empty trie".to_string(),
        }
    }
}

impl From<TrieErr> for io::Error {
    fn from(value: TrieErr) -> Self {
        Self::new(io::ErrorKind::Other, format!("TrieErr: {}", value.msg))
    }
}

impl fmt::Display for TrieErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TrieErr: {}", self.msg)
    }
}

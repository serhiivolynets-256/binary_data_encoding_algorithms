use std::{cmp::Reverse, collections::HashMap};


#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    pub fn other(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left
        }
    }
}

impl From<Direction> for u32 {
    fn from(dir: Direction) -> u32 {
        match dir {
            Direction::Left => 0,
            Direction::Right => 1,
        }
    }
}

impl From<u32> for Direction {
    fn from(a: u32) -> Direction {
        match a {
            0 => Direction::Left,
            _ => Direction::Right,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Node {
    Leaf {
        value: u8,
        weight: u32,
    },
    Inner {
        l: Box<Node>,
        r: Box<Node>,
        weight: u32,
    }
}

impl Node {
    pub fn huffman_tree(mut nodes: Vec<Node>) -> Option<Self> {
        loop {
            nodes.sort_by_key(|n| Reverse(n.weight()));

            match (nodes.pop(), nodes.pop()) {
                (Some(right), Some(left)) => nodes.push(Node::from_children(left, right)),
                (Some(root), None) => return Some(root),
                _ => return None
            }
        }
    }

    pub fn from_children(l: Node, r: Node) -> Self {
        let weight = l.weight() + r.weight();
        Self::Inner {
            l: Box::new(l),
            r: Box::new(r),
            weight,}
    }

    pub fn value(&self) -> Option<u8> {
        match self {
            Self::Inner { .. } => None,
            Self::Leaf { value, ..} => Some(*value),
        }
    }

    pub fn weight(&self) -> u32 {
        match self {
            Self::Inner { weight, .. } => *weight,
            Self::Leaf { weight, ..} => *weight,
        }
    }

    pub fn get_child(&self, dir: Direction) -> Option<&Self> {
        match self {
            Self::Inner { l, r, ..} => {
                match dir {
                    Direction::Left => Some(l.as_ref()),
                    Direction::Right => Some(r.as_ref()),
                }
            },
            Self::Leaf { .. } => None
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf { .. })
    }

    pub fn generate_dict(&self) -> HashMap<u8, (u32, u32)> {
        let mut encoding_dictionary = HashMap::<u8, (u32, u32)>::new();

        for (node, path) in NodeIter::new(self) {
            if node.is_leaf() {
                let path: Vec<Direction> = path.iter().map(|(_, dir)| *dir).collect::<Vec<_>>();
                let mut numeric_repr: u32 = 0u32;
                let mut bit_len: u32 = 0u32;
                for dir in path {
                    numeric_repr |= u32::from(dir) << bit_len;
                    bit_len += 1;
                }
                encoding_dictionary.insert(node.value().unwrap(), (numeric_repr, bit_len));
            }
        }

        encoding_dictionary
    } 
}

#[derive(Debug, Clone)]
pub struct NodeIter<'a>{
    pub current_node: Option<&'a Node>,
    pub path: Vec<(&'a Node, Direction)>,
}

impl<'a> NodeIter<'a> {
    pub fn new(root: &'a Node) -> Self {
        NodeIter {
            current_node: Some(root),
            path: vec![],
        }
    }
}

impl<'a> Iterator for NodeIter<'a> {
    type Item = (&'a Node, Vec<(&'a Node, Direction)>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current_node) = self.current_node {
            let return_item = (current_node, self.path.clone());

            match &current_node {
                Node::Inner { l, .. } => {
                    // Try to go further down, to the left. The right case will be handled by the leaf case.
                    self.path.push((current_node, Direction::Left));
                    self.current_node = Some(l.as_ref());
                }
                Node::Leaf{ .. } => {
                    loop {
                        if let Some((parent, last_direction)) = self.path.pop() {
                            if let Node::Inner { .. } = &parent {
                                // Try to go right, or else go up
                                if last_direction == Direction::Right {
                                    // Go up.
                                    self.current_node = Some(parent);
                                } else if last_direction == Direction::Left {
                                    // go on the right.
                                    self.path.push((parent, Direction::Right));
                                    self.current_node = parent
                                        .get_child(Direction::Right);
                                    break;
                                }
                            }
                        } else {
                            self.current_node = None;
                            return Some(return_item);
                        }
                    }
                }
            }

            return Some(return_item);
        }

        None
    }
}

impl<'a> IntoIterator for &'a Node {
    type Item = (&'a Node, Vec<(&'a Node, Direction)>);
    type IntoIter = NodeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            current_node: Some(self),
            path: vec![],
        }
    }
}

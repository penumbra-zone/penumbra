use std::{borrow::Cow, io::Write};

use decaf377::FieldExt;

use crate::{
    internal::hash::Hash,
    structure::{self, Kind, Place},
    Commitment, Position, Tree,
};

impl crate::Tree {
    /// Renders the tree as a DOT format graph, for visualization of its structure.
    pub fn render_dot<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        dot::render(self, writer)
    }
}

#[derive(Clone, Debug)]
enum Node {
    Internal {
        position: Position,
        height: u8,
        hash: Option<Hash>,
    },
    Leaf {
        position: Position,
        commitment: Commitment,
    },
    Null {
        height: u8,
        position: Position,
    },
}

impl From<structure::Node<'_>> for Node {
    fn from(node: structure::Node<'_>) -> Self {
        Self::Internal {
            position: node.position(),
            height: node.height(),
            hash: node.cached_hash(),
        }
    }
}

#[derive(Clone, Debug)]
struct Edge {
    source: Node,
    target: Node,
    place: Option<Place>,
    helper: bool,
}

impl<'a> dot::GraphWalk<'a, Node, Edge> for Tree {
    fn nodes(&'a self) -> dot::Nodes<'a, Node> {
        let mut nodes = Vec::new();
        structure::traverse(self.structure(), &mut |node| {
            // Its commitment, if it's a leaf
            if let Kind::Leaf {
                commitment: Some(commitment),
            } = node.kind()
            {
                nodes.push(Node::Leaf {
                    position: node.position(),
                    commitment,
                });
            }

            // Its phantom children, if it's an internal node
            // if let Kind::Internal { .. } = node.kind() {
            //     let children_count = node.children().len() as u64;
            //     // Don't do this for the empty root node (the only internal node that can ever have
            //     // zero children)
            //     if children_count != 0 {
            //         for i in children_count..4u64 {
            //             nodes.push(Node::Null {
            //                 height: node.height() - 1,
            //                 // Calculate what the position *would be* for the phantom child
            //                 position: (u64::from(node.position()) + (i * node.stride()) / 4).into(),
            //             });
            //         }
            //     }
            // }

            // The node itself
            nodes.push(node.into());
        });
        Cow::Owned(nodes)
    }

    fn edges(&'a self) -> dot::Edges<'a, Edge> {
        let mut edges = Vec::new();
        structure::traverse(self.structure(), &mut |node| {
            let children = node.children();

            let mut child_nodes: Vec<(Node, Option<Place>)> = Vec::with_capacity(4);

            // Actual children
            for &child in children.iter() {
                child_nodes.push((child.into(), Some(child.place())));
            }

            // Phantom children
            // if !children.is_empty() {
            //     for i in children.len() as u64..4u64 {
            //         child_nodes.push((
            //             Node::Null {
            //                 height: node.height() - 1,
            //                 position: (u64::from(node.position()) + (i * node.stride()) / 4).into(),
            //             },
            //             None,
            //         ));
            //     }
            // }

            // Edges between children, to order them
            // for window in child_nodes.windows(2) {
            //     if let [(a, _), (b, _)] = window {
            //         edges.push(Edge {
            //             source: a.clone(),
            //             target: b.clone(),
            //             place: None,
            //             // Mark as invisible helper edge
            //             helper: true,
            //         });
            //     } else {
            //         unreachable!("windows always of correct size");
            //     }
            // }

            // Edges from parent to each child
            for (child, place) in child_nodes {
                edges.push(Edge {
                    source: node.into(),
                    target: child,
                    place,
                    helper: false,
                });
            }

            // Edge to commitment below, if any
            if let Kind::Leaf {
                commitment: Some(commitment),
            } = node.kind()
            {
                edges.push(Edge {
                    source: node.into(),
                    target: Node::Leaf {
                        position: node.position(),
                        commitment,
                    },
                    place: Some(node.place()),
                    helper: false,
                });
            }
        });
        Cow::Owned(edges)
    }

    fn source(&'a self, edge: &Edge) -> Node {
        edge.source.clone()
    }

    fn target(&'a self, edge: &Edge) -> Node {
        edge.target.clone()
    }
}

impl<'a> dot::Labeller<'a, Node, Edge> for Tree {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("Tree").unwrap()
    }

    fn node_id(&'a self, node: &Node) -> dot::Id<'a> {
        dot::Id::new(match node {
            Node::Internal {
                position, height, ..
            } => {
                format!(
                    "NODE_height_{}_epoch_{}_block_{}_commitment_{}",
                    height,
                    position.epoch(),
                    position.block(),
                    position.commitment()
                )
            }
            Node::Leaf { position, .. } => {
                format!(
                    "COMMITMENT_epoch_{}_block_{}_commitment_{}",
                    position.epoch(),
                    position.block(),
                    position.commitment()
                )
            }
            Node::Null { position, height } => {
                format!(
                    "NULL_height_{}_epoch_{}_block_{}_commitment_{}",
                    height,
                    position.epoch(),
                    position.block(),
                    position.commitment()
                )
            }
        })
        .unwrap()
    }

    fn node_shape(&'a self, node: &Node) -> Option<dot::LabelText<'a>> {
        let shape = match node {
            Node::Internal { hash, .. } => {
                let hash = if let Some(hash) = hash {
                    hash
                } else {
                    return Some(dot::LabelText::label("none"));
                };

                // The "empty" (finished or unfinished) shape is a point
                if hash.is_one() || hash.is_zero() {
                    return Some(dot::LabelText::label("point"));
                }

                // Use the first byte of the hash to determine the shape
                shape(hash.to_bytes()[0])
            }
            Node::Leaf { commitment, .. } => {
                // Use the first byte of the commitment to determine the shape
                shape(commitment.0.to_bytes()[0])
            }
            Node::Null { .. } => {
                return Some(dot::LabelText::label("point"));
            }
        };

        Some(dot::LabelText::label(shape))
    }

    fn node_label(&'a self, node: &Node) -> dot::LabelText<'a> {
        if let Node::Internal { hash: None, .. } = node {
            dot::LabelText::html("<b>?</b>")
        } else {
            dot::LabelText::label("")
        }
    }

    fn edge_label(&'a self, e: &Edge) -> dot::LabelText<'a> {
        let _ignored = e;
        dot::LabelText::LabelStr("".into())
    }

    fn node_style(&'a self, _n: &Node) -> dot::Style {
        dot::Style::Filled
    }

    fn node_color(&'a self, node: &Node) -> Option<dot::LabelText<'a>> {
        match node {
            Node::Internal { hash, .. } => {
                let hash = (*hash)?; // Empty hash gets no color

                // The "empty block"/"empty epoch" color is black
                if hash.is_one() {
                    return Some(dot::LabelText::label("black"));
                }

                // The "unfinished empty block/epoch" color is gray
                if hash.is_zero() {
                    return Some(dot::LabelText::label("gray"));
                }

                Some(dot::LabelText::label(color(hash.to_bytes()[0])))
            }
            Node::Leaf { commitment, .. } => {
                Some(dot::LabelText::label(color(commitment.0.to_bytes()[0])))
            }
            Node::Null { .. } => Some(dot::LabelText::label("gray")),
        }
    }

    fn edge_end_arrow(&'a self, _e: &Edge) -> dot::Arrow {
        dot::Arrow::none()
    }

    fn edge_start_arrow(&'a self, _e: &Edge) -> dot::Arrow {
        dot::Arrow::none()
    }

    fn edge_style(&'a self, edge: &Edge) -> dot::Style {
        // Helper edges are just for alignment
        if edge.helper {
            dot::Style::Dotted
        } else {
            dot::Style::Bold
        }
    }

    fn edge_color(&'a self, edge: &Edge) -> Option<dot::LabelText<'a>> {
        match edge.place {
            Some(Place::Frontier) => Some(dot::LabelText::label("#E800FF")),
            Some(Place::Complete) => Some(dot::LabelText::label("#000000")),
            None => Some(dot::LabelText::label("darkgray")),
        }
    }

    fn kind(&self) -> dot::Kind {
        dot::Kind::Digraph
    }
}

fn shape(byte: u8) -> &'static str {
    match byte % 16 {
        0 => "circle",
        1 => "egg",
        2 => "triangle",
        3 => "diamond",
        4 => "trapezium",
        5 => "parallelogram",
        6 => "house",
        7 => "pentagon",
        8 => "hexagon",
        9 => "septagon",
        10 => "octagon",
        11 => "invtriangle",
        12 => "invtrapezium",
        13 => "invhouse",
        14 => "square",
        15 => "star",
        _ => unreachable!("x % 16 < 16"),
    }
}

fn color(byte: u8) -> &'static str {
    // This is Paul Tol's colorblind-friendly palette, sourced from https://davidmathlogic.com/colorblind/
    match byte % 8 {
        0 => "#332288",
        1 => "#117733",
        2 => "#44AA99",
        3 => "#88CCEE",
        4 => "#DDCC77",
        5 => "#CC6677",
        6 => "#AA4499",
        7 => "#882255",
        _ => unreachable!("x % 8 < 8"),
    }
}

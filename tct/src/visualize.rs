use std::io::{self, Write};

use decaf377::FieldExt;

use crate::{
    structure::{Any, Kind, Node, Place},
    Position,
};

const FRONTIER_EDGE_COLOR: &str = "#E800FF";
const FRONTIER_TERMINUS_COLOR: &str = "#FBD1FF";

fn hash_shape(bytes: &[u8]) -> &'static str {
    match bytes[3] % 16 {
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
        15 => "oval",
        _ => unreachable!("x % 16 < 16"),
    }
}

fn hash_color(bytes: &[u8]) -> String {
    // This is Paul Tol's colorblind-friendly palette, sourced from https://davidmathlogic.com/colorblind/
    let nibble_color = |nibble| match nibble % 8 {
        0 => "#332288",
        1 => "#117733",
        2 => "#44AA99",
        3 => "#88CCEE",
        4 => "#DDCC77",
        5 => "#CC6677",
        6 => "#AA4499",
        7 => "#882255",
        _ => unreachable!("x % 8 < 8"),
    };

    // Get two colors that aren't the same, so no color looks "flat"
    let nibble_1 = bytes[0] % 8;
    let mut nibble_2 = bytes[1] % 7;
    if nibble_2 >= nibble_1 {
        nibble_2 += 1;
    }

    format!("{}:{}", nibble_color(nibble_1), nibble_color(nibble_2))
}

impl crate::Tree {
    /// Renders the tree as a DOT format graph, for visualization of its structure.
    pub fn render_dot<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        DotWriter::digraph(writer, |w| {
            let root = self.structure();
            w.nodes_and_edges(root)?;

            // Connect the commitments with invisible edges to align them
            let mut left = None;
            for (right, _) in self.commitments_ordered() {
                if let Some(left) = left {
                    w.commitment_commitment_edge(left, right)?;
                    // w.commitment_commitment_edge(right, left)?;
                }
                left = Some(right);
            }

            Ok(())
        })
    }
}

struct DotWriter<W: Write> {
    indent: usize,
    writer: W,
}

impl<W: Write> DotWriter<W> {
    fn digraph(mut writer: W, graph: impl FnOnce(&mut Self) -> io::Result<()>) -> io::Result<()> {
        writeln!(writer, "digraph {{")?;
        writeln!(writer, "  fontsize=\"24\";")?;
        writeln!(writer, "  fontname=\"Courier New\";")?;
        writeln!(writer, "  ordering=\"out\";")?;
        let mut dot_writer = DotWriter { indent: 1, writer };
        graph(&mut dot_writer)?;
        writeln!(dot_writer.writer, "}}")
    }

    fn nodes_and_edges(&mut self, node: Node) -> io::Result<()> {
        self.node(node)?; // The node itself
        self.node_commitment(node)?; // Its commitment below, if any
        for child in node.children() {
            // All its children, as subgraphs
            self.subtree(
                child.height(),
                child.position(),
                child.place(),
                !child.children().is_empty(),
                matches!(
                    child.kind(),
                    Kind::Leaf {
                        commitment: Some(_)
                    }
                ),
                |w| w.nodes_and_edges(child),
            )?;
        }
        self.outgoing_edges(node)?; // Connect it to its children
        Ok(())
    }

    fn indent(&mut self) -> io::Result<()> {
        for _ in 0..self.indent {
            write!(self.writer, "  ")?;
        }
        Ok(())
    }

    fn line(&mut self, line: impl FnOnce(&mut W) -> io::Result<()>) -> io::Result<()> {
        self.indent()?;
        line(&mut self.writer)?;
        writeln!(self.writer, ";")
    }

    fn subgraph(
        &mut self,
        id: impl Fn(&mut W) -> io::Result<()>,
        cluster: bool,
        graph: impl FnOnce(&mut Self) -> io::Result<()>,
    ) -> io::Result<()> {
        self.indent()?;
        write!(
            self.writer,
            "subgraph {cluster}SUBGRAPH_",
            cluster = if cluster { "cluster_" } else { "" }
        )?;
        id(&mut self.writer)?;
        writeln!(self.writer, " {{")?;

        // Increase the indent for everything inside
        self.indent += 1;

        // Write the id for the subgraph
        self.indent()?;
        write!(self.writer, "id=\"")?;
        id(&mut self.writer)?;
        writeln!(self.writer, "\";")?;

        // Write the full subgraph
        graph(self)?;

        // Decrease the indent when exiting
        self.indent -= 1;

        self.indent()?;
        writeln!(self.writer, "}}")
    }

    fn subtree(
        &mut self,
        height: u8,
        position: Position,
        place: Place,
        terminal: bool,
        has_commitment: bool,
        tree: impl FnOnce(&mut Self) -> io::Result<()>,
    ) -> io::Result<()> {
        // The node is the focus if it is the terminus of the frontier
        let focus = terminal && place == Place::Frontier && height == 0;

        let id = |w: &mut W| {
            write!(
                w,
                "SUBTREE_height_{}_epoch_{}_block_{}_commitment_{}",
                height,
                position.epoch(),
                position.block(),
                position.commitment()
            )
        };

        let label = |w: &mut W| {
            // Don't label subtrees with commitments directly beneath, it's cleaner
            if has_commitment {
                return Ok(());
            }
            match height {
                16 => write!(w, "{}", position.epoch()),
                8 => write!(w, "{}/{}", position.epoch(), position.block()),
                0 => write!(
                    w,
                    "{}/{}/{}",
                    position.epoch(),
                    position.block(),
                    position.commitment()
                ),
                _ => Ok(()),
            }
        };

        self.subgraph(id, height % 8 == 0, |w| {
            // Write the subgraph label
            w.line(|w| write!(w, "labelloc=\"b\""))?;
            w.line(|w| {
                write!(w, "label=\"")?;
                label(w)?;
                write!(w, "\"")
            })?;

            tree(w)?;

            // Attributes
            let (fill_color, color, dashed) = if focus {
                (FRONTIER_TERMINUS_COLOR, "black", "")
            } else if height == 8 || height == 16 {
                ("none", "grey", ",dashed")
            } else {
                ("none", "none", "")
            };
            let tooltip = match height {
                16 => format!("Epoch {}", position.epoch()),
                8 => format!("Epoch {}, Block {}", position.epoch(), position.block()),
                0 => format!(
                    "Epoch {}, Block {}, Commitment {}",
                    position.epoch(),
                    position.block(),
                    position.commitment()
                ),
                _ => "".to_string(),
            };
            w.line(|w| write!(w, "color=\"{color}\""))?;
            w.line(|w| write!(w, "style=\"rounded,filled,bold{dashed}\""))?;
            w.line(|w| write!(w, "tooltip=\"{tooltip}\""))?;
            w.line(|w| write!(w, "fillcolor=\"{fill_color}\""))
        })
    }

    fn node(&mut self, node: Node) -> io::Result<()> {
        self.line(|w| {
            // The node identifier
            write!(
                w,
                "NODE_height_{}_epoch_{}_block_{}_commitment_{}",
                node.height(),
                node.position().epoch(),
                node.position().block(),
                node.position().commitment(),
            )?;
            // The node attributes
            write!(w, "[fontsize=\"20\"]")?;
            write!(w, "[fontname=\"Courier New\"]")?;
            write!(w, "[ordering=\"out\"]")?;
            write!(w, "[label=\"{}\"]", node_label(&node))?;
            write!(w, "[shape=\"{}\"]", node_shape(&node))?;
            write!(w, "[style=\"{}\"]", node_style(&node))?;
            write!(w, "[color=\"{}\"]", node_border_color(&node))?;
            write!(w, "[fillcolor=\"{}\"]", node_color(&node))?;
            write!(w, "[gradientangle=\"{}\"]", node_gradient_angle(&node))?;
            write!(w, "[width=\"{}\"]", node_width(&node))?;
            write!(w, "[height=\"{}\"]", node_height(&node))?;
            write!(
                w,
                "[tooltip=\"Hash: {}\"]",
                node.cached_hash()
                    .map(|h| format!("{:?}", h))
                    .unwrap_or_else(|| "?".to_string())
            )?;
            write!(w, "[orientation=\"{}\"]", node_orientation(&node))
        })
    }

    fn node_commitment(&mut self, node: Node) -> io::Result<()> {
        if let Kind::Leaf {
            commitment: Some(commitment),
        } = node.kind()
        {
            let id = |w: &mut W| {
                write!(
                    w,
                    "COMMITMENT_epoch_{}_block_{}_commitment_{}",
                    node.position().epoch(),
                    node.position().block(),
                    node.position().commitment()
                )
            };

            self.subgraph(id, true, |w| {
                w.line(|w| write!(w, "style=\"filled\""))?;
                w.line(|w| write!(w, "color=\"black\""))?;
                w.line(|w| write!(w, "fillcolor=\"lightyellow\""))?;
                w.line(|w| write!(w, "style=\"rounded,filled,bold\""))?;
                w.line(|w| {
                    write!(
                        w,
                        "tooltip=\"Epoch {}, Block {}, Commitment {}\"",
                        node.position().epoch(),
                        node.position().block(),
                        node.position().commitment()
                    )
                })?;
                w.line(|w| {
                    write!(w, "label=\"")?;
                    write!(
                        w,
                        "{}/{}/{}",
                        node.position().epoch(),
                        node.position().block(),
                        node.position().commitment()
                    )?;
                    write!(w, "\"")
                })?;
                w.line(|w| {
                    // The node identifier
                    id(w)?;
                    write!(w, "[label=\"\"]")?;
                    write!(w, "[shape=\"{}\"]", hash_shape(&commitment.0.to_bytes()))?;
                    write!(w, "[style=\"filled,bold\"]")?;
                    write!(w, "[color=\"black\"]")?;
                    write!(w, "[width=\"1\"]")?;
                    write!(w, "[height=\"1\"]")?;
                    write!(
                        w,
                        "[fillcolor=\"{}\"]",
                        hash_color(&commitment.0.to_bytes())
                    )?;
                    write!(
                        w,
                        "[gradientangle=\"{}\"]",
                        hash_gradient_angle(&commitment.0.to_bytes())
                    )?;
                    write!(
                        w,
                        "[orientation=\"{}\"]",
                        hash_orientation(&commitment.0.to_bytes())
                    )?;
                    write!(w, "[tooltip=\"Commitment: {}\"]", commitment)
                })
            })?;
        }

        Ok(())
    }

    fn outgoing_edges(&mut self, node: Node) -> io::Result<()> {
        self.node_commitment_edge(node)?;
        let mut left = None;
        for child in node.children() {
            if let Some(left) = left {
                self.sibling_sibling_edge(left, child)?;
                // self.sibling_sibling_edge(child, left)?;
            }
            self.parent_child_edge(node, child)?;
            left = Some(child);
        }
        Ok(())
    }

    fn parent_child_edge(&mut self, parent: Node, child: Node) -> io::Result<()> {
        self.line(|w| {
            write!(
                w,
                "NODE_height_{}_epoch_{}_block_{}_commitment_{}",
                parent.height(),
                parent.position().epoch(),
                parent.position().block(),
                parent.position().commitment()
            )?;
            write!(w, " -> ")?;
            write!(
                w,
                "NODE_height_{}_epoch_{}_block_{}_commitment_{}",
                child.height(),
                child.position().epoch(),
                child.position().block(),
                child.position().commitment()
            )?;
            write!(w, "[label=\"\"]",)?;
            write!(w, "[dir=\"none\"]")?;
            write!(w, "[style=\"bold\"]")?;
            let color = match child.place() {
                Place::Frontier => match child.height() {
                    8 if parent.global_position().unwrap().commitment() == 0 => "black".to_string(),
                    16 if parent.global_position().unwrap().block() == 0
                        && parent.global_position().unwrap().commitment() == 0 =>
                    {
                        "black".to_string()
                    }
                    _ if child.height() > 0 && child.children().is_empty() => "black".to_string(),
                    _ => format!("{FRONTIER_EDGE_COLOR}:invis:{FRONTIER_EDGE_COLOR}"),
                },
                _ => "black".to_string(),
            };
            write!(w, "[color=\"{}\"]", color)
        })
    }

    fn sibling_sibling_edge(&mut self, left: Node, right: Node) -> io::Result<()> {
        self.line(|w| {
            write!(
                w,
                "NODE_height_{}_epoch_{}_block_{}_commitment_{}",
                left.height(),
                left.position().epoch(),
                left.position().block(),
                left.position().commitment()
            )?;
            write!(w, " -> ")?;
            write!(
                w,
                "NODE_height_{}_epoch_{}_block_{}_commitment_{}",
                right.height(),
                right.position().epoch(),
                right.position().block(),
                right.position().commitment()
            )?;
            write!(w, "[label=\"\"]",)?;
            write!(w, "[dir=\"none\"]")?;
            write!(w, "[style=\"invis\"]")?;
            write!(w, "[constraint=false]")
        })
    }

    fn commitment_commitment_edge(&mut self, left: Position, right: Position) -> io::Result<()> {
        self.line(|w| {
            write!(
                w,
                "COMMITMENT_epoch_{}_block_{}_commitment_{}",
                left.epoch(),
                left.block(),
                left.commitment()
            )?;
            write!(w, " -> ")?;
            write!(
                w,
                "COMMITMENT_epoch_{}_block_{}_commitment_{}",
                right.epoch(),
                right.block(),
                right.commitment()
            )?;
            write!(w, "[label=\"\"]",)?;
            write!(w, "[dir=\"none\"]")?;
            write!(w, "[style=\"invis\"]")?;
            write!(w, "[constraint=false]")
        })
    }

    fn node_commitment_edge(&mut self, node: Node) -> io::Result<()> {
        if let Kind::Leaf {
            commitment: Some(_),
        } = node.kind()
        {
            self.line(|w| {
                write!(
                    w,
                    "NODE_height_{}_epoch_{}_block_{}_commitment_{}",
                    node.height(),
                    node.position().epoch(),
                    node.position().block(),
                    node.position().commitment()
                )?;
                write!(w, " -> ")?;
                write!(
                    w,
                    "COMMITMENT_epoch_{}_block_{}_commitment_{}",
                    node.position().epoch(),
                    node.position().block(),
                    node.position().commitment()
                )?;
                write!(w, "[label=\"\"]",)?;
                write!(w, "[dir=\"none\"]")?;
                write!(w, "[style=\"bold\"]")?;
                write!(
                    w,
                    "[lhead=\"cluster_SUBGRAPH_COMMITMENT_epoch_{}_block_{}_commitment_{}\"]",
                    node.position().epoch(),
                    node.position().block(),
                    node.position().commitment()
                )?;
                let color = "black";
                write!(w, "[color=\"{}\"]", color)
            })?;
        }

        Ok(())
    }
}

fn node_shape(node: &Node) -> &'static str {
    let hash = if let Some(hash) = node.cached_hash() {
        hash
    } else {
        return "doublecircle";
    };

    // The "empty" (finished or unfinished) shape is a point
    if hash.is_one() || hash.is_zero() {
        return "circle";
    }

    // Use the first byte of the hash to determine the shape
    hash_shape(&hash.to_bytes())
}

fn node_label(node: &Node) -> &'static str {
    if node.cached_hash().is_none() {
        "?"
    } else {
        ""
    }
}

fn node_style(_node: &Node) -> &'static str {
    "filled,bold"
}

fn node_width(node: &Node) -> &'static str {
    if let Some(hash) = node.cached_hash() {
        if hash.is_one() || hash.is_zero() {
            return "0.15";
        }
    }

    "0.9"
}

fn node_height(node: &Node) -> &'static str {
    node_width(node)
}

fn node_color(node: &Node) -> String {
    let hash = if let Some(hash) = node.cached_hash() {
        hash
    } else {
        return FRONTIER_TERMINUS_COLOR.to_string();
    };

    // The "empty block"/"empty epoch" color is black
    if hash.is_one() {
        return "black".to_string();
    }

    // The "unfinished empty block/epoch" color is gray
    if hash.is_zero() {
        return "lightgray".to_string();
    }

    hash_color(&hash.to_bytes())
}

fn node_border_color(node: &Node) -> &'static str {
    if node.cached_hash().is_none() {
        return FRONTIER_EDGE_COLOR;
    }

    "black"
}

fn node_gradient_angle(node: &Node) -> String {
    let hash = if let Some(hash) = node.cached_hash() {
        hash
    } else {
        return "0".to_string();
    };

    // The "empty block"/"empty epoch" color is black
    if hash.is_one() {
        return "0".to_string();
    }

    // The "unfinished empty block/epoch" color is gray
    if hash.is_zero() {
        return "0".to_string();
    }

    hash_gradient_angle(&hash.to_bytes())
}

fn node_orientation(node: &Node) -> String {
    let hash = if let Some(hash) = node.cached_hash() {
        hash
    } else {
        return "0".to_string();
    };

    // The "empty block"/"empty epoch" color is black
    if hash.is_one() {
        return "0".to_string();
    }

    // The "unfinished empty block/epoch" color is gray
    if hash.is_zero() {
        return "0".to_string();
    }

    hash_orientation(&hash.to_bytes())
}

fn hash_gradient_angle(bytes: &[u8]) -> String {
    let nibble_angle = |nibble| (((nibble % 16) as u64) * 360) / 16;
    format!("{}", nibble_angle(bytes[2]))
}

fn hash_orientation(bytes: &[u8]) -> String {
    let nibble_angle = |nibble| (((nibble % 16) as u64) * 360) / 16;
    format!("{}", nibble_angle(bytes[4]))
}

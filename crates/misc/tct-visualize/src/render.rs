use std::io::{self, Write};

use penumbra_tct::{
    structure::{Kind, Node, Place},
    Position, Tree,
};

mod dot;

/// Renders the tree as a DOT format graph, for visualization of its structure.
pub fn dot<W: Write>(tree: &Tree, writer: &mut W) -> io::Result<()> {
    dot::render(tree, false, writer)
}

/// Renders the tree as a DOT format graph, like [`Tree::render_dot`], but with the formatting
/// of the DOT file more human-readable and well-indented.
pub fn dot_pretty<W: Write>(tree: &Tree, writer: &mut W) -> io::Result<()> {
    dot::render(tree, true, writer)
}

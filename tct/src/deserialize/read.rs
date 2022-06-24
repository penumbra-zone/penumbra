//! Read from ordered traversals of the tree's contents into instructions for constructing the tree.
//!
//! This module serves as a bridge between an arbitrary storage backend keyed by (position, depth),
//! holding [`Fq`] values, and the sequence of [`Instruction`]s required to build the tree with
//! [`deserialize::from_stream`](from_stream).

use super::*;

/// A reader that converts a stream of [`Fq`] values lexicographically ordered by (position, depth)
/// into a sequence of [`Instruction`]s to follow to reconstruct the represented tree.
pub struct Reader<R> {
    reader: R,
    position: u64,
    depth: u8,
    peek: Option<Point>,
}

/// A point in the serialized tree: the hash or commitment (represented by an [`Fq`]), and its
/// position and depth in the tree.
///
/// The depth is the distance from the root, so leaf hashes have depth 24, and commitments
/// themselves have depth 25.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    /// The position of the value.
    pub position: u64,
    /// The depth of the value from the root of the tree.
    ///
    /// Note that this representation means that leaf hashes have depth 24, and commitments
    /// themselves have depth 25.
    pub depth: u8,
    /// The value at this point.
    pub here: Fq,
}

/// An error while reading a tree point from a stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum Error<E> {
    /// The depth of the point in the stream did not increase from the previous point.
    #[error("tree stream reversed or repeated depth for position {position}: expected at least depth{expected_at_least}, got depth {actual}")]
    DepthNonIncreasing {
        /// The position of the error.
        position: u64,
        /// The minimum depth required.
        expected_at_least: u8,
        /// The actual depth observed.
        actual: u8,
    },
    /// The position decreased from the previous point.
    #[error("tree stream reversed position: expected at least position {expected_at_least}, got position {actual}")]
    PositionDecreasing {
        /// The minimum position required.
        expected_at_least: u64,
        /// The actual position observed.
        actual: u64,
    },
    /// The position was not valid with respect to the depth: positions must always be an exact
    /// multiple of the height.
    #[error("tree stream contained invalid position: position {position} is not valid for depth {depth}")]
    InvalidPosition {
        /// The position observed.
        position: u64,
        /// The depth observed.
        depth: u8,
    },
    /// Some underlying error occurred in the stream.
    #[error("{error}")]
    Underlying {
        /// The underlying error.
        #[from]
        error: E,
    },
}

impl<R, E> Reader<R>
where
    R: Stream<Item = Result<Point, E>> + Unpin,
{
    /// Create a new reader from an underlying stream.
    pub fn new(reader: R) -> Result<Self, E> {
        Ok(Self {
            reader,
            position: 0,
            depth: 0,
            peek: None,
        })
    }

    /// Convert this into a stream of instructions, suitable to be read in using [`from_stream`].
    pub fn stream(mut self) -> impl Stream<Item = Result<Instruction, Error<E>>> + Unpin {
        Box::pin(try_stream! {
            while let Some(instruction) = self.next().await? {
                yield instruction;
            }
        })
    }

    /// Get the next instruction from the reader.
    #[allow(clippy::comparison_chain)]
    pub async fn next(&mut self) -> Result<Option<Instruction>, Error<E>> {
        if let Some(Point {
            position,
            depth,
            here,
        }) = self.peek().await?
        {
            if self.position == position {
                if self.depth == depth {
                    // We have consumed the point, and are about to emit an instruction, so we
                    // should set `peek` to `None` so that we keep consuming the stream later
                    self.peek = None;

                    // Examine the upcoming point to determine whether to emit a leaf or a node
                    // instruction
                    if let Some(point) = self.peek().await? {
                        // If the next upcoming point is below us, in the same position, then emit a
                        // node instruction; otherwise, emit a leaf instruction:
                        if self.position == point.position {
                            self.down(Some(here))
                        } else {
                            self.right(here)
                        }
                    } else {
                        // End of stream means we should emit a leaf node
                        self.right(here)
                    }
                } else if self.depth < depth {
                    // Proxy internal node to get down to the place where the next actually
                    // represented internal node lies
                    self.down(None)
                } else {
                    unreachable!("point has been pre-checked for validity")
                }
            } else if self.position < position {
                // Proxy leaf node to fill in our previously-made promise of a full 4-child node,
                // even if not represented in the underlying stream
                self.right(Hash::one().into())
            } else {
                unreachable!("point has been pre-checked for validity")
            }
        } else {
            Ok(None)
        }
    }

    // Advance the position down by one level, returning the appropriate instruction to represent
    // the action taken.
    fn down(&mut self, here: Option<Fq>) -> Result<Option<Instruction>, Error<E>> {
        // Internal nodes have size 4; leaves have only one child
        let size = if self.depth >= 24 {
            Size::One
        } else {
            Size::Four
        };
        self.depth -= 1;
        Ok(Some(Instruction::Node { here, size }))
    }

    // Advance the position right by one stride, adjusting the depth upwards if required to match,
    // and returning the appropriate instruction to represent the action taken.
    fn right(&mut self, here: Fq) -> Result<Option<Instruction>, Error<E>> {
        // Advance the position by one stride
        self.position += self.stride();
        // If we reach the end of the current node, pop up until we get to the next starting level
        while self.position % self.stride() == 0 {
            self.depth -= 1;
        }
        Ok(Some(Instruction::Leaf { here }))
    }

    /// Fill in the currently peeked head of the stream, if it's not already filled in, and return
    /// it for inspection.
    #[allow(clippy::comparison_chain)]
    async fn peek(&mut self) -> Result<Option<Point>, Error<E>> {
        if self.peek.is_none() {
            let point = Pin::new(&mut self.reader).next().await.transpose()?;
            if let Some(
                point @ Point {
                    position, depth, ..
                },
            ) = point
            {
                if self.position == point.position {
                    if self.depth > point.depth {
                        self.peek = Some(point);
                    } else {
                        return Err(Error::DepthNonIncreasing {
                            position,
                            expected_at_least: self.depth + 1,
                            actual: depth,
                        });
                    }
                } else if self.position < point.position {
                    self.peek = Some(point);
                } else {
                    return Err(Error::PositionDecreasing {
                        expected_at_least: self.position,
                        actual: position,
                    });
                }
            }
        }

        Ok(self.peek)
    }

    fn height(&self) -> u8 {
        24u8.saturating_sub(self.depth)
    }

    fn stride(&self) -> u64 {
        4u64.pow(self.height().into())
    }
}

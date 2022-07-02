use super::*;

/// Generate an arbitrary instruction.
pub fn instruction() -> impl proptest::prelude::Strategy<Value = Instruction> {
    use proptest::prelude::*;

    proptest::option::of(crate::commitment::FqStrategy::arbitrary()).prop_flat_map(|option_fq| {
        Size::arbitrary().prop_flat_map(move |children| {
            bool::arbitrary().prop_map(move |variant| {
                if let Some(here) = option_fq {
                    if variant {
                        Instruction::Node {
                            here: Some(here),
                            size: children,
                        }
                    } else {
                        Instruction::Leaf { here }
                    }
                } else {
                    Instruction::Node {
                        here: None,
                        size: children,
                    }
                }
            })
        })
    })
}

/// When inserting a [`Commitment`] into a [`Tree`], should we [`Keep`](Witness::Keep) it to allow
/// it to be witnessed later, or [`Forget`](Witness::Forget) about it after updating the root
/// hash of the tree?
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(any(test, feature = "arbitrary"), derive(proptest_derive::Arbitrary))]
pub enum Witness {
    /// When inserting a [`Commitment`] into a [`Tree`], this flag indicates that we should
    /// immediately forget about it to save space, because we will not want to witness its presence
    /// later.
    ///
    /// This is equivalent to inserting the commitment using [`Witness::Keep`] and then immediately
    /// forgetting that same commitment using [`Tree::forget`], though it is more efficient to
    /// directly forget commitments upon insertion rather than to remember them on insertion and
    /// then immediately forget them.
    Forget,
    /// When inserting a [`Commitment`] into a [`Tree`], this flag indicates that we should keep
    /// this commitment to allow it to be witnessed later.
    Keep,
}

impl serde::Serialize for Witness {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Witness::Forget => serializer.serialize_str("forget"),
            Witness::Keep => serializer.serialize_str("keep"),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Witness {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct WitnessVisitor;

        impl<'de> serde::de::Visitor<'de> for WitnessVisitor {
            type Value = Witness;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("one of \"keep\" or \"forget\"")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                match value {
                    "forget" => Ok(Witness::Forget),
                    "keep" => Ok(Witness::Keep),
                    _ => Err(E::custom(format!(
                        "invalid witness flag: expected \"forget\" or \"keep\", found '{}'",
                        value
                    ))),
                }
            }
        }

        deserializer.deserialize_str(WitnessVisitor)
    }
}

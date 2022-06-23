/// An individual limb value.
pub struct Value(pub u32);

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value(value)
    }
}

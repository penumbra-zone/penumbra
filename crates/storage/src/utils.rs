use anyhow::bail;
/// Splits a range into a tuple of start and end bounds, ignoring the inclusive/exclusive
/// nature of the range bounds. And returns a tuple consisting of the range implementation,
/// and the start and end bounds.
/// # Errors
/// This method returns an error when the range is inclusive on the end bound,
/// and when the lower bound is greater than the upper bound.
pub(crate) fn convert_bounds(
    range: impl std::ops::RangeBounds<Vec<u8>>,
) -> anyhow::Result<(
    impl std::ops::RangeBounds<Vec<u8>>,
    (Option<Vec<u8>>, Option<Vec<u8>>),
)> {
    let start = match range.start_bound() {
        std::ops::Bound::Included(v) => Some(v.clone()),
        std::ops::Bound::Excluded(v) => Some(v.clone()),
        std::ops::Bound::Unbounded => None,
    };

    let end = match range.end_bound() {
        std::ops::Bound::Included(_) => bail!("included end bound not supported"),
        std::ops::Bound::Excluded(v) => Some(v.clone()),
        std::ops::Bound::Unbounded => None,
    };

    if let (Some(k_start), Some(k_end)) = (&start, &end) {
        if k_start > k_end {
            bail!("lower bound is greater than upper bound")
        }
    }

    Ok((range, (start, end)))
}

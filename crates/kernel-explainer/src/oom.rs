// OOM score → description (low/moderate/high/critical).
//
// See plan.md §2f for the bands.

pub fn oom_description(score: i32) -> &'static str {
    match score {
        s if s < 0 => "Adjusted below 0 — unlikely to be killed",
        0..=100 => "Low kill likelihood — kernel prefers to keep this process",
        101..=500 => "Moderate kill likelihood — may be killed under memory pressure",
        501..=900 => "High kill likelihood — one of the first OOM kill candidates",
        _ => "Critical — killed first under any memory pressure",
    }
}

use std::cmp::Ordering;

use konekode_record::DataValue;

use crate::domain::UnitDomain;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Count {
    pub legacy: i64,
    pub unread: i64,
}

#[must_use]
pub fn listed(value: Option<&DataValue>) -> Option<Vec<String>> {
    let Some(DataValue::Text(text)) = value else {
        return None;
    };
    Some(
        text.split(',')
            .filter(|id| !id.is_empty())
            .map(str::to_owned)
            .collect(),
    )
}

pub fn described(
    count: &mut Count,
    own: Option<&DataValue>,
    look_up: impl FnOnce() -> Option<Vec<String>>,
) -> Vec<String> {
    if let Some(told) = listed(own) {
        return told;
    }
    let Some(found) = look_up() else {
        count.unread += 1;
        return Vec::new();
    };
    count.legacy += 1;
    found
}

#[must_use]
pub fn shown_by(domain: Option<&UnitDomain>, beat: &str) -> Option<Vec<String>> {
    let domain = domain?;
    Some(domain.shows.get(beat).cloned().unwrap_or_default())
}

fn array_index(step: f64) -> Option<usize> {
    let integral = matches!(step.fract().partial_cmp(&0.0), Some(Ordering::Equal));
    if !step.is_finite() || !integral || step < 0.0 {
        return None;
    }
    format!("{step:.0}").parse().ok()
}

#[must_use]
pub fn used_by(domain: Option<&UnitDomain>, task: &str, step: f64) -> Option<Vec<String>> {
    let domain = domain?;
    Some(
        domain
            .uses
            .get(task)
            .and_then(|steps| array_index(step).and_then(|index| steps.get(index)))
            .cloned()
            .unwrap_or_default(),
    )
}

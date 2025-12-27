use std::collections::HashSet;

use slint::Model;

use crate::{GroupData, PersonData};

pub(super) fn member_ids(group: &GroupData) -> HashSet<i32> {
    let mut ids = HashSet::new();
    for i in 0..group.members.row_count() {
        if let Some(pd) = group.members.row_data(i) {
            ids.insert(pd.id);
        }
    }
    ids
}

pub(super) fn filter_persons_excluding_group(
    persons: &[PersonData],
    group: &GroupData,
) -> Vec<PersonData> {
    let member_ids = member_ids(group);
    persons
        .iter()
        .filter(|p| !member_ids.contains(&p.id))
        .cloned()
        .collect()
}

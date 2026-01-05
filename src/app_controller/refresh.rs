use std::{
    cell::RefCell,
    rc::Rc,
};

use std::collections::{HashMap, HashSet};

use rusqlite::Connection;
use slint::{ModelRc, SharedString, VecModel};

use crate::{GroupData, MainWindow, PersonData};

use crate::db_operations;

use super::filter::filter_persons_excluding_group;

// Database invariants used by the UI:
// - group id 1 is a special "Camp" group that contains all persons.
// - groups 2..=5 are methodology groups (Cub/Scout/Venture/Rover).
// - ids > 5 are user-managed groups.
const ALL_PERSONS_GROUP_ID: i32 = 1;
const FIRST_USER_MANAGED_GROUP_ID: i32 = 6;

pub(super) fn make_refresh_groups(
    app_weak: slint::Weak<MainWindow>,
    conn: Rc<RefCell<Connection>>,
    selection_groups: Rc<RefCell<Vec<GroupData>>>,
    all_persons_for_selection: Rc<RefCell<Vec<PersonData>>>,
    checked_person_ids: Rc<RefCell<HashSet<i32>>>,
    out_person_ids: Rc<RefCell<HashSet<i32>>>,
    group_members_by_id: Rc<RefCell<HashMap<i32, Vec<i32>>>>,
) -> impl Fn() + Clone + 'static {
    move || {
        let conn_ref = conn.borrow();
        let Some(app) = app_weak.upgrade() else {
            return;
        };

        let Ok(mut groups) = db_operations::get_group_with_members(&conn_ref) else {
            return;
        };

        // We'll populate `persons_list` from the special group with id = 1 ("Camp") which contains all persons.
        // `groups_list` will contain all user-manageable group names ordered by id.
        let mut persons_list: Vec<PersonData> = Vec::new();
        let mut groups_list: Vec<GroupData> = Vec::new();
        let mut group_names: Vec<SharedString> = Vec::new();

        // Order groups by id
        groups.sort_by_key(|g| g.id);

        // Update group->member_ids lookup for main screen actions
        {
            let mut map = group_members_by_id.borrow_mut();
            map.clear();
            for g in &groups {
                map.insert(g.id, g.members.iter().map(|p| p.id).collect());
            }
        }

        let groups_model: Vec<_> = groups
            .into_iter()
            .map(|mut group| {
                sort_members(&mut group.members);

                let members_vec: Vec<_> = group
                    .members
                    .into_iter()
                    .map(person_to_person_data)
                    .collect();

                // Capture global persons list from group id 1 if not yet filled
                if group.id == ALL_PERSONS_GROUP_ID && persons_list.is_empty() {
                    persons_list = members_vec.clone();
                }

                // Add user-manageable group (id > 5) to selection list WITH members for filtering
                if group.id >= FIRST_USER_MANAGED_GROUP_ID {
                    groups_list.push(GroupData {
                        id: group.id,
                        name: SharedString::from(group.name.clone()),
                        members: ModelRc::new(VecModel::from(members_vec.clone())),
                    });
                    group_names.push(SharedString::from(group.name.clone()));
                }

                GroupData {
                    id: group.id,
                    name: SharedString::from(group.name),
                    members: ModelRc::new(VecModel::from(members_vec)),
                }
            })
            .collect();

        // Cache selection data
        *selection_groups.borrow_mut() = groups_list.clone();
        *all_persons_for_selection.borrow_mut() = persons_list.clone();

        // Pre-filter persons when form is first displayed: exclude members of the first selectable group (index 0) if any.
        let initial_filtered = if let Some(first_group) = groups_list.first() {
            filter_persons_excluding_group(&persons_list, first_group)
        } else {
            persons_list.clone()
        };

        // Main screen lists (IN/OUT).
        let out_set = out_person_ids.borrow();
        let mut people_in: Vec<PersonData> = Vec::new();
        let mut people_out: Vec<PersonData> = Vec::new();
        for p in &persons_list {
            if out_set.contains(&p.id) {
                people_out.push(p.clone());
            } else {
                people_in.push(p.clone());
            }
        }

        app.set_people(ModelRc::new(VecModel::from(people_in.clone())));
        app.set_people_out(ModelRc::new(VecModel::from(people_out.clone())));

        let checked_set = checked_person_ids.borrow();
        let checked_in: Vec<bool> = people_in.iter().map(|p| checked_set.contains(&p.id)).collect();
        let checked_out: Vec<bool> = people_out.iter().map(|p| checked_set.contains(&p.id)).collect();
        app.set_people_checked(ModelRc::new(VecModel::from(checked_in)));
        app.set_people_out_checked(ModelRc::new(VecModel::from(checked_out)));
        app.set_filtered_persons_to_group(ModelRc::new(VecModel::from(initial_filtered)));
        app.set_groups(ModelRc::new(VecModel::from(groups_model)));
        app.set_persons_to_group(ModelRc::new(VecModel::from(persons_list)));
        app.set_groups_to_group(ModelRc::new(VecModel::from(groups_list)));
        app.set_groups_to_group_names(ModelRc::new(VecModel::from(group_names)));
    }
}

fn sort_members(members: &mut [db_operations::Person]) {
    use std::cmp::Ordering;

    members.sort_by(|a, b| {
        // Consistent ordering in UI lists:
        // 1) methodology order (Cub -> Rover)
        // 2) surname, case-insensitive
        // 3) name, case-insensitive
        let meth_cmp = (a.methodology as i32).cmp(&(b.methodology as i32));
        if meth_cmp != Ordering::Equal {
            return meth_cmp;
        }

        let sur_cmp = a.surname.to_lowercase().cmp(&b.surname.to_lowercase());
        if sur_cmp != Ordering::Equal {
            return sur_cmp;
        }

        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    });
}

fn person_to_person_data(p: db_operations::Person) -> PersonData {
    PersonData {
        id: p.id,
        name: SharedString::from(p.name),
        surname: SharedString::from(p.surname),
        rank: SharedString::from(p.rank_level.as_str()),
        methodology: p.methodology.as_color(),
    }
}

use std::{
    cell::RefCell,
    rc::Rc,
};

use std::collections::{HashMap, HashSet};

use chrono::Local;
use rusqlite::Connection;
use slint::{ModelRc, SharedString, VecModel};

use crate::{GroupData, LogData, LogDayGroupData, LogMinuteGroupData, MainWindow, PersonData};

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

        // Sync OUT set from DB-backed Person.is_inside using the special "all persons" group.
        if let Some(all_group) = groups.iter_mut().find(|g| g.id == ALL_PERSONS_GROUP_ID) {
            sort_members(&mut all_group.members);

            persons_list = all_group
                .members
                .clone()
                .into_iter()
                .map(person_to_person_data)
                .collect();

            let mut out_set = out_person_ids.borrow_mut();
            out_set.clear();
            for p in &all_group.members {
                if p.is_inside == db_operations::IsInside::Out {
                    out_set.insert(p.id);
                }
            }
        }

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

        // Logs screen model (person in/out events)
        if let Ok(logs) = db_operations::get_log(&conn_ref) {
            let persons_by_id: HashMap<i32, &PersonData> = persons_list.iter().map(|p| (p.id, p)).collect();

            let mut day_groups: Vec<LogDayGroupData> = Vec::new();
            let mut current_day: Option<SharedString> = None;
            let mut current_minutes: Vec<LogMinuteGroupData> = Vec::new();
            let mut current_minute: Option<SharedString> = None;
            let mut current_entries: Vec<LogData> = Vec::new();

            let flush_minute = |minute: Option<SharedString>, minutes: &mut Vec<LogMinuteGroupData>, entries: &mut Vec<LogData>| {
                if let Some(m) = minute {
                    if !entries.is_empty() {
                        minutes.push(LogMinuteGroupData {
                            minute: m,
                            entries: ModelRc::new(VecModel::from(std::mem::take(entries))),
                        });
                    }
                }
            };

            let flush_day = |day: Option<SharedString>, days: &mut Vec<LogDayGroupData>, minutes: &mut Vec<LogMinuteGroupData>| {
                if let Some(d) = day {
                    if !minutes.is_empty() {
                        days.push(LogDayGroupData {
                            day: d,
                            minutes: ModelRc::new(VecModel::from(std::mem::take(minutes))),
                        });
                    }
                }
            };

            for l in logs {
                let p = match persons_by_id.get(&l.entity_id) {
                    Some(p) => *p,
                    None => continue,
                };

                let local_time = l.time.with_timezone(&Local);
                let day = SharedString::from(local_time.format("%Y-%m-%d").to_string());
                let minute = SharedString::from(local_time.format("%H:%M").to_string());
                let seconds = SharedString::from(local_time.format("%H:%M:%S").to_string());

                match &current_day {
                    Some(cur) if *cur == day => {}
                    Some(_) => {
                        flush_minute(current_minute.take(), &mut current_minutes, &mut current_entries);
                        flush_day(current_day.take(), &mut day_groups, &mut current_minutes);
                        current_day = Some(day.clone());
                        current_minute = None;
                    }
                    None => {
                        current_day = Some(day.clone());
                    }
                }

                match &current_minute {
                    Some(cur) if *cur == minute => {}
                    Some(_) => {
                        flush_minute(current_minute.take(), &mut current_minutes, &mut current_entries);
                        current_minute = Some(minute.clone());
                    }
                    None => {
                        current_minute = Some(minute.clone());
                    }
                }

                current_entries.push(LogData {
                    person_id: p.id,
                    name: p.name.clone(),
                    surname: p.surname.clone(),
                    rank: p.rank.clone(),
                    methodology: p.methodology,
                    is_in: l.is_inside == db_operations::IsInside::In,
                    timestamp: seconds,
                });
            }

            flush_minute(current_minute.take(), &mut current_minutes, &mut current_entries);
            flush_day(current_day.take(), &mut day_groups, &mut current_minutes);

            app.set_logs(ModelRc::new(VecModel::from(day_groups)));
        }

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

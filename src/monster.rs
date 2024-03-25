use bracket_lib::prelude::*;

use crate::{combat::WantsToMelee, position::Position, Name, State, ViewShed};

#[derive(Debug)]
pub struct Monster;

pub fn apply_ai(gs: &mut State) {
    let player_pos = *gs.world.query_one_mut::<&Position>(gs.player).unwrap();

    let mut attackers = Vec::new();
    type Q<'w> = (&'w mut Position, &'w Name, &'w mut ViewShed);
    for (e, (pos, _, fov)) in gs.world.query_mut::<Q>().with::<&Monster>() {
        if fov.visible_tiles.contains(&player_pos) {
            let Some(exit) = DijkstraMap::find_lowest_exit(&gs.dm, gs.map.to_idx(*pos), &gs.map)
            else {
                continue;
            };

            if exit == gs.map.to_idx(player_pos) {
                attackers.push(e);
            } else {
                *pos = gs.map.to_pos(exit);
                fov.dirty = true;
            }
        }
    }

    for a in attackers {
        if let Err(err) = gs.world.insert_one(a, WantsToMelee { target: gs.player }) {
            gs.msg_log
                .push(format!("Error inserting Melee component: {err}"));
        }
    }
}

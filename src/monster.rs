use bracket_lib::prelude::*;
use bracket_terminal::console;
use hecs::World;

use crate::{
    combat::WantsToMelee, position::Position, BlocksTile, CombatStats, Name, Percentage,
    Renderable, State, ViewShed,
};

#[derive(Debug)]
pub struct Monster;

pub fn spawn(world: &mut World, x: i32, y: i32) {
    let mut rng = RandomNumberGenerator::new();
    let (glyph, name) = match rng.roll_dice(1, 3) {
        1 => (to_cp437('o'), Name::new("Orc")),
        _ => (to_cp437('g'), Name::new("Goblin")),
    };
    world.spawn((
        Monster,
        Position::new(x, y),
        name,
        CombatStats {
            max_hp: 16,
            hp: 16,
            defense: Percentage::new(0.1),
            power: 4,
        },
        Renderable {
            glyph,
            colors: ColorPair {
                fg: RGBA::named(RED),
                bg: RGBA::named(BLACK),
            },
        },
        ViewShed::new(6),
        BlocksTile,
    ));
}

pub fn apply_ai(gs: &mut State, _ctx: &mut BTerm) {
    let player_pos = *gs.world.query_one_mut::<&Position>(gs.player).unwrap();

    let mut attackers = Vec::new();
    type Q<'w> = (&'w mut Position, &'w Name, &'w mut ViewShed);
    for (e, (pos, _, fov)) in gs.world.query_mut::<Q>().with::<&Monster>() {
        if fov.visible_tiles.contains(&player_pos) {
            let Some(exit) =
                DijkstraMap::find_lowest_exit(&gs.dm, gs.map.point2d_to_index(pos.into()), &gs.map)
            else {
                continue;
            };

            if exit == gs.map.point2d_to_index(player_pos.into()) {
                attackers.push(e);
            } else {
                *pos = gs.map.index_to_position(exit);
                fov.dirty = true;
            }
        }
    }

    for a in attackers {
        if let Err(err) = gs.world.insert_one(a, WantsToMelee { target: gs.player }) {
            console::log(format!("Error inserting Melee component: {err}"));
        }
    }
}

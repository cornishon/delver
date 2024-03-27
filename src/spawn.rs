use bracket_lib::prelude::*;
use hecs::{Entity, World};

use crate::{
    combat::{CombatStats, Percentage},
    monster::Monster,
    position::Position,
    BlocksTile, Name, Player, Renderable, ViewShed,
};

pub fn player(world: &mut World, _rng: &mut RandomNumberGenerator, position: Position) -> Entity {
    world.spawn((
        Player,
        position,
        Name::new("Player"),
        CombatStats {
            max_hp: 100,
            hp: 100,
            accuracy: Percentage::new(0.7),
            defense: Percentage::new(0.2),
            power: 5,
        },
        Renderable {
            glyph: to_cp437('@'),
            colors: ColorPair {
                fg: RGBA::named(YELLOW),
                bg: RGBA::named(BLACK),
            },
        },
        ViewShed::new(6),
        // BlocksTile,
    ))
}

pub fn monster(world: &mut World, rng: &mut RandomNumberGenerator, position: Position) -> Entity {
    let (glyph, name) = match rng.roll_dice(1, 3) {
        1 => (to_cp437('o'), Name::new("Orc")),
        _ => (to_cp437('g'), Name::new("Goblin")),
    };
    world.spawn((
        Monster,
        position,
        name,
        CombatStats {
            max_hp: 16,
            hp: 16,
            accuracy: Percentage::new(0.5),
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
    ))
}

pub fn fill_room(world: &mut World, rng: &mut RandomNumberGenerator, room: Rect) {
    let x = rng.range(room.x1 + 1, room.x2);
    let y = rng.range(room.y1 + 1, room.y2);
    monster(world, rng, Position::new(x, y));
}

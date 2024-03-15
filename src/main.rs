use std::collections::HashSet;

use bracket_lib::prelude::*;
use hecs::World;

use crate::map::Map;

mod grid;
mod map;

#[derive(Debug, Default)]
struct Position {
    x: i32,
    y: i32,
}

impl From<&Position> for Point {
    fn from(&Position { x, y }: &Position) -> Self {
        Point { x, y }
    }
}
impl From<&Point> for Position {
    fn from(&Point { x, y }: &Point) -> Self {
        Position { x, y }
    }
}

#[derive(Debug)]
struct Renderable {
    glyph: FontCharType,
    colors: ColorPair,
}

#[derive(Debug)]
struct Player;

#[derive(Debug)]
struct ViewShed {
    visible_tiles: HashSet<Point>,
    range: i32,
    dirty: bool,
}

impl ViewShed {
    fn new(range: i32) -> Self {
        Self {
            visible_tiles: Default::default(),
            range,
            dirty: true,
        }
    }
}

struct State {
    world: World,
    map: Map,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.compute_visibility();

        let mut draw_batch = DrawBatch::new();
        draw_batch.cls();

        self.map.draw(&mut draw_batch);

        for (_, (pos, render)) in self.world.query_mut::<(&Position, &Renderable)>() {
            if self.map.visible_tiles[pos.into()] {
                draw_batch.set(pos.into(), render.colors, render.glyph);
            }
        }

        draw_batch.submit(0).expect("Draw Batch");
        render_draw_buffer(ctx).expect("Render Buffer");

        self.player_input(ctx);
    }
}

impl State {
    fn player_input(&mut self, ctx: &BTerm) {
        use VirtualKeyCode as Key;
        match ctx.key {
            Some(Key::H | Key::A | Key::Left) => self.try_move_player(-1, 0),
            Some(Key::J | Key::S | Key::Down) => self.try_move_player(0, 1),
            Some(Key::K | Key::W | Key::Up) => self.try_move_player(0, -1),
            Some(Key::L | Key::D | Key::Right) => self.try_move_player(1, 0),
            _ => {}
        }
    }

    fn try_move_player(&mut self, dx: i32, dy: i32) {
        for (_, (pos, fov)) in self
            .world
            .query_mut::<(&mut Position, &mut ViewShed)>()
            .with::<&Player>()
        {
            let new_x = pos.x + dx;
            let new_y = pos.y + dy;
            if self.map.is_passable(new_x, new_y) {
                pos.x = new_x;
                pos.y = new_y;
                fov.dirty = true;
            }
        }
    }

    fn compute_visibility(&mut self) {
        for (_, (fov, pos, player)) in self
            .world
            .query_mut::<(&mut ViewShed, &Position, Option<&Player>)>()
            .into_iter()
            .filter(|(_, (fov, _, _))| fov.dirty)
        {
            fov.dirty = false;
            fov.visible_tiles = field_of_view_set(pos.into(), fov.range, &self.map);
            fov.visible_tiles
                .retain(|p| self.map.tiles.try_to_idx(p.x, p.y).is_some());
            if player.is_some() {
                self.map.visible_tiles.reset();
                for &p in &fov.visible_tiles {
                    self.map.revealed_tiles.set(p.x, p.y, true);
                    self.map.visible_tiles.set(p.x, p.y, true);
                }
            }
        }
    }
}

const CONSOLE_WIDTH: i32 = 60;
const CONSOLE_HEIGHT: i32 = 40;

fn main() -> BError {
    let bterm = BTermBuilder::simple(CONSOLE_WIDTH, CONSOLE_HEIGHT)?
        .with_title("Roguelike")
        .with_tile_dimensions(16, 16)
        .build()?;

    let map = Map::new(CONSOLE_WIDTH as usize, CONSOLE_HEIGHT as usize);
    let mut world = World::new();
    let mut rng = RandomNumberGenerator::new();

    world.spawn((
        Player,
        Position::from(&map.rooms[0].center()),
        Renderable {
            glyph: to_cp437('@'),
            colors: ColorPair {
                fg: RGBA::named(YELLOW),
                bg: RGBA::named(BLACK),
            },
        },
        ViewShed::new(8),
    ));

    for room in &map.rooms[1..] {
        let x = rng.range(room.x1 + 1, room.x2);
        let y = rng.range(room.y1 + 1, room.y2);
        world.spawn((
            Position { x, y },
            Renderable {
                glyph: to_cp437('☺'),
                colors: ColorPair {
                    fg: RGBA::named(RED),
                    bg: RGBA::named(BLACK),
                },
            },
        ));
    }

    main_loop(bterm, State { world, map })
}

use bracket_lib::prelude::*;
use hecs::World;

use crate::map::{xy_idx, Map, TileType};

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

struct State {
    world: World,
    map: Map,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        let mut draw_batch = DrawBatch::new();
        draw_batch.cls();

        self.map.draw(&mut draw_batch);

        for (_, (pos, render)) in self.world.query_mut::<(&Position, &Renderable)>() {
            draw_batch.set(pos.into(), render.colors, render.glyph);
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
        for (_, pos) in self.world.query_mut::<&mut Position>().with::<&Player>() {
            let new_x = (pos.x + dx).clamp(0, CONSOLE_WIDTH - 1);
            let new_y = (pos.y + dy).clamp(0, CONSOLE_HEIGHT - 1);
            if self.map.tiles[xy_idx(new_x, new_y)] != TileType::Wall {
                pos.x = new_x;
                pos.y = new_y;
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

    let map = Map::new();
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
    ));

    for room in &map.rooms[1..] {
        let x = rng.range(room.x1 + 2, room.x2 - 1);
        let y = rng.range(room.y1 + 2, room.y2 - 1);
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

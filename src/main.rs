use std::collections::HashSet;

use bracket_lib::pathfinding::Algorithm2D;
use bracket_lib::prelude::*;
use hecs::World;

use crate::map::Map;
use crate::position::Position;

mod map;
mod position;

#[derive(Debug)]
struct Renderable {
    glyph: FontCharType,
    colors: ColorPair,
}

#[derive(Debug)]
struct Player;

#[derive(Debug)]
struct Monster;

#[derive(Debug)]
struct ViewShed {
    visible_tiles: HashSet<Position>,
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

#[derive(Debug, Default, Clone, Copy)]
pub enum RunState {
    #[default]
    Paused,
    Running,
}

struct State {
    world: World,
    map: Map,
    run_state: RunState,
    player: hecs::Entity,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        if self.player_input(ctx) {
            self.run_state = RunState::Running;
        }
        match self.run_state {
            RunState::Paused => {}
            RunState::Running => {
                self.compute_visibility();

                for (e, pos) in self.world.query_mut::<&Position>().with::<&Monster>() {
                    if self.map.visible_tiles[pos.into()] {
                        println!("{e:?} at {pos:?}");
                    }
                }

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

                self.run_state = RunState::Paused;
            }
        }
    }
}

impl State {
    fn new(map: Map) -> Self {
        Self {
            map,
            world: Default::default(),
            run_state: Default::default(),
            player: hecs::Entity::DANGLING,
        }
    }

    fn player_input(&mut self, ctx: &BTerm) -> bool {
        use VirtualKeyCode as Key;
        match ctx.key {
            Some(Key::H | Key::A | Key::Left) => self.try_move_player(-1, 0),
            Some(Key::J | Key::S | Key::Down) => self.try_move_player(0, 1),
            Some(Key::K | Key::W | Key::Up) => self.try_move_player(0, -1),
            Some(Key::L | Key::D | Key::Right) => self.try_move_player(1, 0),
            _ => false,
        }
    }

    fn try_move_player(&mut self, dx: i8, dy: i8) -> bool {
        let mut moved = false;
        for (_, (pos, fov)) in self
            .world
            .query_mut::<(&mut Position, &mut ViewShed)>()
            .with::<&Player>()
        {
            let new_pos = *pos + Point::new(dx, dy);
            if self.map.is_passable(new_pos.into()) {
                *pos = new_pos;
                fov.dirty = true;
                moved = true;
            }
        }
        moved
    }

    fn compute_visibility(&mut self) {
        for (_, (fov, pos, player)) in self
            .world
            .query_mut::<(&mut ViewShed, &Position, Option<&Player>)>()
            .into_iter()
            .filter(|(_, (fov, _, _))| fov.dirty)
        {
            fov.dirty = false;
            fov.visible_tiles = field_of_view_set(pos.into(), fov.range, &self.map)
                .into_iter()
                .filter_map(|p| Position::try_from(&p).ok())
                .collect();
            fov.visible_tiles.retain(|p| self.map.in_bounds(p.into()));
            if player.is_some() {
                self.map.visible_tiles.fill(false);
                for &p in &fov.visible_tiles {
                    self.map.revealed_tiles[p.into()] = true;
                    self.map.visible_tiles[p.into()] = true;
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
    let mut game_state = State::new(map);
    let mut rng = RandomNumberGenerator::new();

    game_state.player = game_state.world.spawn((
        Player,
        Position::try_from(&game_state.map.rooms[0].center()).unwrap(),
        Renderable {
            glyph: to_cp437('@'),
            colors: ColorPair {
                fg: RGBA::named(YELLOW),
                bg: RGBA::named(BLACK),
            },
        },
        ViewShed::new(6),
    ));

    for room in &game_state.map.rooms[1..] {
        let x = rng.range(room.x1 + 1, room.x2);
        let y = rng.range(room.y1 + 1, room.y2);
        game_state.world.spawn((
            Monster,
            Position::new(x, y),
            Renderable {
                glyph: to_cp437(match rng.roll_dice(1, 3) {
                    1 => 'o',
                    _ => 'g',
                }),
                colors: ColorPair {
                    fg: RGBA::named(RED),
                    bg: RGBA::named(BLACK),
                },
            },
            ViewShed::new(6),
        ));
    }

    main_loop(bterm, game_state)
}

use std::collections::HashSet;

use crate::combat::{CombatStats, Percentage, WantsToMelee};
use crate::map::Map;
use crate::position::Position;
use bracket_lib::pathfinding::Algorithm2D;
use bracket_lib::prelude::*;
use hecs::{Entity, World};
use map::TileType;

mod combat;
mod map;
mod monster;
mod position;
mod ui;

#[derive(Debug)]
struct Renderable {
    glyph: FontCharType,
    colors: ColorPair,
}

#[derive(Debug)]
pub struct Name(pub String);
impl Name {
    pub fn new(name: impl ToString) -> Self {
        Self(name.to_string())
    }
}
impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug)]
struct Player;

#[derive(Debug)]
struct BlocksTile;

#[derive(Debug)]
struct ViewShed {
    visible_tiles: HashSet<Position>,
    range: u16,
    dirty: bool,
}

pub enum PlayerMove {
    Move,
    Attack(Entity),
    None,
}

impl ViewShed {
    fn new(range: u16) -> Self {
        Self {
            visible_tiles: Default::default(),
            range,
            dirty: true,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Phase {
    #[default]
    Startup,
    AwaitingInput,
    PlayerTurn,
    MonsterTurn,
}

struct State {
    world: World,
    map: Map,
    dm: DijkstraMap,
    phase: Phase,
    player: hecs::Entity,
    msg_log: Vec<String>,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.phase {
            Phase::Startup => {
                self.compute_visibility();
                self.update_map();
                self.render(ctx);
                self.msg_log.push("Welcome to the game.".into());
                ui::draw_ui(self, ctx);
                self.phase = Phase::AwaitingInput;
            }
            Phase::AwaitingInput => {
                if self.player_input(ctx) {
                    self.phase = Phase::PlayerTurn;
                } else if ctx.key == Some(VirtualKeyCode::M) {
                    for (idx, tile) in self.map.tiles.iter().enumerate() {
                        let d = self.dm.map[idx];
                        if *tile == TileType::Floor && d > 0.5 && d < 10.0 {
                            let mp = self.map.to_pos(idx);
                            ctx.print(mp.x, mp.y, d);
                        }
                    }
                }
            }
            Phase::PlayerTurn => {
                self.compute_visibility();
                self.compute_dijkstra_map();
                combat::run(self);
                self.update_map();
                self.phase = Phase::MonsterTurn;
            }
            Phase::MonsterTurn => {
                self.compute_visibility();
                self.compute_dijkstra_map();
                monster::apply_ai(self, ctx);
                combat::run(self);
                self.update_map();
                self.phase = Phase::AwaitingInput;

                self.render(ctx);
                ui::draw_ui(self, ctx);
            }
        }
    }
}

impl State {
    pub fn new(map: Map) -> Self {
        Self {
            dm: DijkstraMap::new_empty(map.width, map.height, 100.0),
            map,
            world: Default::default(),
            phase: Default::default(),
            player: hecs::Entity::DANGLING,
            msg_log: Default::default(),
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
        let mut target = None;

        type Q<'w> = (&'w mut Position, &'w mut ViewShed);
        for (_, (pos, fov)) in self.world.query::<Q>().with::<&Player>().iter() {
            let new_pos = *pos + Point::new(dx, dy);

            for e in &self.map.entities[new_pos.into()] {
                if let Ok(true) = self.world.satisfies::<&CombatStats>(*e) {
                    target = Some(*e);
                    moved = true;
                }
            }

            if self.map.is_passable(new_pos) {
                self.map.blocked[pos.into()] = false;
                *pos = new_pos;
                fov.dirty = true;
                moved = true;
            }
        }

        if let Some(target) = target {
            self.world
                .insert_one(self.player, WantsToMelee { target })
                .expect("Player exists");
        }

        moved
    }

    fn compute_visibility(&mut self) {
        for (e, (fov, pos)) in self
            .world
            .query_mut::<(&mut ViewShed, &Position)>()
            .into_iter()
            .filter(|(_, (fov, _))| fov.dirty)
        {
            fov.dirty = false;
            fov.visible_tiles = field_of_view_set(pos.into(), fov.range.into(), &self.map)
                .into_iter()
                .filter(|&p| self.map.in_bounds(p))
                .filter(|&p| DistanceAlg::Manhattan.distance2d(pos.into(), p) <= fov.range as f32)
                .filter_map(|p| Position::try_from(&p).ok())
                .collect();

            if e == self.player {
                self.map.visible.fill(false);
                for &p in &fov.visible_tiles {
                    self.map.revealed[p.into()] = true;
                    self.map.visible[p.into()] = true;
                }
            }
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        let mut draw_batch = DrawBatch::new();
        draw_batch.target(0);
        draw_batch.cls();

        self.map.draw(&mut draw_batch);

        for (_, (pos, render)) in self.world.query_mut::<(&Position, &Renderable)>() {
            if self.map.visible[pos.into()] {
                draw_batch.set(pos.into(), render.colors, render.glyph);
            }
        }

        draw_batch.submit(0).expect("Draw Batch");
        render_draw_buffer(ctx).expect("Render Buffer");
    }

    fn update_map(&mut self) {
        let mut blocked = self.map.new_grid();
        for (idx, entry) in blocked.iter_mut().enumerate() {
            *entry = self.map.is_opaque(idx);
        }
        self.map.clear_entities();
        for (e, pos) in self.world.query_mut::<&Position>().with::<&BlocksTile>() {
            blocked[pos.into()] = true;
            self.map.entities[pos.into()].push(e);
        }
        self.map.blocked = blocked;
    }

    fn compute_dijkstra_map(&mut self) {
        let player_pos = *self.world.query_one_mut::<&Position>(self.player).unwrap();
        let player_pos = self.map.to_idx(player_pos);
        self.dm = DijkstraMap::new(
            self.map.width,
            self.map.height,
            &[player_pos],
            &self.map,
            100.0,
        );
        // have to manually set player position to <1.0, as we use this dijkstra map
        // do decide monster target, and DijkstraMap::new sets it to 2.0 (move away, then move back)
        self.dm.map[player_pos] = 0.6;
    }
}

const CONSOLE_WIDTH: i32 = 60;
const CONSOLE_HEIGHT: i32 = 42;
const UI_HEIGHT: i32 = 10;

fn main() -> BError {
    let bterm = BTermBuilder::simple(CONSOLE_WIDTH, CONSOLE_HEIGHT)?
        .with_title("Roguelike")
        .with_tile_dimensions(16, 16)
        .build()?;

    let map = Map::new(
        CONSOLE_WIDTH as usize,
        (CONSOLE_HEIGHT - UI_HEIGHT) as usize,
    );
    let mut gs = State::new(map);
    let mut rng = RandomNumberGenerator::new();

    gs.player = gs.world.spawn((
        Player,
        Position::try_from(&gs.map.rooms[0].center()).unwrap(),
        Name::new("Player"),
        CombatStats {
            max_hp: 30,
            hp: 30,
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
    ));

    for room in &gs.map.rooms[1..] {
        let x = rng.range(room.x1 + 1, room.x2);
        let y = rng.range(room.y1 + 1, room.y2);
        monster::spawn(&mut gs.world, x, y);
    }

    main_loop(bterm, gs)
}

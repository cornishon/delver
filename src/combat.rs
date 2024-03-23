use bracket_lib::random::RandomNumberGenerator;
use bracket_terminal::console;
use hecs::{Entity, World};

use crate::{Name, State};

#[derive(Debug, Clone, Copy)]
pub struct Percentage(f32);
impl Percentage {
    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: Percentage,
    pub power: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Debug, Clone)]
pub struct SufferDamage {
    pub queue: Vec<i32>,
}

impl SufferDamage {
    pub fn new(amount: i32) -> Self {
        Self {
            queue: vec![amount],
        }
    }
    pub fn add_damage(world: &mut World, victim: Entity, amount: i32) {
        if let Ok(suffering) = world.query_one_mut::<&mut SufferDamage>(victim) {
            suffering.queue.push(amount);
        } else if let Err(err) = world.insert_one(victim, SufferDamage::new(amount)) {
            console::log(format!("Error inserting Damage component: {err}"));
        }
    }
}

pub fn melee_combat(gs: &mut State) {
    let mut rng = RandomNumberGenerator::new();
    let mut to_damage = Vec::new();
    let mut attackers = Vec::new();
    for (e, (wants_melee, name, stats)) in gs
        .world
        .query::<(&WantsToMelee, &Name, &CombatStats)>()
        .iter()
    {
        attackers.push(e);
        if stats.hp <= 0 {
            continue;
        }
        let Ok(mut target) = gs
            .world
            .query_one::<(&CombatStats, &Name)>(wants_melee.target)
        else {
            continue;
        };
        if let Some((target_stats, target_name)) = target.get() {
            if target_stats.hp <= 0 {
                continue;
            }
            let damage = stats.power as f32 * (1.0 - target_stats.defense.0);
            let (mut damage, fractional) = (damage as i32, damage.fract());
            if rng.range(0.0, 1.0) > fractional {
                damage += 1;
            }
            if damage == 0 {
                console::log(format!("{name} is unable to damage {target_name}"));
            } else {
                console::log(format!("{name} hits {target_name} for {}", damage));
                to_damage.push((wants_melee.target, damage));
            }
        }
    }
    for (target, damage) in to_damage {
        SufferDamage::add_damage(&mut gs.world, target, damage);
    }
    for e in attackers {
        if let Err(err) = gs.world.remove_one::<WantsToMelee>(e) {
            console::log(format!("Error inserting Melee component: {err}"))
        }
    }
}

pub fn apply_damage(gs: &mut State) {
    let mut victims = Vec::new();
    for (e, (combat_stats, suffer_damage)) in
        gs.world.query_mut::<(&mut CombatStats, &SufferDamage)>()
    {
        combat_stats.hp -= suffer_damage.queue.iter().sum::<i32>();
        victims.push(e);
    }
    for e in victims {
        if let Err(err) = gs.world.remove_one::<SufferDamage>(e) {
            console::log(format!("Error removing Damage component: {err}"));
        }
    }
}

pub fn delete_the_dead(gs: &mut State) {
    let mut dead = Vec::new();
    for (e, stats) in gs.world.query_mut::<&CombatStats>() {
        if stats.hp <= 0 {
            dead.push(e);
        }
    }
    for e in dead {
        _ = gs.world.despawn(e);
    }
}

pub fn run(gs: &mut State) {
    melee_combat(gs);
    apply_damage(gs);
    delete_the_dead(gs);
}

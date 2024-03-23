use bracket_lib::terminal::{BTerm, BLACK, RED, WHITE, YELLOW};

use crate::{combat::CombatStats, State, CONSOLE_HEIGHT, CONSOLE_WIDTH, UI_HEIGHT};

pub fn draw_ui(gs: &mut State, ctx: &mut BTerm) {
    const PADDING: i32 = 1;

    let mut y = CONSOLE_HEIGHT - UI_HEIGHT;
    ctx.draw_box(0, y, CONSOLE_WIDTH - 1, UI_HEIGHT - 1, WHITE, BLACK);

    if let Ok(&CombatStats { max_hp, hp, .. }) = gs.world.query_one_mut::<&CombatStats>(gs.player) {
        let health = format!(" HP: {:2} / {:2} ", hp, max_hp);
        ctx.print_color(PADDING, y, YELLOW, BLACK, health);
        ctx.draw_bar_horizontal(15, y, 43, hp, max_hp, RED, BLACK);
    }

    for msg in &gs.msg_log[gs.msg_log.len().saturating_sub(UI_HEIGHT as usize - 2)..] {
        y += 1;
        ctx.print(PADDING, y, msg);
    }
}

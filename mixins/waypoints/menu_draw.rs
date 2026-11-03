// The menu: the top buttons, then a row per waypoint with its colour, name,
// coordinates and distance, and Rename and delete buttons.
fn waypoints_menu_draw(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
) {
    use crate::game::mod_waypoints as m;
    if screen.id != "mod" || game.native_mod_screen() != Some(m::ID) {
        return;
    }
    let [_, _, w, h] = screen.rect;
    let (store, ui, settings) = m::with(|k| game.native_setting(m::ID, k), |s| (s.store.clone(), s.ui.clone(), s.settings()));
    let layout = m::Layout::new(w, h, store.list.len(), ui.scroll);
    let mouse = screen.mouse;
    let player = game.native_player();
    game.native_fill(batch, [0.0, 0.0, w, h], 0x9000_0000);
    game.native_fill(batch, layout.panel, 0xC010_1010);
    let title = format!("{} {}", m::NAME, m::VERSION);
    let tw = game.native_text_width(&title);
    game.native_text(batch, &title, [(w - tw) / 2.0, layout.title_y], m::INK, true);
    let button = |batch: &mut horizon_client::hud::HudBatch, rect: [f32; 4], label: &str, ink: u32| {
        let hover = m::inside(rect, mouse);
        game.native_fill(batch, rect, if hover { 0xFF60_6060 } else { 0xFF38_3838 });
        game.native_fill(batch, [rect[0], rect[1] + rect[3] - 1.0, rect[2], 1.0], 0xFF20_2020);
        let lw = game.native_text_width(label);
        game.native_text(batch, label, [rect[0] + (rect[2] - lw) / 2.0, rect[1] + 3.0], ink, true);
    };
    for &(b, rect) in &layout.buttons {
        match b {
            m::Button::AddHere | m::Button::Shown | m::Button::Markers => {
                button(batch, rect, &m::button_label(b, settings), m::INK);
            }
            m::Button::Rename(_) => button(batch, rect, "Rename", m::INK),
            m::Button::Delete(i) => {
                let sure = ui.confirm_delete == Some(i);
                button(batch, rect, if sure { "?" } else { "x" }, if sure { 0xFF_5555 } else { m::GREY });
            }
            m::Button::Color(i) => {
                let color = store.list.get(i).map_or(m::INK, |wp| wp.color);
                let ring = if m::inside(rect, mouse) { 0xFFFF_FFFF } else { 0xFF00_0000 };
                game.native_fill(batch, [rect[0] - 1.0, rect[1] - 1.0, rect[2] + 2.0, rect[3] + 2.0], ring);
                game.native_fill(batch, rect, 0xFF00_0000 | color);
            }
            m::Button::Name(_) => {}
        }
    }
    for &(i, row) in &layout.rows {
        let Some(wp) = store.list.get(i) else {
            continue;
        };
        if m::inside(row, mouse) {
            game.native_fill(batch, row, 0x30FF_FFFF);
        }
        let rename_x = layout
            .buttons
            .iter()
            .find(|b| b.0 == m::Button::Rename(i))
            .map_or(row[0] + row[2], |b| b.1[0]);
        let [x, y, z] = wp.pos;
        let mut info = format!("{x} {y} {z}");
        if let Some(p) = &player {
            if p.dimension == wp.dimension {
                info = format!("{info}  {}", m::distance_label(m::distance([p.x, p.y, p.z], wp.target())));
            } else {
                info = format!("{info}  {}", wp.dimension.trim_start_matches("minecraft:").replace('_', " "));
            }
        }
        let iw = game.native_text_width(&info);
        game.native_text(batch, &info, [rename_x - 6.0 - iw, row[1] + 4.0], m::GREY, true);
        let name_x = row[0] + 16.0;
        let room = rename_x - 12.0 - iw - name_x;
        match &ui.editing {
            Some((j, text, fresh)) if *j == i => {
                let field = [name_x - 2.0, row[1] + 1.0, room + 4.0, m::BUTTON_H];
                game.native_fill(batch, field, 0xFF00_0000);
                game.native_fill(batch, [field[0], field[1] + field[3] - 1.0, field[2], 1.0], 0xFFFF_FFFF);
                let shown = m::fit(text, room - 6.0, |t| game.native_text_width(t));
                let ink = if *fresh { m::GREY } else { m::INK };
                game.native_text(batch, &shown, [name_x, row[1] + 4.0], ink, false);
                let cx = name_x + game.native_text_width(&shown) + 1.0;
                game.native_text(batch, "_", [cx, row[1] + 4.0], m::INK, false);
            }
            _ => {
                let shown = m::fit(&wp.name, room, |t| game.native_text_width(t));
                game.native_text(batch, &shown, [name_x, row[1] + 4.0], m::INK, true);
            }
        }
    }
    let empty = "No waypoints yet: + Add here, or N while playing";
    if store.list.is_empty() {
        let ew = game.native_text_width(empty);
        game.native_text(batch, empty, [(w - ew) / 2.0, layout.footer_y - 40.0], m::GREY, true);
    }
    let more = store.list.len().saturating_sub(layout.fits);
    let footer = if ui.editing.is_some() {
        "Type a name, Enter keeps it, Esc leaves it".to_owned()
    } else if more > 0 {
        format!("Scroll for more ({}) - colour square: next colour - Esc closes", store.list.len())
    } else {
        "Click the colour square for the next colour - Esc closes".to_owned()
    };
    let fw = game.native_text_width(&footer);
    game.native_text(batch, &footer, [(w - fw) / 2.0, layout.footer_y], m::GREY, true);
}

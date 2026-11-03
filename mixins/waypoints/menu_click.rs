// A click on the menu: its buttons, rows and swatches.
fn waypoints_menu_click(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    use crate::game::mod_waypoints as m;
    if screen.id != "mod" || game.native_mod_screen() != Some(m::ID) {
        return;
    }
    *canceled = true;
    if button != 0 {
        return;
    }
    let here = game.native_player().map(|p| (p.dimension, m::block_of([p.x, p.y, p.z])));
    let [_, _, w, h] = screen.rect;
    let files = m::with(|k| game.native_setting(m::ID, k), |s| {
        let layout = m::Layout::new(w, h, s.store.list.len(), s.ui.scroll);
        s.click(layout.hit(screen.mouse), here).then(|| s.files())
    });
    for (key, value) in files.unwrap_or_default() {
        game.native_set_setting(m::ID, key, &value);
    }
}

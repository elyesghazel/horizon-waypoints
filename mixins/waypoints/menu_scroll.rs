// The wheel scrolls the menu's list.
fn waypoints_menu_scroll(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    scroll: [f64; 2],
    canceled: &mut bool,
) {
    use crate::game::mod_waypoints as m;
    if screen.id != "mod" || game.native_mod_screen() != Some(m::ID) || scroll[1] == 0.0 {
        return;
    }
    *canceled = true;
    let [_, _, w, h] = screen.rect;
    m::with(|k| game.native_setting(m::ID, k), |s| {
        let fits = m::Layout::new(w, h, s.store.list.len(), s.ui.scroll).fits;
        s.scroll(scroll[1] > 0.0, fits);
    });
}

// A key on the menu: typing a name, Enter, Escape, Backspace, and the menu
// key to close it. Escape with nothing being renamed closes the menu.
fn waypoints_menu_key(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    key: &str,
    canceled: &mut bool,
) {
    use crate::game::mod_waypoints as m;
    if screen.id != "mod" || game.native_mod_screen() != Some(m::ID) {
        return;
    }
    let menu_key = game.native_mod_key(m::ID, m::KEYS[2][0], m::KEYS[2][2]);
    let (taken, files) = m::with(|k| game.native_setting(m::ID, k), |s| {
        let (taken, save) = s.menu_key(key, &menu_key);
        (taken, save.then(|| s.files()))
    });
    *canceled = taken == m::MenuKey::Taken;
    for (k, value) in files.unwrap_or_default() {
        game.native_set_setting(m::ID, k, &value);
    }
}

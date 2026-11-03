// A character typed on the menu goes into the name being renamed.
fn waypoints_menu_char(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    c: char,
    canceled: &mut bool,
) {
    use crate::game::mod_waypoints as m;
    if screen.id != "mod" || game.native_mod_screen() != Some(m::ID) {
        return;
    }
    *canceled = true;
    m::with(|k| game.native_setting(m::ID, k), |s| s.typed(c));
}

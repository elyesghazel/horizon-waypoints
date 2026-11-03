// Closes the menu when a click or its key asked to: the screen hooks can't.
fn waypoints_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::mod_waypoints as m;
    if !game.native_window() || !game.native_on(m::ID) {
        return;
    }
    let close = {
        let g = &*game;
        m::with(|k| g.native_setting(m::ID, k), |s| std::mem::take(&mut s.ui.close))
    };
    if close && game.native_mod_screen() == Some(m::ID) {
        game.native_mod_screen_close(m::ID);
    }
}

// A click on the mod's settings page: change the setting and keep it.
fn waypoints_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::mod_waypoints as m;
    if mod_id != m::ID {
        return;
    }
    let files = {
        let g = &*game;
        m::with(|k| g.native_setting(m::ID, k), |s| {
            let mut settings = s.settings();
            settings.click(button);
            s.settings = Some(settings);
            s.files()
        })
    };
    for (key, value) in files {
        game.native_set_setting(m::ID, key, &value);
    }
}

// The Add key saves a waypoint where you stand; the Toggle key hides or
// shows them all.
fn waypoints_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::mod_waypoints as m;
    if !pressed || !game.native_on(m::ID) || !game.native_playing() {
        return;
    }
    let add = key == game.native_mod_key(m::ID, m::KEYS[0][0], m::KEYS[0][2]);
    let toggle = key == game.native_mod_key(m::ID, m::KEYS[1][0], m::KEYS[1][2]);
    if !add && !toggle {
        return;
    }
    let Some(p) = game.native_player() else {
        return;
    };
    let (line, files) = {
        let g = &*game;
        m::with(|k| g.native_setting(m::ID, k), |s| {
            let line = if add {
                s.add_here("", None, p.dimension, m::block_of([p.x, p.y, p.z]))
            } else {
                s.toggle()
            };
            (line, s.files())
        })
    };
    for (key, value) in files {
        game.native_set_setting(m::ID, key, &value);
    }
    game.native_chat(&line);
}

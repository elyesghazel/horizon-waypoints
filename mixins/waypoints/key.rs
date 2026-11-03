// N saves a waypoint where you stand and opens the menu to name it; U opens
// the menu; the Toggle key hides or shows them all.
fn waypoints_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::mod_waypoints as m;
    if !pressed || !game.native_on(m::ID) || !game.native_playing() {
        return;
    }
    let add = key == game.native_mod_key(m::ID, m::KEYS[0][0], m::KEYS[0][2]);
    let toggle = key == game.native_mod_key(m::ID, m::KEYS[1][0], m::KEYS[1][2]);
    let menu = key == game.native_mod_key(m::ID, m::KEYS[2][0], m::KEYS[2][2]);
    if !add && !toggle && !menu {
        return;
    }
    let Some(p) = game.native_player() else {
        return;
    };
    if menu {
        if game.native_mod_screen_open(m::ID) {
            let g = &*game;
            m::with(|k| g.native_setting(m::ID, k), |s| s.open_menu(None, key));
        }
        return;
    }
    let (line, files, new) = {
        let g = &*game;
        m::with(|k| g.native_setting(m::ID, k), |s| {
            let line = if add {
                s.add_here("", None, p.dimension, m::block_of([p.x, p.y, p.z]))
            } else {
                s.toggle()
            };
            let new = add.then(|| s.store.list.len() - 1);
            (line, s.files(), new)
        })
    };
    for (k, value) in files {
        game.native_set_setting(m::ID, k, &value);
    }
    // A new waypoint opens the menu with its name ready to type over.
    if let Some(i) = new
        && game.native_mod_screen_open(m::ID)
    {
        let g = &*game;
        m::with(|k| g.native_setting(m::ID, k), |s| s.open_menu(Some(i), key));
        return;
    }
    game.native_chat(&line);
}

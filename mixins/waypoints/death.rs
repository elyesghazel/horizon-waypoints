// A death adds (or moves) the Death waypoint where it happened. Reads the
// health the hearts show; draws nothing, so Horizon's own rows stay.
fn waypoints_death(game: &mut crate::game::Game<'_>, inputs: &horizon_client::hud_layout::HealthInputs) {
    use crate::game::mod_waypoints as m;
    if !game.native_on(m::ID) {
        return;
    }
    let Some(p) = game.native_player() else {
        return;
    };
    let health = inputs.health;
    let found = {
        let g = &*game;
        m::with(|k| g.native_setting(m::ID, k), |s| {
            (s.died(health) && s.settings().deaths).then(|| {
                let line = s.add_here(m::DEATH_NAME, Some(m::PALETTE[0].0), p.dimension, m::block_of([p.x, p.y, p.z]));
                (line, s.files())
            })
        })
    };
    if let Some((line, files)) = found {
        for (key, value) in files {
            game.native_set_setting(m::ID, key, &value);
        }
        game.native_chat(&line);
    }
}

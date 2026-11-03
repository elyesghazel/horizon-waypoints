// /wp add, remove, list, toggle and clear.
fn waypoints_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    use crate::game::mod_waypoints as m;
    if !game.native_on(m::ID) {
        return;
    }
    let Some(command) = m::parse(command) else {
        return;
    };
    *ran = true;
    let Some(p) = game.native_player() else {
        game.native_chat("Not in a world");
        return;
    };
    let at = [p.x, p.y, p.z];
    let (lines, files) = {
        let g = &*game;
        m::with(|k| g.native_setting(m::ID, k), |s| {
            let lines = match command {
                m::Command::Add { name, color } => vec![s.add_here(&name, color, p.dimension, m::block_of(at))],
                m::Command::Remove(which) => vec![match s.store.remove(&which) {
                    Some(name) => format!("Removed {name}"),
                    None => format!("No waypoint {which}: /wp list shows them"),
                }],
                m::Command::List => s.list_lines(p.dimension, at),
                m::Command::Clear => {
                    s.store.list.clear();
                    vec!["Removed every waypoint".to_owned()]
                }
                m::Command::Toggle => vec![s.toggle()],
                m::Command::Help => m::HELP.iter().map(|l| (*l).to_owned()).collect(),
            };
            (lines, s.files())
        })
    };
    for (key, value) in files {
        game.native_set_setting(m::ID, key, &value);
    }
    for line in lines {
        game.native_chat(&line);
    }
}

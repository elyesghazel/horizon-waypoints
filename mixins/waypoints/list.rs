// The mod's row in Options > Mods, with its two keys.
fn waypoints_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::mod_waypoints as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: m::KEYS.iter().map(|k| k.map(str::to_owned)).collect(),
    });
}

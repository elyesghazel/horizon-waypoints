// The markers over each waypoint in the world, and the list of the nearest
// ones in a corner, each with an arrow that points at it.
fn waypoints_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use crate::game::mod_waypoints as m;
    use horizon_client::extras::Corner;
    if !game.native_on(m::ID) || !game.native_playing() {
        return;
    }
    let Some(p) = game.native_player() else {
        return;
    };
    let at = [p.x, p.y, p.z];
    let (settings, rows, camera) = {
        let g = &*game;
        m::with(|k| g.native_setting(m::ID, k), |s| {
            let rows: Vec<(m::Waypoint, f64)> =
                s.store.nearest(p.dimension, at).into_iter().map(|(w, d)| (w.clone(), d)).collect();
            (s.settings(), rows, s.camera)
        })
    };
    if !settings.shown || rows.is_empty() {
        return;
    }
    let size = [batch.width, batch.height];
    if settings.markers
        && let Some((camera, fov)) = camera
    {
        // The farthest first, so nearer labels draw over them.
        for (w, d) in rows.iter().rev().filter(|r| r.1 > 2.0) {
            let Some([x, y]) = m::project(camera, p.yaw, p.pitch, fov, size, w.target()) else {
                continue;
            };
            if x < 0.0 || y < 0.0 || x > size[0] || y > size[1] {
                continue;
            }
            let label = format!("{} {}", w.name, m::distance_label(*d));
            let tw = game.native_text_width(&label);
            game.native_quad(batch, [[x, y - 4.0], [x + 4.0, y], [x, y + 4.0], [x - 4.0, y]], 0xFF00_0000 | w.color);
            game.native_fill(batch, [x - tw / 2.0 - 2.0, y - 17.0, tw + 4.0, 11.0], m::BACKGROUND);
            game.native_text(batch, &label, [x - tw / 2.0, y - 15.0], m::INK, true);
        }
    }
    let shown = m::LIST_SIZES[settings.list];
    let corner = Corner::ALL[settings.corner];
    if shown == 0 || !game.native_corner_shown(corner) {
        return;
    }
    let rows: Vec<_> = rows.iter().take(shown).collect();
    let labels: Vec<(String, String)> = rows.iter().map(|(w, d)| (w.name.clone(), m::distance_label(*d))).collect();
    let name_w = labels.iter().map(|l| game.native_text_width(&l.0)).fold(0.0, f32::max);
    let dist_w = labels.iter().map(|l| game.native_text_width(&l.1)).fold(0.0, f32::max);
    let (w, h) = (12.0 + name_w + 6.0 + dist_w + 3.0, 10.0 * rows.len() as f32 + 2.0);
    let (x, y) = corners.place(corner, w, h);
    game.native_fill(batch, [x, y, w, h], m::BACKGROUND);
    for (i, ((wp, _), (name, dist))) in rows.iter().zip(&labels).enumerate() {
        let row = y + 2.0 + 10.0 * i as f32;
        let turn = m::bearing(at, p.yaw, wp.target());
        game.native_quad(batch, m::arrow([x + 6.0, row + 4.0], turn, 8.0), 0xFF00_0000 | wp.color);
        game.native_text(batch, name, [x + 12.0, row], m::INK, true);
        let dw = game.native_text_width(dist);
        game.native_text(batch, dist, [x + w - 2.0 - dw, row], m::GREY, true);
    }
}

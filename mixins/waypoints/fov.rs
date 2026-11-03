// The world camera and its field of view, kept for the markers this frame.
fn waypoints_camera(game: &crate::game::Game<'_>, camera: [f64; 3], degrees: &mut f32) {
    use crate::game::mod_waypoints as m;
    if !game.native_on(m::ID) || !degrees.is_finite() {
        return;
    }
    let fov = *degrees;
    m::with(|k| game.native_setting(m::ID, k), |s| s.camera = Some((camera, fov)));
}

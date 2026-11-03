// Waypoints: named spots you save with a key, keep in a menu of their own
// (rename, recolour, delete) and find again: a marker over each one in the
// world with its name and distance, and a list in a corner with an arrow that
// points at each one. A death adds a waypoint where you died; `/wp` does it
// all from chat too.
// This file is both the mod's Cargo library (its tests) and the module the
// bundle adds to the game.

/// The mod's id (its settings folder), name and version.
pub const ID: &str = "waypoints";
pub const NAME: &str = "Waypoints";
pub const VERSION: &str = "1.1.0";
pub const DESCRIPTION: &str = "Save spots with N, manage them in a menu (U) and find them again: markers in the world and a list with arrows. A death adds one too.";
/// Its keys: name, label and default key.
pub const KEYS: [[&str; 3]; 3] = [
    ["add", "New waypoint here", "key.keyboard.n"],
    ["toggle", "Show or hide waypoints", "key.keyboard.unknown"],
    ["menu", "Waypoint menu", "key.keyboard.u"],
];
/// The panel: translucent black, as the official mods' boxes.
pub const BACKGROUND: u32 = 0x7800_0000;
pub const INK: u32 = 0xFF_FFFF;
pub const GREY: u32 = 0xA0_A0A0;
/// Waypoint colours, given in turn to new waypoints.
pub const PALETTE: [(u32, &str); 8] = [
    (0xFF_5555, "red"),
    (0x55_FF55, "green"),
    (0x55_55FF, "blue"),
    (0xFF_FF55, "yellow"),
    (0xFF_AA00, "gold"),
    (0x55_FFFF, "aqua"),
    (0xFF_55FF, "pink"),
    (0xFF_FFFF, "white"),
];
/// The corners, as stored and as the settings page labels them.
pub const CORNERS: [(&str, &str); 4] = [
    ("top_left", "Top left"),
    ("top_right", "Top right"),
    ("bottom_left", "Bottom left"),
    ("bottom_right", "Bottom right"),
];
/// How many waypoints the list shows, nearest first.
pub const LIST_SIZES: [usize; 4] = [3, 5, 8, 0];
/// The name a death's waypoint gets.
pub const DEATH_NAME: &str = "Death";

/// One saved spot.
#[derive(Clone, Debug, PartialEq)]
pub struct Waypoint {
    pub name: String,
    /// `minecraft:overworld`, `minecraft:the_nether`, `minecraft:the_end`.
    pub dimension: String,
    pub pos: [i32; 3],
    pub color: u32,
}

impl Waypoint {
    /// The middle of the block, a little above it: where the marker points.
    pub fn target(&self) -> [f64; 3] {
        let [x, y, z] = self.pos;
        [f64::from(x) + 0.5, f64::from(y) + 1.0, f64::from(z) + 0.5]
    }
}

/// Every waypoint, in the order they were made.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Store {
    pub list: Vec<Waypoint>,
}

impl Store {
    /// From the `list` file: one waypoint per line, tab separated
    /// (name, dimension, x, y, z, colour as hex). Bad lines are skipped.
    pub fn read(text: &str) -> Self {
        let list = text
            .lines()
            .filter_map(|line| {
                let mut f = line.split('\t');
                let name = f.next()?.trim();
                let dimension = f.next()?.trim();
                let mut n = || f.next()?.trim().parse::<i32>().ok();
                let pos = [n()?, n()?, n()?];
                let color = u32::from_str_radix(f.next().unwrap_or("ffffff").trim(), 16).ok()?;
                (!name.is_empty() && !dimension.is_empty()).then(|| Waypoint {
                    name: name.to_owned(),
                    dimension: dimension.to_owned(),
                    pos,
                    color: color & 0xFF_FFFF,
                })
            })
            .collect();
        Self { list }
    }

    pub fn write(&self) -> String {
        self.list
            .iter()
            .map(|w| {
                let [x, y, z] = w.pos;
                format!(
                    "{}\t{}\t{x}\t{y}\t{z}\t{:06x}\n",
                    w.name, w.dimension, w.color
                )
            })
            .collect()
    }

    /// The next colour: the palette's first one used least.
    fn next_color(&self) -> u32 {
        let used = |c: u32| self.list.iter().filter(|w| w.color == c).count();
        PALETTE
            .iter()
            .map(|p| p.0)
            .min_by_key(|&c| used(c))
            .unwrap_or(INK)
    }

    /// `Waypoint 3`: the first number not taken.
    fn free_name(&self) -> String {
        (1..)
            .map(|i| format!("Waypoint {i}"))
            .find(|name| self.find(name).is_none())
            .unwrap_or_default()
    }

    /// The waypoint named `name` (any case).
    pub fn find(&self, name: &str) -> Option<usize> {
        self.list
            .iter()
            .position(|w| w.name.eq_ignore_ascii_case(name))
    }

    /// Adds a waypoint, or moves the one of that name here. Returns its name.
    pub fn add(
        &mut self,
        name: &str,
        color: Option<u32>,
        dimension: &str,
        pos: [i32; 3],
    ) -> String {
        let name = clean_name(name);
        let name = if name.is_empty() {
            self.free_name()
        } else {
            name
        };
        if let Some(i) = self.find(&name) {
            let w = &mut self.list[i];
            w.dimension = dimension.to_owned();
            w.pos = pos;
            if let Some(c) = color {
                w.color = c;
            }
            return w.name.clone();
        }
        let color = color.unwrap_or_else(|| self.next_color());
        self.list.push(Waypoint {
            name: name.clone(),
            dimension: dimension.to_owned(),
            pos,
            color,
        });
        name
    }

    /// Removes the waypoint named `selector`, or the list's number `selector`
    /// (1 is the first `/wp list` shows). Returns its name.
    pub fn remove(&mut self, selector: &str) -> Option<String> {
        let i = self.find(selector).or_else(|| {
            let n: usize = selector.trim().parse().ok()?;
            (1..=self.list.len()).contains(&n).then(|| n - 1)
        })?;
        Some(self.list.remove(i).name)
    }

    /// Renames waypoint `i`: an empty name keeps the old one, a name another
    /// waypoint has gets a number after it.
    pub fn rename(&mut self, i: usize, name: &str) {
        let name = clean_name(name);
        if i >= self.list.len() || name.is_empty() {
            return;
        }
        let taken = |n: &str| {
            self.list
                .iter()
                .enumerate()
                .any(|(j, w)| j != i && w.name.eq_ignore_ascii_case(n))
        };
        let mut new = name.clone();
        let mut k = 2;
        while taken(&new) {
            new = format!("{name} {k}");
            k += 1;
        }
        self.list[i].name = new;
    }

    /// Waypoint `i` takes the palette's next colour.
    pub fn cycle_color(&mut self, i: usize) {
        if let Some(w) = self.list.get_mut(i) {
            let at = PALETTE
                .iter()
                .position(|p| p.0 == w.color)
                .map_or(0, |j| j + 1);
            w.color = PALETTE[at % PALETTE.len()].0;
        }
    }

    /// The waypoints in `dimension`, nearest first, with their distance.
    pub fn nearest(&self, dimension: &str, at: [f64; 3]) -> Vec<(&Waypoint, f64)> {
        let mut found: Vec<_> = self
            .list
            .iter()
            .filter(|w| w.dimension == dimension)
            .map(|w| (w, distance(at, w.target())))
            .collect();
        found.sort_by(|a, b| a.1.total_cmp(&b.1));
        found
    }
}

/// A name as typed, without tabs or line breaks, at most 32 characters.
pub fn clean_name(name: &str) -> String {
    let name: String = name
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .take(32)
        .collect();
    name.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A colour by name (`red`, `gold`…) or as hex (`#ff8800`, `ff8800`).
pub fn parse_color(word: &str) -> Option<u32> {
    let word = word.trim().to_ascii_lowercase();
    if let Some(p) = PALETTE.iter().find(|p| p.1 == word) {
        return Some(p.0);
    }
    let hex = word.strip_prefix('#')?;
    (hex.len() == 6)
        .then(|| u32::from_str_radix(hex, 16).ok())
        .flatten()
}

pub fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// `12 m`, `1.4 km`.
pub fn distance_label(blocks: f64) -> String {
    if blocks < 1000.0 {
        format!("{blocks:.0} m")
    } else {
        format!("{:.1} km", blocks / 1000.0)
    }
}

/// Where `target` is from someone at `at` facing `yaw` (degrees, 0 south,
/// 90 west): degrees to turn, -180 to 180, positive to the right.
pub fn bearing(at: [f64; 3], yaw: f32, target: [f64; 3]) -> f32 {
    let (dx, dz) = (target[0] - at[0], target[2] - at[2]);
    // The yaw that would face the target, as Minecraft counts yaw.
    let facing = (-dx).atan2(dz).to_degrees() as f32;
    (facing - yaw + 540.0).rem_euclid(360.0) - 180.0
}

/// Where `target` shows on a `width` x `height` screen for a camera at
/// `camera` looking along `yaw` and `pitch` with a vertical field of view of
/// `fov` degrees, or None behind the camera.
pub fn project(
    camera: [f64; 3],
    yaw: f32,
    pitch: f32,
    fov: f32,
    [width, height]: [f32; 2],
    target: [f64; 3],
) -> Option<[f32; 2]> {
    let (yaw, pitch) = (f64::from(yaw).to_radians(), f64::from(pitch).to_radians());
    let forward = [
        -yaw.sin() * pitch.cos(),
        -pitch.sin(),
        yaw.cos() * pitch.cos(),
    ];
    let right = [-yaw.cos(), 0.0, -yaw.sin()];
    // right x forward: up for the camera.
    let up = [
        right[1] * forward[2] - right[2] * forward[1],
        right[2] * forward[0] - right[0] * forward[2],
        right[0] * forward[1] - right[1] * forward[0],
    ];
    let d = [
        target[0] - camera[0],
        target[1] - camera[1],
        target[2] - camera[2],
    ];
    let dot = |v: [f64; 3]| d[0] * v[0] + d[1] * v[1] + d[2] * v[2];
    let (z, x, y) = (dot(forward), dot(right), dot(up));
    if z < 0.05 {
        return None;
    }
    let scale = f64::from(height) / 2.0 / (f64::from(fov).to_radians() / 2.0).tan();
    Some([
        (f64::from(width) / 2.0 + x / z * scale) as f32,
        (f64::from(height) / 2.0 - y / z * scale) as f32,
    ])
}

/// The corners of an arrow `size` pixels long round `center`, turned `degrees`
/// clockwise from pointing up: tip, right wing, notch, left wing.
pub fn arrow(center: [f32; 2], degrees: f32, size: f32) -> [[f32; 2]; 4] {
    let (sin, cos) = degrees.to_radians().sin_cos();
    let at = |x: f32, y: f32| [center[0] + x * cos - y * sin, center[1] + x * sin + y * cos];
    let h = size / 2.0;
    [
        at(0.0, -h),
        at(h * 0.8, h),
        at(0.0, h * 0.4),
        at(-h * 0.8, h),
    ]
}

/// The settings page's buttons and their stored values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub shown: bool,
    pub markers: bool,
    /// Indices in `CORNERS` and `LIST_SIZES`.
    pub corner: usize,
    pub list: usize,
    pub deaths: bool,
}

impl Settings {
    /// From the stored values; shown, with markers, a list of 5 in the top
    /// left, deaths kept, for the ones missing.
    pub fn read(get: impl Fn(&str) -> Option<String>) -> Self {
        let flag = |key: &str| get(key).is_none_or(|v| v.trim() != "false");
        let corner = get("corner")
            .and_then(|v| CORNERS.iter().position(|c| c.0 == v.trim()))
            .unwrap_or(0);
        let list = get("list")
            .and_then(|v| LIST_SIZES.iter().position(|n| n.to_string() == v.trim()))
            .unwrap_or(1);
        Self {
            shown: flag("shown"),
            markers: flag("markers"),
            corner,
            list,
            deaths: flag("deaths"),
        }
    }
    pub fn labels(&self) -> Vec<String> {
        let on = |b: bool| if b { "ON" } else { "OFF" };
        let list = match LIST_SIZES[self.list] {
            0 => "Off".to_owned(),
            n => format!("{n} nearest"),
        };
        vec![
            format!("Shown: {}", on(self.shown)),
            format!("Markers in the world: {}", on(self.markers)),
            format!("List: {list}"),
            format!("List corner: {}", CORNERS[self.corner].1),
            format!("Waypoint on death: {}", on(self.deaths)),
        ]
    }
    pub fn click(&mut self, i: usize) {
        match i {
            0 => self.shown = !self.shown,
            1 => self.markers = !self.markers,
            2 => self.list = (self.list + 1) % LIST_SIZES.len(),
            3 => self.corner = (self.corner + 1) % CORNERS.len(),
            4 => self.deaths = !self.deaths,
            _ => {}
        }
    }
    pub fn stored(&self) -> [(&'static str, String); 5] {
        let flag = |b: bool| if b { "true" } else { "false" }.to_owned();
        [
            ("shown", flag(self.shown)),
            ("markers", flag(self.markers)),
            ("list", LIST_SIZES[self.list].to_string()),
            ("corner", CORNERS[self.corner].0.to_owned()),
            ("deaths", flag(self.deaths)),
        ]
    }
}

/// What `/wp ...` asks for.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Add { name: String, color: Option<u32> },
    Remove(String),
    List,
    Clear,
    Toggle,
    Help,
}

/// `/wp` and its words (without the slash), or None for another command.
pub fn parse(command: &str) -> Option<Command> {
    let mut words = command.split_whitespace();
    if words.next()? != "wp" {
        return None;
    }
    let verb = words.next().unwrap_or("help").to_ascii_lowercase();
    let mut rest: Vec<&str> = words.collect();
    Some(match verb.as_str() {
        "add" | "set" | "new" => {
            // A last word that is a colour colours it: `/wp add Base gold`.
            let color = rest.last().and_then(|w| parse_color(w));
            if color.is_some() {
                rest.pop();
            }
            Command::Add {
                name: rest.join(" "),
                color,
            }
        }
        "remove" | "del" | "delete" | "rm" if !rest.is_empty() => Command::Remove(rest.join(" ")),
        "list" | "ls" => Command::List,
        "clear" if rest.first() == Some(&"confirm") => Command::Clear,
        "toggle" | "hide" | "show" => Command::Toggle,
        _ => Command::Help,
    })
}

pub const HELP: [&str; 6] = [
    "/wp add [name] [colour]: a waypoint where you stand",
    "/wp remove <name or number>",
    "/wp list: this dimension's, nearest first",
    "/wp toggle: hide or show them",
    "/wp clear confirm: remove every waypoint",
    "Colours: red green blue yellow gold aqua pink white, or #rrggbb",
];

/// The block a player at `p` stands in.
pub fn block_of(p: [f64; 3]) -> [i32; 3] {
    [
        p[0].floor() as i32,
        p[1].floor() as i32,
        p[2].floor() as i32,
    ]
}

/// What can be clicked on the menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    AddHere,
    Shown,
    Markers,
    /// Row `i` of the store: its colour swatch, its name (rename), its
    /// Rename button and its delete button.
    Color(usize),
    Name(usize),
    Rename(usize),
    Delete(usize),
}

/// The menu's places, in GUI pixels (`[x, y, w, h]`), for a `width` x
/// `height` window, `rows` waypoints and the list scrolled by `scroll`.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub panel: [f32; 4],
    pub title_y: f32,
    pub buttons: Vec<(Button, [f32; 4])>,
    /// The rows shown: the store index and the row's rectangle.
    pub rows: Vec<(usize, [f32; 4])>,
    pub footer_y: f32,
    /// How many rows fit.
    pub fits: usize,
}

pub const ROW: f32 = 16.0;
pub const BUTTON_H: f32 = 14.0;

impl Layout {
    pub fn new(width: f32, height: f32, rows: usize, scroll: usize) -> Self {
        let w = (width - 16.0).clamp(160.0, 340.0);
        let x = ((width - w) / 2.0).max(0.0);
        let top = 12.0;
        let title_y = top;
        let bar_y = top + 14.0;
        let third = (w - 8.0) / 3.0;
        let mut buttons = vec![
            (Button::AddHere, [x, bar_y, third, BUTTON_H]),
            (Button::Shown, [x + third + 4.0, bar_y, third, BUTTON_H]),
            (
                Button::Markers,
                [x + 2.0 * (third + 4.0), bar_y, third, BUTTON_H],
            ),
        ];
        let list_y = bar_y + BUTTON_H + 8.0;
        let footer_y = height - 14.0;
        let fits = (((footer_y - 4.0 - list_y) / ROW).floor().max(1.0)) as usize;
        let scroll = scroll.min(rows.saturating_sub(fits));
        let mut shown = Vec::new();
        for (slot, i) in (scroll..rows).take(fits).enumerate() {
            let y = list_y + ROW * slot as f32;
            shown.push((i, [x, y, w, ROW]));
            let del = [x + w - BUTTON_H, y + 1.0, BUTTON_H, BUTTON_H];
            let rename = [del[0] - 4.0 - 44.0, y + 1.0, 44.0, BUTTON_H];
            buttons.push((Button::Color(i), [x + 2.0, y + 3.0, 10.0, 10.0]));
            buttons.push((Button::Name(i), [x + 16.0, y, w * 0.4, ROW]));
            buttons.push((Button::Rename(i), rename));
            buttons.push((Button::Delete(i), del));
        }
        Self {
            panel: [x - 6.0, top - 6.0, w + 12.0, footer_y + 12.0 - top],
            title_y,
            buttons,
            rows: shown,
            footer_y,
            fits,
        }
    }
    /// The button under `at`, if any.
    pub fn hit(&self, at: [f32; 2]) -> Option<Button> {
        self.buttons
            .iter()
            .find(|(_, r)| inside(*r, at))
            .map(|b| b.0)
    }
}

/// The character key `key.keyboard.n` types, for a letter or digit key.
pub fn key_char(key: &str) -> Option<char> {
    let rest = key.strip_prefix("key.keyboard.")?;
    let mut chars = rest.chars();
    let c = chars.next()?;
    (chars.next().is_none() && c.is_ascii_alphanumeric()).then_some(c)
}

/// A top button's label.
pub fn button_label(button: Button, settings: Settings) -> String {
    let on = |b: bool| if b { "ON" } else { "OFF" };
    match button {
        Button::AddHere => "+ Add here".to_owned(),
        Button::Shown => format!("Shown: {}", on(settings.shown)),
        Button::Markers => format!("Markers: {}", on(settings.markers)),
        Button::Rename(_) => "Rename".to_owned(),
        Button::Delete(_) => "x".to_owned(),
        Button::Color(_) | Button::Name(_) => String::new(),
    }
}

/// `text` cut to `max` pixels by `width`, with `..` when cut.
pub fn fit(text: &str, max: f32, width: impl Fn(&str) -> f32) -> String {
    if width(text) <= max {
        return text.to_owned();
    }
    let mut cut: String = text.to_owned();
    while !cut.is_empty() && width(&format!("{cut}..")) > max {
        cut.pop();
    }
    format!("{cut}..")
}

pub fn inside([x, y, w, h]: [f32; 4], [px, py]: [f32; 2]) -> bool {
    px >= x && py >= y && px < x + w && py < y + h
}

/// The menu's own state while it is open.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ui {
    /// The row being renamed and the text typed so far; `fresh` while the
    /// first key will replace the whole name (a new waypoint's).
    pub editing: Option<(usize, String, bool)>,
    /// A delete clicked once: the second click deletes.
    pub confirm_delete: Option<usize>,
    pub scroll: usize,
    /// Ask the next tick to close the menu (the click hooks can't).
    pub close: bool,
    /// The character of the key that opened the menu, which the game hands
    /// the new screen right after: dropped once.
    pub swallow: Option<char>,
}

/// What a key does on the menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuKey {
    /// Swallowed: nothing else hears it.
    Taken,
    /// Left to the game (Escape closes the menu).
    Passed,
}

/// The mod's state in the window's game: the waypoints and settings, read
/// once from their files, and what the hooks last saw.
#[derive(Clone, Debug, Default)]
pub struct State {
    pub store: Store,
    pub settings: Option<Settings>,
    /// The world camera and field of view the last frame used.
    pub camera: Option<([f64; 3], f32)>,
    /// The health last frame, to see a death.
    pub health: f32,
    /// The menu, while it is open.
    pub ui: Ui,
    loaded: bool,
}

impl State {
    /// Reads the files on first use.
    pub fn load(&mut self, get: impl Fn(&str) -> Option<String>) {
        if !self.loaded {
            self.loaded = true;
            self.store = Store::read(&get("list").unwrap_or_default());
            self.settings = Some(Settings::read(get));
        }
    }
    pub fn settings(&self) -> Settings {
        self.settings.unwrap_or(Settings::read(|_| None))
    }
    /// Adds (or moves) a waypoint at block `pos`: the line chat shows.
    pub fn add_here(
        &mut self,
        name: &str,
        color: Option<u32>,
        dimension: &str,
        pos: [i32; 3],
    ) -> String {
        let name = self.store.add(name, color, dimension, pos);
        let [x, y, z] = pos;
        format!("Waypoint {name} at {x} {y} {z}")
    }
    /// Hides or shows everything: the line chat shows.
    pub fn toggle(&mut self) -> String {
        let mut settings = self.settings();
        settings.shown = !settings.shown;
        self.settings = Some(settings);
        if settings.shown {
            "Waypoints shown"
        } else {
            "Waypoints hidden"
        }
        .to_owned()
    }
    /// `/wp list`'s lines: every waypoint by its number, with its distance
    /// from `at` in `dimension` or the dimension it is in.
    pub fn list_lines(&self, dimension: &str, at: [f64; 3]) -> Vec<String> {
        if self.store.list.is_empty() {
            return vec!["No waypoints yet: /wp add <name>, or press the Add key".to_owned()];
        }
        self.store
            .list
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let [x, y, z] = w.pos;
                let where_ = if w.dimension == dimension {
                    distance_label(distance(at, w.target()))
                } else {
                    w.dimension
                        .trim_start_matches("minecraft:")
                        .replace('_', " ")
                };
                format!("{}. {}: {x} {y} {z} ({where_})", i + 1, w.name)
            })
            .collect()
    }
    /// The files to keep: the list and every setting.
    pub fn files(&self) -> Vec<(&'static str, String)> {
        let mut files = vec![("list", self.store.write())];
        files.extend(self.settings().stored());
        files
    }
    /// The menu opens fresh; `rename` starts renaming that row at once.
    pub fn open_menu(&mut self, rename: Option<usize>, key: &str) {
        self.ui = Ui {
            swallow: key_char(key),
            ..Ui::default()
        };
        if let Some(i) = rename.filter(|&i| i < self.store.list.len()) {
            self.ui.editing = Some((i, self.store.list[i].name.clone(), true));
            // Scrolled so that the row is in view.
            self.ui.scroll = i;
        }
    }
    /// Ends a rename, keeping what was typed. True when something changed.
    pub fn commit(&mut self) -> bool {
        match self.ui.editing.take() {
            Some((i, text, _)) => {
                let before = self.store.list.get(i).map(|w| w.name.clone());
                self.store.rename(i, &text);
                before != self.store.list.get(i).map(|w| w.name.clone())
            }
            None => false,
        }
    }
    /// A click on `button`; `here` is the player's dimension and block, for
    /// Add here. True when the files should be kept.
    pub fn click(&mut self, button: Option<Button>, here: Option<(&str, [i32; 3])>) -> bool {
        // A click anywhere ends a rename (the name field itself keeps it).
        let mut changed = match (button, &self.ui.editing) {
            (Some(Button::Name(i)), Some((j, _, _))) if i == *j => return false,
            _ => self.commit(),
        };
        if !matches!(button, Some(Button::Delete(_))) {
            self.ui.confirm_delete = None;
        }
        let mut settings = self.settings();
        match button {
            Some(Button::AddHere) => {
                if let Some((dimension, pos)) = here {
                    self.store.add("", None, dimension, pos);
                    let i = self.store.list.len() - 1;
                    self.ui.editing = Some((i, self.store.list[i].name.clone(), true));
                    self.ui.scroll = i;
                    changed = true;
                }
            }
            Some(Button::Shown) => {
                settings.shown = !settings.shown;
                changed = true;
            }
            Some(Button::Markers) => {
                settings.markers = !settings.markers;
                changed = true;
            }
            Some(Button::Color(i)) => {
                self.store.cycle_color(i);
                changed = true;
            }
            Some(Button::Name(i) | Button::Rename(i)) => {
                if let Some(w) = self.store.list.get(i) {
                    self.ui.editing = Some((i, w.name.clone(), false));
                }
            }
            Some(Button::Delete(i)) => {
                if self.ui.confirm_delete == Some(i) {
                    if i < self.store.list.len() {
                        self.store.list.remove(i);
                        changed = true;
                    }
                    self.ui.confirm_delete = None;
                } else {
                    self.ui.confirm_delete = Some(i);
                }
            }
            None => {}
        }
        self.settings = Some(settings);
        changed
    }
    /// A key on the menu: Enter keeps a rename, Escape drops it, Backspace
    /// takes a character back; the menu key closes the menu. Returns what
    /// happens to the key and whether the files should be kept.
    pub fn menu_key(&mut self, key: &str, menu_key: &str) -> (MenuKey, bool) {
        self.ui.swallow = None;
        if let Some((_, text, fresh)) = &mut self.ui.editing {
            return match key {
                "key.keyboard.enter" | "key.keyboard.keypad.enter" => {
                    (MenuKey::Taken, self.commit())
                }
                "key.keyboard.escape" => {
                    self.ui.editing = None;
                    (MenuKey::Taken, false)
                }
                "key.keyboard.backspace" => {
                    if *fresh {
                        text.clear();
                        *fresh = false;
                    } else {
                        text.pop();
                    }
                    (MenuKey::Taken, false)
                }
                // Every other key types (its character comes on its own).
                _ => (MenuKey::Taken, false),
            };
        }
        if key == menu_key {
            self.ui.close = true;
            return (MenuKey::Taken, false);
        }
        match key {
            "key.keyboard.escape" => (MenuKey::Passed, false),
            _ => (MenuKey::Taken, false),
        }
    }
    /// A character typed on the menu: into the name being renamed.
    pub fn typed(&mut self, c: char) {
        if self
            .ui
            .swallow
            .take()
            .is_some_and(|k| k.eq_ignore_ascii_case(&c))
        {
            return;
        }
        if let Some((_, text, fresh)) = &mut self.ui.editing {
            if *fresh {
                text.clear();
                *fresh = false;
            }
            if !c.is_control() && text.chars().count() < 32 {
                text.push(c);
            }
        }
    }
    /// The wheel over the menu: up shows earlier rows.
    pub fn scroll(&mut self, up: bool, fits: usize) {
        let most = self.store.list.len().saturating_sub(fits);
        self.ui.scroll = if up {
            self.ui.scroll.saturating_sub(1)
        } else {
            (self.ui.scroll + 1).min(most)
        };
    }
    /// True once when the health drops to 0 from above it.
    pub fn died(&mut self, health: f32) -> bool {
        let died = self.health > 0.0 && health <= 0.0;
        self.health = health;
        died
    }
}

thread_local! {
    static STATE: std::cell::RefCell<State> = std::cell::RefCell::new(State::default());
}

/// Runs `f` on the state, its files read through `get` the first time.
pub fn with<R>(get: impl Fn(&str) -> Option<String>, f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.load(get);
        f(&mut s)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_round_trips_and_skips_bad_lines() {
        let mut s = Store::default();
        assert_eq!(
            s.add("  My   Base ", None, "minecraft:overworld", [10, 64, -5]),
            "My Base"
        );
        assert_eq!(
            s.add("", Some(0x12_3456), "minecraft:the_nether", [1, 2, 3]),
            "Waypoint 1"
        );
        assert_eq!(
            s.add("", None, "minecraft:overworld", [0, 70, 0]),
            "Waypoint 2"
        );
        let text = s.write();
        assert_eq!(Store::read(&text), s);
        let read = Store::read(&format!("broken line\n\t\t\n{text}x\ty\t1\t2\n"));
        assert_eq!(read, s);
        assert_eq!(s.list[1].color, 0x12_3456);
        assert_ne!(s.list[0].color, s.list[2].color);
    }

    #[test]
    fn add_moves_a_waypoint_of_the_same_name() {
        let mut s = Store::default();
        s.add("Base", None, "minecraft:overworld", [0, 64, 0]);
        let color = s.list[0].color;
        s.add("base", None, "minecraft:overworld", [5, 65, 5]);
        assert_eq!(s.list.len(), 1);
        assert_eq!((s.list[0].pos, s.list[0].color), ([5, 65, 5], color));
    }

    #[test]
    fn remove_by_name_or_number() {
        let mut s = Store::default();
        for name in ["A", "B", "C"] {
            s.add(name, None, "minecraft:overworld", [0, 0, 0]);
        }
        assert_eq!(s.remove("b").as_deref(), Some("B"));
        assert_eq!(s.remove("2").as_deref(), Some("C"));
        assert_eq!(s.remove("5"), None);
        assert_eq!(s.remove("nope"), None);
        assert_eq!(s.list.len(), 1);
    }

    #[test]
    fn nearest_first_in_one_dimension() {
        let mut s = Store::default();
        s.add("Far", None, "minecraft:overworld", [100, 63, 0]);
        s.add("Near", None, "minecraft:overworld", [10, 63, 0]);
        s.add("Nether", None, "minecraft:the_nether", [1, 63, 0]);
        let found = s.nearest("minecraft:overworld", [0.0, 64.0, 0.0]);
        let names: Vec<_> = found.iter().map(|f| f.0.name.as_str()).collect();
        assert_eq!(names, ["Near", "Far"]);
    }

    #[test]
    fn bearing_follows_minecraft_yaw() {
        let at = [0.0, 64.0, 0.0];
        // Facing south (+Z): a target to the south is ahead, west is right.
        assert!(bearing(at, 0.0, [0.0, 64.0, 10.0]).abs() < 0.01);
        assert!((bearing(at, 0.0, [-10.0, 64.0, 0.0]) - 90.0).abs() < 0.01);
        assert!((bearing(at, 0.0, [10.0, 64.0, 0.0]) + 90.0).abs() < 0.01);
        assert!((bearing(at, 0.0, [0.0, 64.0, -10.0]).abs() - 180.0).abs() < 0.01);
        // Facing west (yaw 90): west is ahead, north (-Z) is right.
        assert!(bearing(at, 90.0, [-10.0, 64.0, 0.0]).abs() < 0.01);
        assert!((bearing(at, 90.0, [0.0, 64.0, -10.0]) - 90.0).abs() < 0.01);
        assert!(bearing(at, 450.0, [-10.0, 64.0, 0.0]).abs() < 0.01);
    }

    #[test]
    fn project_puts_ahead_in_the_middle() {
        let size = [320.0, 180.0];
        let cam = [0.0, 64.0, 0.0];
        let mid = project(cam, 0.0, 0.0, 70.0, size, [0.0, 64.0, 10.0]).unwrap();
        assert!((mid[0] - 160.0).abs() < 0.01 && (mid[1] - 90.0).abs() < 0.01);
        // West is to the right when facing south; higher is up the screen.
        let right = project(cam, 0.0, 0.0, 70.0, size, [-2.0, 66.0, 10.0]).unwrap();
        assert!(right[0] > 160.0 && right[1] < 90.0);
        assert_eq!(project(cam, 0.0, 0.0, 70.0, size, [0.0, 64.0, -10.0]), None);
        // Looking down (pitch 90 is straight down): below is in the middle.
        let down = project(cam, 0.0, 89.0, 70.0, size, [0.0, 54.0, 0.1]).unwrap();
        assert!((down[0] - 160.0).abs() < 1.0 && (down[1] - 90.0).abs() < 2.0);
        // The edge of the field of view is the edge of the screen.
        let half = (35.0_f64).to_radians().tan() * 10.0;
        let top = project(cam, 0.0, 0.0, 70.0, size, [0.0, 64.0 + half, 10.0]).unwrap();
        assert!(top[1].abs() < 0.01);
    }

    #[test]
    fn arrow_turns_clockwise() {
        let up = arrow([10.0, 10.0], 0.0, 8.0);
        assert_eq!(up[0], [10.0, 6.0]);
        let right = arrow([10.0, 10.0], 90.0, 8.0);
        assert!((right[0][0] - 14.0).abs() < 1e-4 && (right[0][1] - 10.0).abs() < 1e-4);
    }

    #[test]
    fn commands_parse() {
        assert_eq!(parse("say hi"), None);
        assert_eq!(parse("wp"), Some(Command::Help));
        assert_eq!(
            parse("wp add My Base gold"),
            Some(Command::Add {
                name: "My Base".into(),
                color: Some(0xFF_AA00)
            })
        );
        assert_eq!(
            parse("wp add #00ff00"),
            Some(Command::Add {
                name: String::new(),
                color: Some(0x00_FF00)
            })
        );
        assert_eq!(
            parse("wp add"),
            Some(Command::Add {
                name: String::new(),
                color: None
            })
        );
        assert_eq!(parse("wp rm 2"), Some(Command::Remove("2".into())));
        assert_eq!(parse("wp remove"), Some(Command::Help));
        assert_eq!(parse("wp clear"), Some(Command::Help));
        assert_eq!(parse("wp clear confirm"), Some(Command::Clear));
        assert_eq!(parse("wp LIST"), Some(Command::List));
        assert_eq!(parse("wp hide"), Some(Command::Toggle));
    }

    #[test]
    fn settings_round_trip() {
        let s = Settings::read(|_| None);
        assert_eq!(
            (s.shown, s.markers, s.corner, s.list, s.deaths),
            (true, true, 0, 1, true)
        );
        assert_eq!(s.labels()[2], "List: 5 nearest");
        let mut s2 = s;
        s2.click(2);
        s2.click(2);
        s2.click(3);
        s2.click(4);
        assert_eq!(s2.labels()[2], "List: Off");
        let stored = s2.stored();
        let back = Settings::read(|k| stored.iter().find(|f| f.0 == k).map(|f| f.1.clone()));
        assert_eq!(back, s2);
    }

    #[test]
    fn a_death_is_seen_once() {
        let mut s = State::default();
        assert!(!s.died(20.0));
        assert!(!s.died(5.0));
        assert!(s.died(0.0));
        assert!(!s.died(0.0));
        assert!(!s.died(20.0));
        // Joining a world dead is not a death.
        let mut fresh = State::default();
        assert!(!fresh.died(0.0));
    }

    fn three() -> State {
        let mut s = State::default();
        for name in ["A", "B", "C"] {
            s.store.add(name, None, "minecraft:overworld", [0, 64, 0]);
        }
        s
    }

    #[test]
    fn menu_renames_by_typing() {
        let mut s = three();
        s.open_menu(Some(1), "key.keyboard.n");
        // The opening key's own character is dropped.
        s.typed('n');
        assert_eq!(s.ui.editing, Some((1, "B".into(), true)));
        // The first character replaces a fresh name.
        for c in "Base".chars() {
            s.typed(c);
        }
        s.menu_key("key.keyboard.backspace", "key.keyboard.u");
        s.typed('e');
        assert_eq!(
            s.menu_key("key.keyboard.enter", "key.keyboard.u"),
            (MenuKey::Taken, true)
        );
        assert_eq!(s.store.list[1].name, "Base");
        assert_eq!(s.ui.editing, None);
        // A taken name gets a number; Escape drops a rename.
        s.click(Some(Button::Rename(0)), None);
        s.typed('x');
        s.menu_key("key.keyboard.backspace", "key.keyboard.u");
        s.menu_key("key.keyboard.backspace", "key.keyboard.u");
        for c in "base".chars() {
            s.typed(c);
        }
        s.commit();
        assert_eq!(s.store.list[0].name, "base 2");
        s.click(Some(Button::Rename(2)), None);
        s.typed('z');
        assert_eq!(
            s.menu_key("key.keyboard.escape", "key.keyboard.u"),
            (MenuKey::Taken, false)
        );
        assert_eq!(s.store.list[2].name, "C");
    }

    #[test]
    fn menu_keys_close_and_pass_escape() {
        let mut s = three();
        s.open_menu(None, "key.keyboard.u");
        assert_eq!(
            s.menu_key("key.keyboard.n", "key.keyboard.u"),
            (MenuKey::Taken, false)
        );
        assert!(!s.ui.close);
        assert_eq!(
            s.menu_key("key.keyboard.escape", "key.keyboard.u").0,
            MenuKey::Passed
        );
        s.menu_key("key.keyboard.u", "key.keyboard.u");
        assert!(s.ui.close);
    }

    #[test]
    fn menu_clicks() {
        let mut s = three();
        s.open_menu(None, "key.keyboard.u");
        assert!(s.click(
            Some(Button::AddHere),
            Some(("minecraft:overworld", [1, 2, 3]))
        ));
        assert_eq!(s.store.list.len(), 4);
        assert_eq!(s.ui.editing, Some((3, "Waypoint 1".into(), true)));
        // Delete asks twice; another click in between forgets the first.
        assert!(!s.click(Some(Button::Delete(0)), None));
        assert_eq!(s.ui.confirm_delete, Some(0));
        s.click(Some(Button::Shown), None);
        assert!(!s.settings().shown);
        assert!(!s.click(Some(Button::Delete(0)), None));
        assert!(s.click(Some(Button::Delete(0)), None));
        assert_eq!(s.store.list.len(), 3);
        let before = s.store.list[0].color;
        s.click(Some(Button::Color(0)), None);
        assert_ne!(s.store.list[0].color, before);
        s.click(Some(Button::Markers), None);
        assert!(!s.settings().markers);
    }

    #[test]
    fn layout_hits_and_scrolls() {
        let l = Layout::new(400.0, 240.0, 20, 0);
        assert!(l.fits >= 5 && l.fits < 20);
        assert_eq!(l.rows[0].0, 0);
        let [x, y, w, h] = l.buttons[0].1;
        assert_eq!(l.hit([x + w / 2.0, y + h / 2.0]), Some(Button::AddHere));
        let row = l.rows[1].1;
        assert_eq!(
            l.hit([row[0] + row[2] - 5.0, row[1] + 5.0]),
            Some(Button::Delete(1))
        );
        assert_eq!(l.hit([0.0, 239.0]), None);
        // Scrolled past the end, it stops at the last full page.
        let end = Layout::new(400.0, 240.0, 20, 99);
        assert_eq!(end.rows.last().unwrap().0, 19);
        assert_eq!(end.rows.len(), end.fits);
        let mut s = three();
        s.scroll(false, 2);
        s.scroll(false, 2);
        assert_eq!(s.ui.scroll, 1);
        s.scroll(true, 2);
        s.scroll(true, 2);
        assert_eq!(s.ui.scroll, 0);
    }

    #[test]
    fn names_and_distances() {
        assert_eq!(clean_name("a\tb\nc"), "a b c");
        assert_eq!(clean_name(&"x".repeat(40)).len(), 32);
        assert_eq!(distance_label(12.4), "12 m");
        assert_eq!(distance_label(1460.0), "1.5 km");
        assert_eq!(block_of([-0.5, 64.9, 3.2]), [-1, 64, 3]);
        assert_eq!(parse_color("GOLD"), Some(0xFF_AA00));
        assert_eq!(parse_color("#12345"), None);
        assert_eq!(key_char("key.keyboard.n"), Some('n'));
        assert_eq!(key_char("key.keyboard.enter"), None);
        let w = |t: &str| t.len() as f32 * 6.0;
        assert_eq!(fit("Base", 60.0, w), "Base");
        assert_eq!(fit("A very long name", 60.0, w), "A very l..");
    }
}

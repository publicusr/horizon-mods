//! Horizon's Coordinates HUD, formerly the WebAssembly SDK's example mod: the block position and
//! the direction the player faces in a box in the top-left corner, `/where` to put the position in
//! chat and `/coords` to hide or show the box, drawn and written as 2.0.0 did, and giving way to
//! F3's debug screen as 2.0.0 did. Its setting keeps the file of earlier versions
//! (`horizon-mods/.horizon-mod-data/coords_hud/shown`).
use mod_api::{
    ClientHost, Command, CommandEvent, Corner, Flow, Hooks, RegisterClientCommandsEvent, Registrar,
    RegistrationContext, RenderGuiPostEvent, SettingSpec, Text,
};

/// The mod's id: its settings folder.
pub const ID: &str = "coords_hud";
/// The WebAssembly mods' panel: translucent black, white text with a shadow.
pub const BACKGROUND: u32 = 0x7800_0000;
/// Opaque white (2.0.0's `0xFFFFFF`, in `argb` as the SDK takes it).
pub const INK: u32 = 0xFFFF_FFFF;
/// A line's height and the text's inset, in GUI pixels.
pub const LINE: f32 = 10.0;
pub const PADDING: f32 = 2.0;
/// The corner the box shows in, as 2.0.0's.
pub const CORNER: Corner = Corner::TopLeft;
/// The client commands: `/where` and `/coords`.
pub const COMMANDS: [&str; 2] = ["where", "coords"];

/// The setting the player picks on Coordinates HUD's page in Options > Mods: the box shown or
/// hidden, kept in `shown` (`true`/`false`) as earlier versions kept it.
pub fn settings() -> [SettingSpec; 1] {
    [
        SettingSpec::toggle("shown", "coords_hud.setting.shown", true)
            .tooltip("coords_hud.setting.shown.tooltip"),
    ]
}

/// Direction.fromYRot: yaw 0 faces south, 90 west.
pub fn facing(yaw: f32) -> &'static str {
    let i = (yaw / 90.0 + 0.5).floor() as i32 & 3;
    ["South (+Z)", "West (-X)", "North (-Z)", "East (+X)"][i as usize]
}

/// The box's lines, as the WebAssembly mod wrote them.
pub fn lines(x: f64, y: f64, z: f64, yaw: f32) -> [String; 2] {
    [
        format!(
            "XYZ {} / {} / {}",
            one_decimal(x),
            one_decimal(y),
            one_decimal(z)
        ),
        format!("Facing {}", facing(yaw)),
    ]
}

/// A coordinate with one decimal; a small negative that rounds to zero is `0.0`, not `-0.0`.
fn one_decimal(v: f64) -> String {
    let text = format!("{v:.1}");
    if text == "-0.0" { "0.0".into() } else { text }
}

/// `/where`'s line: `12, 70, -4 in minecraft:the_nether`.
pub fn where_line(x: f64, y: f64, z: f64, dimension: &str) -> String {
    // `+ 0.0` turns a floored -0.0 into 0.0, which prints without a minus sign.
    format!(
        "{:.0}, {:.0}, {:.0} in {dimension}",
        x.floor() + 0.0,
        y.floor() + 0.0,
        z.floor() + 0.0
    )
}

/// The box around lines whose widest is `width` GUI pixels: `[w, h]`.
pub fn size(width: f32, lines: usize) -> [f32; 2] {
    [width + 2.0 * PADDING, LINE * lines as f32 + PADDING]
}

/// `/coords`'s line in chat.
pub fn toggled_line(shown: bool) -> &'static str {
    if shown {
        "Coordinates panel shown"
    } else {
        "Coordinates panel hidden"
    }
}

/// The command's name (`where` of `where now`), when it is one of the mod's.
pub fn own_command(command: &str) -> Option<&'static str> {
    let name = command.split(' ').next().unwrap_or("");
    COMMANDS.into_iter().find(|c| *c == name)
}

/// The local player's position and yaw.
fn player(host: &dyn ClientHost) -> Option<([f64; 3], f32)> {
    let entity = host.local_player()?.entity();
    let pos = entity.position(host)?;
    let [yaw, _] = entity.rotation(host)?;
    Some((pos, yaw))
}

#[derive(Default)]
struct CoordsHud;

#[mod_api::horizon_mod]
impl Hooks for CoordsHud {
    fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
        for setting in settings() {
            // Only a malformed spec is refused, and the tests check every spec.
            let _ = r.register_setting(setting);
        }
    }

    /// `/where` and `/coords` in the chat box's completions, as the game's own commands.
    fn register_client_commands(
        &self,
        _: &mut dyn ClientHost,
        event: &mut RegisterClientCommandsEvent<'_>,
    ) {
        event
            .commands
            .extend(COMMANDS.into_iter().map(Command::new));
    }

    /// `/where` puts the position in chat; `/coords` hides or shows the box, and keeps it.
    fn command(&self, host: &mut dyn ClientHost, event: &mut CommandEvent<'_>) -> Flow {
        let Some(name) = own_command(event.command) else {
            return Flow::Continue;
        };
        if !host.on(ID) {
            return Flow::Continue;
        }
        let line = if name == "coords" {
            let shown = !host.setting_bool("shown").unwrap_or(true);
            host.set_setting(ID, "shown", if shown { "true" } else { "false" });
            toggled_line(shown).to_owned()
        } else {
            match player(host) {
                Some(([x, y, z], _)) => where_line(x, y, z, host.dimension()),
                None => "Not in a world".to_owned(),
            }
        };
        let _ = host.chat_line(Text::literal(line));
        Flow::Cancel
    }

    /// The box in the top-left corner, below what the corner holds already; none while F3's debug
    /// screen takes the corner.
    fn render_gui_post(&self, host: &dyn ClientHost, event: &mut RenderGuiPostEvent<'_>) {
        if !host.on(ID) || !host.native_corner_shown(CORNER) {
            return;
        }
        if !host.setting_bool("shown").unwrap_or(true) {
            return;
        }
        let Some(([x, y, z], yaw)) = player(host) else {
            return;
        };
        let lines = lines(x, y, z, yaw);
        let draw = &mut *event.draw;
        let widest = lines.iter().map(|l| draw.text_width(l)).fold(0.0, f32::max);
        let [w, h] = size(widest, lines.len());
        let (bx, by) = event.corners.place(CORNER, w, h);
        draw.fill([bx, by, w, h], BACKGROUND);
        for (i, line) in lines.iter().enumerate() {
            draw.text(
                line,
                [bx + PADDING, by + PADDING + LINE * i as f32],
                INK,
                true,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_setting_is_one_the_game_takes() {
        for setting in settings() {
            assert_eq!(setting.check(), Ok(()), "{}", setting.key);
        }
    }

    #[test]
    fn reads_the_file_earlier_versions_wrote() {
        // 1.x and 2.x wrote `true`/`false`, and read anything but `false` as shown.
        let [shown] = settings();
        use mod_api::SettingValue::Toggle;
        assert_eq!(shown.read(None), Toggle(true), "shown, as 2.0.0");
        assert_eq!(shown.read(Some("true")), Toggle(true));
        assert_eq!(shown.read(Some("false\n")), Toggle(false));
        assert_eq!(shown.read(Some("maybe")), Toggle(true));
        assert_eq!(shown.store(Toggle(false)).as_deref(), Some("false"));
        assert_eq!(shown.store(Toggle(true)).as_deref(), Some("true"));
    }

    #[test]
    fn facing_follows_vanilla_yaw() {
        assert_eq!(facing(0.0), "South (+Z)");
        assert_eq!(facing(90.0), "West (-X)");
        assert_eq!(facing(-180.0), "North (-Z)");
        assert_eq!(facing(-90.0), "East (+X)");
        assert_eq!(facing(314.0), "East (+X)");
    }

    #[test]
    fn facing_turns_at_the_half_way_yaws() {
        // Direction.fromYRot: each side owns 90 degrees centred on its own yaw, so it
        // changes half way (45, 135, 225, 315), the half way yaw itself going to the next.
        let south = "South (+Z)";
        let (west, north, east) = ("West (-X)", "North (-Z)", "East (+X)");
        for (yaw, side) in [
            (44.9, south),
            (45.0, west),
            (134.9, west),
            (135.0, north),
            (224.9, north),
            (225.0, east),
            (314.9, east),
            (315.0, south),
            (-44.9, south),
            (-45.0, south),
            (-45.1, east),
            (-135.0, east),
            (-135.1, north),
        ] {
            assert_eq!(facing(yaw), side, "yaw {yaw}");
        }
    }

    #[test]
    fn lines_read_as_the_webassembly_mods() {
        assert_eq!(
            lines(12.5, 64.0, -3.25, 180.0),
            ["XYZ 12.5 / 64.0 / -3.2", "Facing North (-Z)"]
        );
        assert_eq!(
            where_line(12.5, 70.2, -3.25, "minecraft:the_nether"),
            "12, 70, -4 in minecraft:the_nether"
        );
    }

    #[test]
    fn a_coordinate_just_below_zero_is_never_written_as_negative_zero() {
        // `{:.1}` of -0.04 and `{:.0}` of -0.0 both print a minus sign.
        assert_eq!(lines(-0.04, 64.0, -0.04, 0.0)[0], "XYZ 0.0 / 64.0 / 0.0");
        assert_eq!(
            where_line(-0.0, 70.0, -0.0, "minecraft:overworld"),
            "0, 70, 0 in minecraft:overworld"
        );
        // Real negatives keep their sign.
        assert_eq!(lines(-0.06, 64.0, -3.25, 0.0)[0], "XYZ -0.1 / 64.0 / -3.2");
        assert_eq!(
            where_line(-0.3, 70.0, -4.0, "minecraft:overworld"),
            "-1, 70, -4 in minecraft:overworld"
        );
    }

    #[test]
    fn the_box_is_as_2_0_0_laid_it_out() {
        // 2.0.0: the widest line plus 4, 10 per line plus 2, text 2 in.
        assert_eq!(size(100.0, 2), [104.0, 22.0]);
        assert_eq!(CORNER, Corner::TopLeft);
    }

    #[test]
    fn only_its_own_commands_are_taken() {
        assert_eq!(own_command("where"), Some("where"));
        assert_eq!(own_command("coords"), Some("coords"));
        assert_eq!(own_command("where now"), Some("where"));
        assert_eq!(own_command("coordsx"), None);
        assert_eq!(own_command("tp 0 0 0"), None);
        assert_eq!(own_command(""), None);
        assert_eq!(toggled_line(true), "Coordinates panel shown");
        assert_eq!(toggled_line(false), "Coordinates panel hidden");
    }
}

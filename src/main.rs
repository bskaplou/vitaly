extern crate hidapi;

use argh::FromArgs;
use hidapi::HidApi;

mod keycodes;
mod keymap;
mod protocol;

mod altrepeats;
mod combos;
mod common;
mod devices;
mod keyoverrides;
mod keys;
mod layers;
mod layout;
mod load;
mod lock;
mod macros;
mod rgb;
mod save;
mod settings;
mod tapdances;

/// VIA/Vial HID USB cli tool
#[derive(FromArgs)]
struct VialClient {
    /// device product id
    #[argh(option, short = 'i')]
    id: Option<u16>,

    /// show vitaly version
    #[argh(switch, short = 'v')]
    version: bool,

    /// command to run
    #[argh(subcommand)]
    command: Option<CommandEnum>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum CommandEnum {
    Devices(CommandDevices),
    Lock(CommandLock),
    Settings(CommandSettings),
    Layers(CommandLayers),
    Keys(CommandKeys),
    Combos(CommandCombos),
    Macros(CommandMacros),
    TapDances(CommandTapDances),
    KeyOverrides(CommandKeyOverrides),
    AltRepeats(CommandAltRepeats),
    Load(CommandLoad),
    Save(CommandSave),
    Rgb(rgb::CommandRgb),
    Layout(CommandLayout),
}

#[derive(FromArgs, PartialEq, Debug)]
/// List connected devices
#[argh(subcommand, name = "devices")]
struct CommandDevices {
    /// scan for capabilities
    #[argh(switch, short = 'c')]
    capabilities: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// List connected devices
#[argh(subcommand, name = "lock")]
struct CommandLock {
    /// scan for capabilities
    #[argh(switch, short = 'u')]
    unlock: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Combos operations
#[argh(subcommand, name = "combos")]
struct CommandCombos {
    /// combo number
    #[argh(option, short = 'n')]
    number: Option<u8>,

    /// value expression in format KEY_1 + KEY_2 + KEY_3 + KEY_4 = KEY_5
    #[argh(option, short = 'v')]
    value: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Macros operations
#[argh(subcommand, name = "macros")]
struct CommandMacros {
    /// macro number
    #[argh(option, short = 'n')]
    number: Option<u8>,

    /// value expression in format Text(some text); Tap(KC_1); Down(KC_D); Up(KC_D)
    #[argh(option, short = 'v')]
    value: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// TapDance operations
#[argh(subcommand, name = "tapdances")]
struct CommandTapDances {
    /// tap dance number
    #[argh(option, short = 'n')]
    number: Option<u8>,

    /// value expression in format TAP_KEY + HOLD_KEY + DOUBLE_TAP_KEY + TAPHOLD_KEY ~ TAPPING_TERM_MS
    #[argh(option, short = 'v')]
    value: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// KeyOverride operations
#[argh(subcommand, name = "keyoverrides")]
struct CommandKeyOverrides {
    /// tap dance number
    #[argh(option, short = 'n')]
    number: Option<u8>,

    /// value expression in format trigger=KC_1; replacement=KC_2; layers=1|2|3; trigger_mods=LS|RS; negative_mod_mask=LC|RC; suppressed_mods =LGUI|RGUI; options=ko_enabled|ko_option_activation_trigger_down
    #[argh(option, short = 'v')]
    value: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// AltRepeat operations
#[argh(subcommand, name = "altrepeats")]
struct CommandAltRepeats {
    /// alt repeat number
    #[argh(option, short = 'n')]
    number: Option<u8>,

    /// value expression in format keycode=KC_1; alt_keycode=KC_2; allowed_mods=LS; options=arep_enabled   
    #[argh(option, short = 'v')]
    value: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Layers operations
#[argh(subcommand, name = "layers")]
struct CommandLayers {
    /// meta file (to use instead of vial meta)
    #[argh(option, short = 'm')]
    meta: Option<String>,

    /// show positions instead of assignments
    #[argh(switch, short = 'p')]
    positions: bool,

    /// layer number
    #[argh(option, short = 'n')]
    number: Option<u8>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Key operations
#[argh(subcommand, name = "keys")]
struct CommandKeys {
    /// meta file (to use instead of vial meta)
    #[argh(option, short = 'm')]
    meta: Option<String>,

    /// key layer
    #[argh(option, short = 'l')]
    layer: u8,

    /// key position
    #[argh(option, short = 'p')]
    position: String,

    /// key value
    #[argh(option, short = 'v')]
    value: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Settings operations
#[argh(subcommand, name = "settings")]
struct CommandSettings {
    /// setting identifier
    #[argh(option, short = 'q')]
    qsid: Option<f64>,

    /// set setting value
    #[argh(option, short = 'v')]
    value: Option<String>,

    /// reset all settings into default values
    #[argh(switch, short = 'r')]
    reset: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Load configuration from file
#[argh(subcommand, name = "load")]
struct CommandLoad {
    /// meta file (to use instead of vial meta)
    #[argh(option, short = 'm')]
    meta: Option<String>,

    /// path to layout file
    #[argh(option, short = 'f')]
    file: String,

    /// preview content of layout file instead of loading into keyboard
    #[argh(switch, short = 'p')]
    preview: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Save configuration into file
#[argh(subcommand, name = "save")]
struct CommandSave {
    /// meta file (to use instead of vial meta)
    #[argh(option, short = 'm')]
    meta: Option<String>,

    /// path to layout file
    #[argh(option, short = 'f')]
    file: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Layout options
#[argh(subcommand, name = "layout")]
struct CommandLayout {
    /// meta file (to use instead of vial meta)
    #[argh(option, short = 'm')]
    meta: Option<String>,
}

fn command_for_devices(id: Option<u16>, command: &CommandEnum) {
    match HidApi::new() {
        Ok(api) => {
            for device in api.device_list() {
                if device.usage_page() == protocol::USAGE_PAGE
                    && device.usage() == protocol::USAGE_ID
                    && (id.is_none() || id.unwrap() == device.product_id())
                {
                    println!(
                        "Product name: {:?} id: {:?},\nManufacturer name: {:?}, id: {:?},\nRelease: {:?}, Serial: {:?}, Path: {:?}",
                        device.product_string().unwrap(),
                        device.product_id(),
                        device.manufacturer_string().unwrap(),
                        device.vendor_id(),
                        device.release_number(),
                        device.serial_number().unwrap(),
                        device.path(),
                    );
                    let result = match command {
                        CommandEnum::Devices(ops) => devices::run(&api, device, ops.capabilities),
                        CommandEnum::Lock(ops) => lock::run(&api, device, ops.unlock),
                        CommandEnum::Combos(ops) => {
                            combos::run(&api, device, ops.number, &ops.value)
                        }
                        CommandEnum::Macros(ops) => {
                            macros::run(&api, device, ops.number, &ops.value)
                        }
                        CommandEnum::TapDances(ops) => {
                            tapdances::run(&api, device, ops.number, &ops.value)
                        }
                        CommandEnum::KeyOverrides(ops) => {
                            keyoverrides::run(&api, device, ops.number, &ops.value)
                        }
                        CommandEnum::AltRepeats(ops) => {
                            altrepeats::run(&api, device, ops.number, &ops.value)
                        }
                        CommandEnum::Layers(ops) => {
                            layers::run(&api, device, &ops.meta, ops.positions, ops.number)
                        }
                        CommandEnum::Keys(ops) => keys::run(
                            &api,
                            device,
                            &ops.meta,
                            ops.layer,
                            &ops.position,
                            &ops.value,
                        ),
                        CommandEnum::Settings(ops) => {
                            settings::run(&api, device, &ops.qsid, &ops.value, ops.reset)
                        }
                        CommandEnum::Load(ops) => {
                            load::run(&api, device, &ops.meta, &ops.file, ops.preview)
                        }
                        CommandEnum::Save(ops) => save::run(&api, device, &ops.meta, &ops.file),
                        CommandEnum::Rgb(ops) => rgb::run(&api, device, ops),
                        CommandEnum::Layout(ops) => layout::run(&api, device, &ops.meta),
                    };
                    match result {
                        Ok(_) => {
                            // nothing here
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e)
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
}

fn main() {
    let options: VialClient = argh::from_env();
    if options.version {
        println!("vitaly {}", env!("CARGO_PKG_VERSION"))
    } else if let Some(command) = options.command {
        command_for_devices(options.id, &command);
    } else {
        println!(
            "vitaly {}\nRun vitaly --help for more information.",
            env!("CARGO_PKG_VERSION")
        )
    }
}

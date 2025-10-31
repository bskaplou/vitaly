# Vail CLI tool

Vial CLI tool allows to configure keyboard through VIA/Vial protocol with command line interface.

## Global options

### Identifier

By default tool runs subcommands on all connected devices.

Option --id/-i can be used to select particular device.

For example

```
❯ vitaly devices # all devices
Product name: "Keychron K6 Pro" id: 611,
Manufacturer name: "Keychron", id: 13364,
Release: 256, Serial: "", Path: "DevSrvsID:4294971185"

Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4296802461"


❯ vitaly -i 611 devices # single device
Product name: "Keychron K6 Pro" id: 611,
Manufacturer name: "Keychron", id: 13364,
Release: 256, Serial: "", Path: "DevSrvsID:4294971185"
```

### Devices subcommand

Devices subcommand allows to list compatible devices. For example

```
❯ vitaly devices
Product name: "Keychron K6 Pro" id: 611,
Manufacturer name: "Keychron", id: 13364,
Release: 256, Serial: "", Path: "DevSrvsID:4294971185"

Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4296802461"
```

Flag -c allows to list devices capabilities as well. For example

```
❯ vitaly devices -c
Product name: "Keychron K6 Pro" id: 611,
Manufacturer name: "Keychron", id: 13364,
Release: 256, Serial: "", Path: "DevSrvsID:4294971185"
Capabilities:
	via_version: 12
	vial_version: 0
	companion_hid_version: 1
	layer_count: 5
	tap_dance_count: 0
	combo_count: 0
	key_override_count: 0
	alt_repeat_key_count: 0
	caps_word: false
	layer_lock: false

Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4296802461"
Capabilities:
	via_version: 9
	vial_version: 6
	companion_hid_version: 1
	layer_count: 8
	tap_dance_count: 32
	combo_count: 32
	key_override_count: 32
	alt_repeat_key_count: 32
	caps_word: true
	layer_lock: true
```

### Settings subcommand

Settings subcommand allows to list and alter keyboard settings.

Settings in list are addressed by qsid - QMK setting identifier and with bit for boolean settings encoded into bits.

It allows to dump full list of settings together with current values as follows

```
❯ vitaly -i 4626 settings
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4296802461"

Magic:
	21.0) Swap Caps Lock and Left Control = false
	21.1) Treat Caps Lock as Control = false
	21.2) Swap Left Alt and GUI = false
	21.3) Swap Right Alt and GUI = false
	21.4) Disable the GUI keys = false
	21.5) Swap ` and Escape = false
	21.6) Swap \ and Backspace = false
	21.7) Enable N-key rollover = false
	21.8) Swap Left Control and GUI = false
	21.9) Swap Right Control and GUI = false

Grave Escape:
	1.0) Always send Escape if Alt is pressed = false
	1.1) Always send Escape if Control is pressed = false
	1.2) Always send Escape if GUI is pressed = false
	1.3) Always send Escape if Shift is pressed = false

Tap-Hold:
	7) Tapping Term = 200
	22) Permissive Hold = false
	23) Hold On Other Key Press = false
	24) Retro Tapping = false
	25) Quick Tap Term = 200
	18) Tap Code Delay = 0
	19) Tap Hold Caps Delay = 80
	20) Tapping Toggle = 5
	26) Chordal Hold = false
	27) Flow Tap = 0

Auto Shift:
	3.0) Enable = false
	3.1) Enable for modifiers = false
	4) Timeout = 175
	3.2) Do not Auto Shift special keys = false
	3.3) Do not Auto Shift numeric keys = false
	3.4) Do not Auto Shift alpha characters = false
	3.5) Enable keyrepeat = false
	3.6) Disable keyrepeat when timeout is exceeded = false

Combo:
	2) Time out period for combos = 30

One Shot Keys:
	5) Tapping this number of times holds the key until tapped once again = 5
	6) Time (in ms) before the one shot key is released = 5000

Mouse keys:
	9) Delay between pressing a movement key and cursor movement = 10
	10) Time between cursor movements in milliseconds = 20
	11) Step size = 8
	12) Maximum cursor speed at which acceleration stops = 10
	13) Time until maximum cursor speed is reached = 30
	14) Delay between pressing a wheel key and wheel movement = 10
	15) Time between wheel movements = 80
	16) Maximum number of scroll steps per scroll action = 8
	17) Time until maximum scroll speed is reached = 40
```

It also allows to dump single setting if called with option --qsid/-q as follows 

```
❯ vitaly -i 4626 settings --qsid 6
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4296802461"
6) Time (in ms) before the one shot key is released = 5000
```

It allows to alter setting if called with options -q and -v while -q addresses setting to be altered and -v passess desired value as follows

```

```

*WIP*

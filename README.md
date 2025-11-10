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
❯ vitaly -i 4626 settings --qsid 6 -v 4000
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
Option "Time (in ms) before the one shot key is released" = 4000 now
```

Boolean option example

```
❯ vitaly -i 4626 settings -q 26
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
26) Chordal Hold = false
```

Boolean option encoded into bit

```
❯ vitaly -i 4626 settings -q 3.2 -v true
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
Option "Do not Auto Shift special keys" = true now

❯ vitaly -i 4626 settings -q 3.2
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
3.2) Do not Auto Shift special keys = true
```

### Layers subcommand

Layers subcommand is designed to help with visual layers dumping

It can dump button wiring positings if called with option positions


```
❯ vitaly -i 4626 layers -p
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
        ╔══╗╔══╗                     ╔══╗╔══╗
╔══╗╔══╗║0 ║║0 ║╔══╗             ╔══╗║5 ║║5 ║╔══╗╔══╗
║0 ║║0 ║║2 ║║3 ║║0 ║╔══╗     ╔══╗║5 ║║3 ║║2 ║║5 ║║5 ║
║0 ║║1 ║╚══╝╚══╝║4 ║║0 ║     ║5 ║║4 ║╚══╝╚══╝║1 ║║0 ║
╚══╝╚══╝╔══╗╔══╗╚══╝║5 ║     ║5 ║╚══╝╔══╗╔══╗╚══╝╚══╝
╔══╗╔══╗║1 ║║1 ║╔══╗╚══╝     ╚══╝╔══╗║6 ║║6 ║╔══╗╔══╗
║1 ║║1 ║║2 ║║3 ║║1 ║╔══╗     ╔══╗║6 ║║3 ║║2 ║║6 ║║6 ║
║0 ║║1 ║╚══╝╚══╝║4 ║║1 ║     ║6 ║║4 ║╚══╝╚══╝║1 ║║0 ║
╚══╝╚══╝╔══╗╔══╗╚══╝║5 ║     ║5 ║╚══╝╔══╗╔══╗╚══╝╚══╝
╔══╗╔══╗║2 ║║2 ║╔══╗╚══╝     ╚══╝╔══╗║7 ║║7 ║╔══╗╔══╗
║2 ║║2 ║║2 ║║3 ║║2 ║╔══╗     ╔══╗║7 ║║3 ║║2 ║║7 ║║7 ║
║0 ║║1 ║╚══╝╚══╝║4 ║║2 ║     ║7 ║║4 ║╚══╝╚══╝║1 ║║0 ║
╚══╝╚══╝╔══╗╔══╗╚══╝║5 ║     ║5 ║╚══╝╔══╗╔══╗╚══╝╚══╝
╔══╗╔══╗║3 ║║3 ║╔══╗╚══╝     ╚══╝╔══╗║8 ║║8 ║╔══╗╔══╗
║3 ║║3 ║║2 ║║3 ║║3 ║╔══╗     ╔══╗║8 ║║3 ║║2 ║║8 ║║8 ║
║0 ║║1 ║╚══╝╚══╝║4 ║║3 ║     ║8 ║║4 ║╚══╝╚══╝║1 ║║0 ║
╚══╝╚══╝        ╚══╝║5 ║     ║5 ║╚══╝        ╚══╝╚══╝
                    ╚══╝     ╚══╝

           ╔══╗╔══╗ ╔══╗     ╔══╗ ╔══╗╔══╗
           ║4 ║║4 ║ ║4 ║     ║9 ║ ║9 ║║9 ║
           ║3 ║║4 ║ ║5 ║     ║5 ║ ║4 ║║3 ║
           ╚══╝╚══╝ ╚══╝     ╚══╝ ╚══╝╚══╝
```

It also can dump current layer keymap


```
❯ vitaly -i 4626 layers
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
Layer: 0
        ╔══╗╔══╗                     ╔══╗╔══╗
╔══╗╔══╗║2 ║║3 ║╔══╗             ╔══╗║8 ║║9 ║╔══╗╔══╗
║⎋ ║║1 ║║  ║║  ║║4 ║╔══╗     ╔══╗║7 ║║  ║║  ║║0 ║║- ║
║  ║║  ║╚══╝╚══╝║  ║║5 ║     ║6 ║║  ║╚══╝╚══╝║  ║║  ║
╚══╝╚══╝╔══╗╔══╗╚══╝║  ║     ║  ║╚══╝╔══╗╔══╗╚══╝╚══╝
╔══╗╔══╗║W ║║E ║╔══╗╚══╝     ╚══╝╔══╗║I ║║O ║╔══╗╔══╗
║⇥ ║║Q ║║  ║║  ║║R ║╔══╗     ╔══╗║U ║║  ║║  ║║P ║║= ║
║  ║║  ║╚══╝╚══╝║  ║║T ║     ║Y ║║  ║╚══╝╚══╝║  ║║  ║
╚══╝╚══╝╔══╗╔══╗╚══╝║  ║     ║  ║╚══╝╔══╗╔══╗╚══╝╚══╝
╔══╗╔══╗║S ║║D ║╔══╗╚══╝     ╚══╝╔══╗║K ║║L ║╔══╗╔══╗
║MO║║A ║║  ║║  ║║F ║╔══╗     ╔══╗║J ║║  ║║  ║║; ║║' ║
║1 ║║  ║╚══╝╚══╝║  ║║G ║     ║H ║║  ║╚══╝╚══╝║  ║║  ║
╚══╝╚══╝╔══╗╔══╗╚══╝║  ║     ║  ║╚══╝╔══╗╔══╗╚══╝╚══╝
╔══╗╔══╗║X ║║C ║╔══╗╚══╝     ╚══╝╔══╗║, ║║. ║╔══╗╔══╗
║L⇧║║Z ║║  ║║  ║║V ║╔══╗     ╔══╗║M ║║  ║║  ║║/ ║║R⇧║
║  ║║  ║╚══╝╚══╝║  ║║B ║     ║N ║║  ║╚══╝╚══╝║  ║║  ║
╚══╝╚══╝        ╚══╝║  ║     ║  ║╚══╝        ╚══╝╚══╝
                    ╚══╝     ╚══╝

           ╔══╗╔══╗ ╔══╗     ╔══╗ ╔══╗╔══╗
           ║L⎈║║L⌥║ ║L⌘║     ║␣ ║ ║⏎ ║║R⌘║
           ║  ║║  ║ ║  ║     ║  ║ ║  ║║  ║
           ╚══╝╚══╝ ╚══╝     ╚══╝ ╚══╝╚══╝
```

Layer command accepts --number option which specifies layer number to dump

```
❯ cargo run -- -i 4626 layers -n 2
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/vial_cli -i 4626 layers -n 2`
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
Layer: 2
        ╔══╗╔══╗                     ╔══╗╔══╗
╔══╗╔══╗║▽ ║║▽ ║╔══╗             ╔══╗║▽ ║║▽ ║╔══╗╔══╗
║▽ ║║▽ ║║  ║║  ║║▽ ║╔══╗     ╔══╗║▽ ║║  ║║  ║║▽ ║║▽ ║
║  ║║  ║╚══╝╚══╝║  ║║▽ ║     ║▽ ║║  ║╚══╝╚══╝║  ║║  ║
╚══╝╚══╝╔══╗╔══╗╚══╝║  ║     ║  ║╚══╝╔══╗╔══╗╚══╝╚══╝
╔══╗╔══╗║▽ ║║▽ ║╔══╗╚══╝     ╚══╝╔══╗║Mo║║⚙↓║╔══╗╔══╗
║▽ ║║▽ ║║  ║║  ║║▽ ║╔══╗     ╔══╗║⚙↑║║↑ ║║  ║║▽ ║║▽ ║
║  ║║  ║╚══╝╚══╝║  ║║▽ ║     ║▽ ║║  ║╚══╝╚══╝║  ║║  ║
╚══╝╚══╝╔══╗╔══╗╚══╝║  ║     ║  ║╚══╝╔══╗╔══╗╚══╝╚══╝
╔══╗╔══╗║▽ ║║▽ ║╔══╗╚══╝     ╚══╝╔══╗║Mo║║Mo║╔══╗╔══╗
║▽ ║║▽ ║║  ║║  ║║▽ ║╔══╗     ╔══╗║Mo║║↓ ║║→ ║║⚙→║║▽ ║
║  ║║  ║╚══╝╚══╝║  ║║▽ ║     ║⚙←║║← ║╚══╝╚══╝║  ║║  ║
╚══╝╚══╝╔══╗╔══╗╚══╝║  ║     ║  ║╚══╝╔══╗╔══╗╚══╝╚══╝
╔══╗╔══╗║▽ ║║▽ ║╔══╗╚══╝     ╚══╝╔══╗║Mo║║▽ ║╔══╗╔══╗
║▽ ║║▽ ║║  ║║  ║║▽ ║╔══╗     ╔══╗║Mo║║5 ║║  ║║▽ ║║▽ ║
║  ║║  ║╚══╝╚══╝║  ║║▽ ║     ║Mo║║4 ║╚══╝╚══╝║  ║║  ║
╚══╝╚══╝        ╚══╝║  ║     ║3 ║╚══╝        ╚══╝╚══╝
                    ╚══╝     ╚══╝

           ╔══╗╔══╗ ╔══╗     ╔══╗ ╔══╗╔══╗
           ║▽ ║║▽ ║ ║▽ ║     ║Mo║ ║Mo║║*1║
           ║  ║║  ║ ║  ║     ║1 ║ ║2 ║║  ║
           ╚══╝╚══╝ ╚══╝     ╚══╝ ╚══╝╚══╝
*1 - QK_LAYER_LOCK
```

By default layers command reads keyboard layout data straight from keyboard but it works only for Vial keyboards,
VIA keyboards doesn't have layout data in keyboard memory and it's necessary to pass metadata file as an argument
Such a files can be downloaded from here https://github.com/the-via/keyboards/tree/master/src


```
❯ vitaly -i 611 layers -m k6pro.json
Product name: "Keychron K6 Pro" id: 611,
Manufacturer name: "Keychron", id: 13364,
Release: 256, Serial: "", Path: "DevSrvsID:4316856485"
Layer: 0
╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══════╗╔══╗
║⎋ ║║1 ║║2 ║║3 ║║4 ║║5 ║║6 ║║7 ║║8 ║║9 ║║0 ║║- ║║= ║║⌫     ║║*1║
║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║      ║║  ║
╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══════╝╚══╝
╔════╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔════╗╔══╗
║⇥   ║║Q ║║W ║║E ║║R ║║T ║║Y ║║U ║║I ║║O ║║P ║║( ║║) ║║\   ║║⇱ ║
║    ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║    ║║  ║
╚════╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚════╝╚══╝
╔═════╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔═══════╗╔══╗
║⇪    ║║A ║║S ║║D ║║F ║║G ║║H ║║J ║║K ║║L ║║; ║║' ║║⏎      ║║⇞ ║
║     ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║       ║║  ║
╚═════╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚═══════╝╚══╝
╔═══════╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗╔═════╗╔══╗╔══╗
║L⇧     ║║Z ║║X ║║C ║║V ║║B ║║N ║║M ║║, ║║. ║║/ ║║R⇧   ║║↑ ║║⇟ ║
║       ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║  ║║     ║║  ║║  ║
╚═══════╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝╚═════╝╚══╝╚══╝
╔═══╗╔═══╗╔═══╗╔═══════════════════════╗╔══╗╔══╗╔══╗╔══╗╔══╗╔══╗
║L⎈ ║║⌨  ║║⌨  ║║␣                      ║║⌨ ║║MO║║TG║║← ║║↓ ║║→ ║
║   ║║0  ║║2  ║║                       ║║3 ║║2 ║║4 ║║  ║║  ║║  ║
╚═══╝╚═══╝╚═══╝╚═══════════════════════╝╚══╝╚══╝╚══╝╚══╝╚══╝╚══╝
*1 - QK_BACKLIGHT_STEP
```

::: warning
For now vitaly doesn't support keybord meta files with rotated buttons, support might be added later
:::

::: warning
--meta option is temporary solution, later keyboard metadata database can be embedded into application
:::


### Keys subcommand

Keys subcommand is able to dump and assign signle button to layer and position

Dump

```
❯ vitaly -i 4626 keys -l 0 -p 0,0
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
Key on layer=0, row=0, col=0 => KC_ESCAPE
```

Assign

```
❯ vitaly -i 4626 keys -l 0 -p 0,0 -v KC_COMMA
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
Key on layer=0, row=0, col=0 set to => KC_COMMA, keycode = 0x36
```

Key command accepts all official QMK keycodes listed here together with aliases https://docs.qmk.fm/keycodes_basic .
It is expected to support all the things which are proposed by QMK itself like MO(1), MT(MOD\_LSFT,KC\_2), LSFT(KC\_1) etc...

### Combos subcommand


Combos subcommand is able to dump existing combos and add new combos to buttons

Dump

```
❯ vitaly -i 4626 combos
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
Combos list:
0) KC_E + KC_R = LSFT(KC_9)
1) KC_U + KC_I = LSFT(KC_0)
2) KC_Q + KC_W = LSFT(KC_GRAVE)
3) KC_S + KC_D = MO(4)
4) KC_AUDIO_VOL_DOWN + KC_AUDIO_VOL_UP = KC_AUDIO_MUTE
5) KC_K + KC_L = MO(4)
6) KC_O + KC_P = LSFT(KC_BACKSLASH)
7) KC_A + KC_S = KC_GRAVE
8) KC_W + KC_E = TG(5)
9) KC_D + KC_F = LSFT(KC_LEFT_BRACKET)
10) KC_J + KC_K = LSFT(KC_RIGHT_BRACKET)
11) KC_Z + KC_X = MO(3)
12) KC_L + KC_SEMICOLON = KC_BACKSLASH
13) KC_C + KC_V = KC_LEFT_BRACKET
14) KC_M + KC_COMMA = KC_RIGHT_BRACKET
15) KC_COMMA + KC_DOT = KC_BACKSPACE
16) KC_X + KC_C = MO(2)
17) KC_DOT + KC_SLASH = MO(3)
18) KC_LEFT_SHIFT + KC_RIGHT_SHIFT = QK_CAPS_WORD_TOGGLE
Combo slots 19 - 31 are EMPTY
```

New combo

```
❯ vitaly -i 4626 combos -n 19 -v 'KC_1 + KC_2 = KC_3'
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
Saving combo 19) KC_1 + KC_2 = KC_3
```


Delete combo 
```
❯ vitaly -i 4626 combos -n 19 -v 'KC_NO = KC_NO'
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
Saving combo 19) EMPTY
```

### Macros subcommand

Macros subcommand allows to dump and define macroses.

Value format is

```
Text(some text); Tap(KC_1); Down(KC_D); Up(KC_D)
```

Full dump example

```
❯ vitaly -i 4626 macros
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4317317913"
Macros list:
0) Delay(5); Delay(1000)
1) Text(test)
2) Down(KC_1)
3) Up(KC_1)
4) Tap(KC_1)
5) Text(TEST); Delay(100)
6) Tap(QK_KB_0)
```

Single macro dump example

```
❯ vitaly -i 4626 macros -n 5
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4317317913"
5) Text(TEST); Delay(100)
```

Define new macro

```
❯ vitaly -i 4626 macros -n 7 -v 'Tap(KC_E); Delay(100); Tap(KC_S)'
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4317317913"
Updated macros list:
0) Delay(5); Delay(1000)
1) Text(test)
2) Down(KC_1)
3) Up(KC_1)
4) Tap(KC_1)
5) Text(TEST); Delay(100)
6) Tap(QK_KB_0)
7) Tap(KC_E); Delay(100); Tap(KC_S)
Macros successfully updated
```

Delete macro

```
❯ vitaly -i 4626 macros -n 7 -v ''
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4317317913"
Updated macros list:
0) Delay(5); Delay(1000)
1) Text(test)
2) Down(KC_1)
3) Up(KC_1)
4) Tap(KC_1)
5) Text(TEST); Delay(100)
6) Tap(QK_KB_0)
Macros successfully updated
```

### TapDances command

Tap dances command allows to dump and define tapdances.

Value format is

```
TAP_KEY + HOLD_KEY + DOUBLE_TAP_KEY + TAPHOLD_KEY ~ TAPPING_TERM_MS
```

Define example
```
❯ cargo run -- -i 4626 tapdances -n 0 -v 'KC_1 + LSFT(KC_1) ~ 30'
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
Saving tap dance 0) On tap: KC_1, On hold: LSFT(KC_1), Tapping term (ms) = 30
```

Dump

```
❯ vitaly -i 4626 tapdances
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
TapDance list:
0) On tap: KC_1, On hold: LSFT(KC_1), Tapping term (ms) = 30
TapDance slots 1 - 31 are EMPTY
```

Undefine tapdance
```
❯ vitaly -i 4626 tapdances -n 0 -v 'KC_NO ~ 30'
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
Saving tap dance 0) EMPTY
```


### KeyOverrides subcommand

KeyOverrides subcommand is designed to dump and define key overrides

Dump 
```
❯ vitaly -i 4626 keyoverrides
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
KeyOverride list:
0) trigger = KC_BACKSPACE; replacement = KC_DELETE; layers = 0;
	trigger_mods = MOD_BIT_LSHIFT;
	negative_mod_mask = KC_NO;
	suppressed_mods = MOD_BIT_LSHIFT;
	ko_option_activation_trigger_down = true
	ko_option_activation_required_mod_down = false
	ko_option_activation_negative_mod_up = false
	ko_option_one_mod = false
	ko_option_no_reregister_trigger = false
	ko_option_no_unregister_on_other_key_down = false
	ko_enabled = true
1) EMPTY
...
```

value format is
```
trigger=KC_1; replacement=KC_2; layers=1|2|3; trigger_mods=LS|RS; negative_mod_mask=LC|RC; suppressed_mods =LGUI|RGUI; options=ko_enabled|ko_option_activation_trigger_down
```

Define key override example

```
❯ vitaly -i 4626 keyoverrides -n 1 -v 'trigger=KC_1; replacement=KC_2; layers=0; trigger_mods=MOD_BIT_LSHIFT; suppressed_mods=MOD_BIT_LSHIFT; options=ko_enabled'
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
Key override 1) trigger = KC_1; replacement = KC_2; layers = 0;
	trigger_mods = MOD_BIT_LSHIFT;
	negative_mod_mask = KC_NO;
	suppressed_mods = MOD_BIT_LSHIFT;
	ko_option_activation_trigger_down = false
	ko_option_activation_required_mod_down = false
	ko_option_activation_negative_mod_up = false
	ko_option_one_mod = false
	ko_option_no_reregister_trigger = false
	ko_option_no_unregister_on_other_key_down = false
	ko_enabled = true
Saved
```

Clean key override

```
❯ vitaly -i 4626 keyoverrides -n 1 -v 'trigger=KC_NO'
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
Key override 1) EMPTY
Saved
```

### AltRepeats

AltRepeats subcommand allows to dump and define subrepeats


Define alt repleat

```
❯ vitaly -i 4626 altrepeats -n 0 -v 'keycode=KC_1; alt_keycode=KC_2; allowed_mods=MOD_BIT_LSHIFT; options=arep_enabled'
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
Saving altrepeat 0) keycode = KC_1; alt_keycode = KC_2;
	allowed_mods = MOD_BIT_LSHIFT;
	arep_option_default_to_this_alt_key = false
	arep_option_bidirectional = false
	arep_option_ignore_mod_handedness = false
	arep_enabled = true
```

List defined alt repeats

```
❯ vitaly -i 4626 altrepeats
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
AltRepeat list:
0) keycode = KC_1; alt_keycode = KC_2;
	allowed_mods = MOD_BIT_LSHIFT;
	arep_option_default_to_this_alt_key = false
	arep_option_bidirectional = false
	arep_option_ignore_mod_handedness = false
	arep_enabled = true
1) EMPTY
...
```

Delete alt repeat

```
❯ vitaly -i 4626 altrepeats -n 0 -v 'keycode=KC_NO'
Product name: "silakka54" id: 4626,
Manufacturer name: "Squalius-cephalus", id: 65261,
Release: 256, Serial: "vial:f64c2b3c", Path: "DevSrvsID:4316856206"
Saving altrepeat 0) EMPTY
```

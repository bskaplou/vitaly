mod altrepeats;
pub use crate::commands::altrepeats::run as altrepeats_run;

mod save;
pub use crate::commands::save::run as save_run;

mod load;
pub use crate::commands::load::run as load_run;

mod layers;
pub use crate::commands::layers::run as layers_run;

mod tapdances;
pub use crate::commands::tapdances::run as tapdances_run;

mod settings;
pub use crate::commands::settings::run as settings_run;

mod rgb;
pub use crate::commands::rgb::{CommandRgb, run as rgb_run};

mod macros;
pub use crate::commands::macros::run as macros_run;

mod lock;
pub use crate::commands::lock::run as lock_run;

mod layout;
pub use crate::commands::layout::run as layout_run;

mod keys;
pub use crate::commands::keys::run as keys_run;

mod keyoverrides;
pub use crate::commands::keyoverrides::run as keyoverrides_run;

mod encoders;
pub use crate::commands::encoders::run as encoders_run;

mod devices;
pub use crate::commands::devices::run as devices_run;

mod combos;
pub use crate::commands::combos::run as combos_run;

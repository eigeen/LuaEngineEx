pub mod monster;

use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum HookError {
    #[snafu(display("{reason}: {source}"))]
    Hook {
        source: mhw_toolkit::game::hooks::HookError,
        reason: String,
    },
}

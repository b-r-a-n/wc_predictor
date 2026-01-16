//! CLI command implementations.

pub mod match_cmd;
pub mod simulate;
pub mod team;
pub mod teams;

pub use match_cmd::run_match;
pub use simulate::run_simulate;
pub use team::run_team;
pub use teams::run_teams;

# Updates the steps needed to compute a given path.
cargo bench --bench steps_complete >baseline
cargo bench --bench steps_partial >>baseline

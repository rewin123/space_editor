cd crates/shared
cargo publish --dry-run
timeout /t 10
cd ..
cd undo
cargo publish --dry-run
timeout /t 10
cd ..
cd prefab
cargo publish --dry-run
timeout /t 10
cd ..
cd persistance
cargo publish --dry-run
timeout /t 10
cd ..
cd editor_core
cargo publish
timeout /t 10
cd ..
cd editor_ui
cargo publish
timeout /t 10
cd ..
cd ..
cd modules
cd bevy_xpbd_plugin
cargo publish
timeout /t 10
cd ..
cd ..
cargo publish
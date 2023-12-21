cd crates/shared
cargo publish
timeout /t 600
cd ..
cd undo
cargo publish
timeout /t 600
cd ..
cd prefab
cargo publish
timeout /t 600
cd ..
cd persistance
cargo publish
timeout /t 600
cd ..
cd editor_core
cargo publish
timeout /t 600
cd ..
cd editor_ui
cargo publish
timeout /t 600
cd ..
cd ..
cd modules
cd bevy_xpbd_plugin
cargo publish
timeout /t 600
cd ..
cd ..
cargo publish
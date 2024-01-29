plugin_path="/home/dragonblade316/.config/rstreamdeck/plugins"

cargo build

rm $plugin_path/basic_plugin
rm $plugin_path/media_control

cp ./target/debug/basic_plugin $plugin_path/basic_plugin
cp ./target/debug/media_control $plugin_path/media_control

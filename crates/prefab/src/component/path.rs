//Planned to use trait for support file dialog per field

use shared::ext::egui_file;

pub trait AssetPath {
    fn get_filter(&self) -> egui_file::Filter;
    fn set_path(&mut self, path: &str);
    fn get_path_mut(&mut self) -> &mut String;
}

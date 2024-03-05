//Planned to use trait for support file dialog per field

use kcg_shared::ext::egui_file;

/// NOT USED. Will be used to auto converting reflected structures with assets to saveble AutoStructs
pub trait AssetPath {
    fn get_filter<T: std::ops::Deref>(&self) -> egui_file::Filter<T>;
    fn set_path(&mut self, path: &str);
    fn get_path_mut(&mut self) -> &mut String;
}

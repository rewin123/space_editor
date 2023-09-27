
//Planned to use trait for support file dialog per field

pub trait AssetPath {
    fn get_filter(&self) -> egui_file::Filter;
    fn set_path(&mut self, path : &str);
    fn get_path_mut(&mut self) -> &mut String;
}
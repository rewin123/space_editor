
use bevy::{prelude::*, ecs::{component::ComponentId, change_detection::MutUntyped, world::unsafe_world_cell::UnsafeWorldCell}, reflect::{ReflectFromPtr, TypeInfo, DynamicEnum, DynamicVariant, DynamicTuple, DynamicStruct}, ptr::PtrMut};
use bevy_egui::*;

use super::registration::EditorRegistry;


pub fn ui_for_reflect(
        ui : &mut egui::Ui,
        value : &mut dyn Reflect,
        hash : &str,
        name : &str,
        set_changed : &mut impl FnMut(),
        world : &mut UnsafeWorldCell) {

    //custom reflect detect
    unsafe {
        let mut editor: Mut<'_, EditorRegistry> = world.get_resource_mut::<EditorRegistry>().unwrap();
        if let Some(custom_reflect) = editor.custom_reflect.get_mut(&value.type_id()) {
            (custom_reflect.reflect)(ui, value, hash, name, set_changed, world);
            return;
        }
    }

    //default reflect path
    match value.reflect_mut() {
        bevy::reflect::ReflectMut::Struct(value) => {ui_for_struct(ui, value, hash, name, set_changed, world)},
        bevy::reflect::ReflectMut::TupleStruct(value) => {ui_for_tuple_struct(ui, value, hash, name, set_changed, world)},
        bevy::reflect::ReflectMut::Tuple(value) => {ui_for_tuple(ui, value, hash, name, set_changed, world)},
        bevy::reflect::ReflectMut::List(value) => {ui_for_list(ui, value, hash, name, set_changed, world)},
        bevy::reflect::ReflectMut::Array(value) => {ui_for_array(ui, value, hash, name, set_changed, world)},
        bevy::reflect::ReflectMut::Map(value) => {ui_for_map(ui, value, hash, name, set_changed, world)},
        bevy::reflect::ReflectMut::Enum(value) => {ui_for_enum(ui, value, hash, name, set_changed, world)},
        bevy::reflect::ReflectMut::Value(value) => {ui_for_value(ui, value, hash, name, set_changed, world)},
    }
}



pub fn ui_for_map(
    ui : &mut egui::Ui,
    value : &mut dyn bevy::reflect::Map,
    hash : &str,
    name : &str,
    set_changed : &mut impl FnMut(),
    world : &mut UnsafeWorldCell
) {
    let hash = format!("{}{}", hash, name);
    ui.label(name);
    ui.indent(&hash, |ui| {
        for idx in 0..value.len() {
            let (key, subvalue) = value.get_at_mut(idx).unwrap();
            let subname = format!("{:?}", key);
            ui_for_reflect(ui, subvalue , &format!("{}{}", hash, subname), &subname, set_changed, world);
        }
    });
}

pub  fn ui_for_tuple(
    ui : &mut egui::Ui,
    value : &mut dyn bevy::reflect::Tuple,
    hash : &str,
    name : &str,
    set_changed : &mut impl FnMut(),
    world : &mut UnsafeWorldCell
) {
    let hash = format!("{}{}", hash, name);
    ui.label(name);
    ui.indent(&hash, |ui| {
        for idx in 0..value.field_len() {
            let subname = format!("{}", idx);
            ui_for_reflect(ui, value.field_mut(idx).unwrap(), &format!("{}{}", hash, subname), &subname, set_changed, world);
        }
    });
}

pub fn ui_for_array(
    ui : &mut egui::Ui,
    value : &mut dyn bevy::reflect::Array,
    hash : &str,
    name : &str,
    set_changed : &mut impl FnMut(),
    world : &mut UnsafeWorldCell
) {
let hash = format!("{}{}", hash, name);
    ui.label(name);
    ui.indent(&hash, |ui| {
        for idx in 0..value.len() {
            let subname = format!("{}", idx);
            ui_for_reflect(ui, value.get_mut(idx).unwrap(), &format!("{}{}", hash, subname), &subname, set_changed, world);
        }
    });
}

pub fn ui_for_list(
    ui : &mut egui::Ui,
    value : &mut dyn bevy::reflect::List,
    hash : &str,
    name : &str,
    set_changed : &mut impl FnMut(),
    world : &mut UnsafeWorldCell
) {
    let hash = format!("{}{}", hash, name);
    if name != "" {
    ui.label(name);
        ui.indent(&hash, |ui| {
            for idx in 0..value.len() {
                let subname = format!("{}", idx);
                ui_for_reflect(ui, value.get_mut(idx).unwrap(), &format!("{}{}", hash, subname), &subname, set_changed, world);
            }
        });
    } else {
        for idx in 0..value.len() {
            let subname = format!("{}", idx);
            ui_for_reflect(ui, value.get_mut(idx).unwrap(), &format!("{}{}", hash, subname), &subname, set_changed, world);
        }
    }
}

pub fn ui_for_enum(
    ui : &mut egui::Ui,
    value : &mut dyn bevy::reflect::Enum,
    hash : &str,
    name : &str,
    set_changed : &mut impl FnMut(),
    world : &mut UnsafeWorldCell
) {
    let hash = format!("{}{}", hash, name);
    let selected_name = value.variant_name().to_string();
    let TypeInfo::Enum(enum_info) = value.get_represented_type_info().unwrap() else {
        ui.label("Broken enum");
        return;
    };
    let mut next_name = selected_name.clone();
    egui::ComboBox::new(&hash, name)
            .selected_text(&selected_name)
            .show_ui(ui, |ui| {
        for idx in 0..enum_info.variant_len() {
            let name = enum_info.variant_at(idx).unwrap().name();
            ui.selectable_value(&mut next_name, name.to_string(), name);
        }
    });
    if next_name != selected_name {
        let info = enum_info.variant(&next_name).unwrap();
        let variant = match info {
            bevy::reflect::VariantInfo::Struct(s) => DynamicVariant::Struct(DynamicStruct::default()),
            bevy::reflect::VariantInfo::Tuple(s) => DynamicVariant::Tuple(DynamicTuple::default()),
            bevy::reflect::VariantInfo::Unit(s) => DynamicVariant::Unit,
        };
        let new_variant = DynamicEnum::new(next_name, variant);
        value.apply(&new_variant);
    }
    ui.label(&format!("Enum: {}", selected_name));
    if value.field_len() > 0 {
        ui.indent(&format!("{}indent", hash), |ui| {
            for idx in 0..value.field_len() {
                let name = value.name_at(idx).unwrap_or("").to_string();
                let field = value.field_at_mut(idx).unwrap();
                ui_for_reflect(ui, field, &hash, &name, set_changed, world);
            }
        });
    }
}

pub fn ui_for_tuple_struct(
    ui : &mut egui::Ui,
    value : &mut dyn TupleStruct,
    hash : &str,
    name : &str,
    set_changed : &mut impl FnMut(),
    world : &mut UnsafeWorldCell
) {
    let hash = format!("{}{}", hash, value.type_name());
    if name != "" {
        egui::CollapsingHeader::new(format!("{}", name))
                .show(ui, |ui| {
            if value.field_len() == 1 {
                ui_for_reflect(ui, value.field_mut(0).unwrap(), format!("{}{}", hash, &format!("{}", 0)).as_str(), "", set_changed, world)
            } else {
                for idx in 0..value.field_len() {
                    if let Some(field) = value.field_mut(idx) {
                        ui_for_reflect(ui, field, format!("{}{}", hash, &format!("{}", idx)).as_str(), &format!("{}", idx), set_changed, world)
                    }
                }
            }
        });
    } else {
        if value.field_len() == 1 {
            ui_for_reflect(ui, value.field_mut(0).unwrap(), format!("{}{}", hash, &format!("{}", 0)).as_str(), "", set_changed, world)
        } else {
            for idx in 0..value.field_len() {
                if let Some(field) = value.field_mut(idx) {
                    ui_for_reflect(ui, field, format!("{}{}", hash, &format!("{}", idx)).as_str(), &format!("{}", idx), set_changed, world)
                }
            }
        }
    }
}

pub fn ui_for_value(
    ui : &mut egui::Ui,
    value : &mut dyn Reflect,
    hash : &str,
    name : &str,
    set_changed : &mut impl FnMut(),
    world : &mut UnsafeWorldCell) {
let hash = format!("{}{}", hash, value.type_name());

if value.represents::<f32>() {
    let val = value.downcast_mut::<f32>().unwrap();
    ui.horizontal(|ui| {
        ui.label(name);
        if ui.add(egui::DragValue::new(val).min_decimals(2).speed(0.1)).changed() {
            set_changed();
        }
    });
} else if value.represents::<Entity>() {
    let val = value.downcast_ref::<Entity>().unwrap();
    ui.label(format!("{} : {:?}", name, val));
} else if value.represents::<String>() {
    let val = value.downcast_mut::<String>().unwrap();   
    ui.horizontal(|ui| {
        ui.label(name);
        ui.add(egui::TextEdit::singleline(val));
    });
} else {
    ui.label(format!("{} not reflected", name));
}
}

pub fn ui_for_struct(
    ui : &mut egui::Ui,
    value : &mut dyn Struct,
    hash : &str,
    name : &str,
    set_changed : &mut impl FnMut(),
    world : &mut UnsafeWorldCell) {
        
    let hash = format!("{}{}", hash, value.type_name());
    if name != "" {
        egui::CollapsingHeader::new(format!("{}", name))
                .show(ui, |ui| {
            for idx in 0..value.field_len() {
                let mut name = "".to_string();
                if let Some(name_str) = value.name_at(idx) {
                    name = name_str.to_string();
                }
                if let Some(field) = value.field_at_mut(idx) {
                    ui_for_reflect(ui, field, format!("{}{}", hash, name).as_str(), &name, set_changed, world)
                }
            }
        });
    } else {
        for idx in 0..value.field_len() {
            let mut name = "".to_string();
            if let Some(name_str) = value.name_at(idx) {
                name = name_str.to_string();
            }
            if let Some(field) = value.field_at_mut(idx) {
                ui_for_reflect(ui, field, format!("{}{}", hash, name).as_str(), &name, set_changed, world)
            }
        }
    }
}
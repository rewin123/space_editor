use std::any::{Any, TypeId};

use bevy::{prelude::{AppTypeRegistry, ResMut}, reflect::Reflect};
use bevy_egui::egui;
use bevy_inspector_egui::{reflect_inspector::InspectorUi, inspector_egui_impls::InspectorEguiImpl};

use crate::prefab::component::EntityLink;



fn many_unimplemented<T: Any>(
    ui: &mut egui::Ui,
    _options: &dyn Any,
    _id: egui::Id,
    _env: InspectorUi<'_, '_>,
    _values: &mut [&mut dyn Reflect],
    _projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
) -> bool { 

    false
}

fn setup_ref_registry(
    mut reg : ResMut<AppTypeRegistry>
) {
    let mut reg = reg.write();
    reg.get_mut(TypeId::of::<EntityLink>())
        .unwrap_or_else(|| panic!("EntityRef not registered!"))
        .insert(
            InspectorEguiImpl::new(
                entity_ref_ui,
                entity_ref_ui_readonly,
                many_unimplemented::<EntityLink>
            )
        )
}

fn entity_ref_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    options: &dyn Any,
    id: egui::Id,
    mut env: InspectorUi<'_, '_>,
) -> bool {
    if let Some(value) = value.downcast_mut::<EntityLink>() {
        if let Some(world) = &env.context.world {
            egui::ComboBox::new(id, "")
                .selected_text(format!("{:?}", value.entity))
                .show_ui(ui, |ui| {
                    unsafe {
                        for e in world.world().world().iter_entities() {
                            if ui.selectable_value(&mut value.entity, e.id(), format!("{:?}", e.id())).clicked() {
                                return true;
                            }
                        }
                    }
                    false
                });
            false
        } else {
            ui.label(format!("{:?}", &value.entity));
            false
        }
    } else {
        false
    }
}

pub fn entity_ref_ui_readonly(
    value: &dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    _: egui::Id,
    _: InspectorUi<'_, '_>,
) {

}
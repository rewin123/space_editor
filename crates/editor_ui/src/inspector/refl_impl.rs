use std::any::{Any, TypeId};

use bevy::{
    prelude::{AppTypeRegistry, ResMut},
    reflect::Reflect,
};
use bevy_egui_next::egui;
use space_shared::ext::bevy_inspector_egui::{
    inspector_egui_impls::InspectorEguiImpl, reflect_inspector::InspectorUi,
};

use space_prefab::component::EntityLink;

/// Method from `bevy_inspector_egui` to make dummy reflection ui
pub fn many_unimplemented<T: Any>(
    _ui: &mut egui::Ui,
    _options: &dyn Any,
    _id: egui::Id,
    _env: InspectorUi<'_, '_>,
    _values: &mut [&mut dyn Reflect],
    _projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
) -> bool {
    false
}

/// Custom UI for [`EntityLink`] struct
pub fn setup_ref_registry(reg: ResMut<AppTypeRegistry>) {
    let mut reg = reg.write();
    reg.get_mut(TypeId::of::<EntityLink>())
        .unwrap_or_else(|| panic!("EntityRef not registered!"))
        .insert(InspectorEguiImpl::new(
            entity_ref_ui,
            entity_ref_ui_readonly,
            many_unimplemented::<EntityLink>,
        ))
}

/// Custom UI for [`EntityLink`] struct
pub fn entity_ref_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _options: &dyn Any,
    id: egui::Id,
    env: InspectorUi<'_, '_>,
) -> bool {
    if let Some(value) = value.downcast_mut::<EntityLink>() {
        if let Some(world) = &env.context.world {
            egui::ComboBox::new(id, "")
                .selected_text(format!("{:?}", value.entity))
                .show_ui(ui, |ui| {
                    let world_ref = unsafe { world.world().world() };
                    for e in world_ref.iter_entities() {
                        if ui
                            .selectable_value(&mut value.entity, e.id(), format!("{:?}", e.id()))
                            .clicked()
                        {
                            return true;
                        }
                    }
                    false
                });
        } else {
            ui.label(format!("{:?}", &value.entity));
        }
    }
    false
}

/// Custom UI for [`EntityLink`] struct
pub fn entity_ref_ui_readonly(
    _value: &dyn Any,
    _ui: &mut egui::Ui,
    _: &dyn Any,
    _: egui::Id,
    _: InspectorUi<'_, '_>,
) {
}

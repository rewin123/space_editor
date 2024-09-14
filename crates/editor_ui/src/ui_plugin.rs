use crate::tools::gizmo::*;
use crate::*;
use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use meshless_visualizer::draw_light_gizmo;

use self::{change_chain::ChangeChainViewPlugin, editor_tab_name::EditorTabName};

/// All systems for editor ui will be placed in UiSystemSet
#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct UiSystemSet;

/// Plugin for editor ui
pub struct EditorUiPlugin {
    pub use_standard_layout: bool,
}

impl Default for EditorUiPlugin {
    fn default() -> Self {
        Self {
            use_standard_layout: true,
        }
    }
}

/// State to determine if editor ui should be shown (or hidden for any reason)
#[derive(Hash, PartialEq, Eq, Debug, Clone, States, Default)]
pub enum ShowEditorUi {
    #[default]
    Show,
    Hide,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct EditorGizmo;

impl FlatPluginList for EditorUiPlugin {
    #[cfg(not(tarpaulin_include))]
    fn add_plugins_to_group(&self, group: PluginGroupBuilder) -> PluginGroupBuilder {
        let mut res = group
            .add(SelectedPlugin)
            .add(MeshlessVisualizerPlugin)
            .add(EditorUiCore::default())
            .add(GameViewPlugin)
            .add(menu_toolbars::BottomMenuPlugin)
            .add(MouseCheck)
            .add(CameraViewTabPlugin)
            .add(SpaceHierarchyPlugin::default())
            .add(SpaceInspectorPlugin)
            .add(GizmoToolPlugin)
            .add(ChangeChainViewPlugin)
            .add(settings::SettingsWindowPlugin);

        if self.use_standard_layout {
            res = res.add(DefaultEditorLayoutPlugin);
        }

        res
    }
}

impl PluginGroup for EditorUiPlugin {
    #[cfg(not(tarpaulin_include))]
    fn build(self) -> PluginGroupBuilder {
        let mut group = PluginGroupBuilder::start::<Self>();
        group = self.add_plugins_to_group(group);
        group
    }
}

pub struct DefaultEditorLayoutPlugin;

impl Plugin for DefaultEditorLayoutPlugin {
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        app.init_layout_group::<DoublePanelGroup, _>();

        app.layout_push::<DoublePanelGroup, _, _>(DoublePanel::Main, EditorTabName::GameView);
        app.layout_push::<DoublePanelGroup, _, _>(DoublePanel::TopLeft, EditorTabName::Hierarchy);
        app.layout_push::<DoublePanelGroup, _, _>(
            DoublePanel::BottomLeft,
            EditorTabName::Inspector,
        );
    }
}

pub struct EditorUiCore {
    pub disable_no_editor_cams: bool,
}

impl Default for EditorUiCore {
    fn default() -> Self {
        Self {
            disable_no_editor_cams: true,
        }
    }
}

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct AfterStateTransition;

impl Plugin for EditorUiCore {
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        use bevy::app::MainScheduleOrder;

        app.init_state::<ShowEditorUi>();
        app.init_resource::<EditorUi>();

        app.configure_sets(
            Update,
            UiSystemSet
                .in_set(EditorSet::Editor)
                .run_if(in_state(EditorState::Editor).and_then(in_state(ShowEditorUi::Show))),
        );

        app.init_resource::<ScheduleEditorTabStorage>();
        app.add_systems(
            Update,
            (
                show_editor_ui
                    .before(update_pan_orbit)
                    .before(ui_camera_block)
                    .after(menu_toolbars::top_menu)
                    .after(menu_toolbars::bottom_menu),
                set_camera_viewport,
            )
                .in_set(UiSystemSet)
                .before(PanOrbitCameraSystemSet),
        );

        app.add_systems(
            PostUpdate,
            set_camera_viewport
                .run_if(has_window_changed)
                .in_set(UiSystemSet),
        );
        app.add_systems(
            Update,
            reset_camera_viewport.run_if(in_state(EditorState::Game)),
        );
        app.add_systems(OnEnter(ShowEditorUi::Hide), reset_camera_viewport);
        app.editor_tab_by_trait(GameViewTab::default());

        app.editor_tab_by_trait(self::debug_panels::DebugWorldInspector {});

        app.init_resource::<EditorLoader>();

        app.insert_resource(EditorCameraEnabled(true));

        //app.add_systems(
        //    Startup,
        //    (set_start_state, apply_state_transition::<EditorState>).chain(),
        //);

        // Create a new schedule for systems that need to run after state transition
        let after_state_transition = Schedule::new(AfterStateTransition);
        app.add_schedule(after_state_transition);

        // Modify the schedule order to make this run after `StateTransition`
        app.world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_after(StateTransition, AfterStateTransition);

        app.add_systems(Startup, (set_start_state, apply_deferred).chain());

        //play systems
        app.add_systems(OnEnter(EditorState::GamePrepare), save_prefab_before_play);
        // clean up meshless children on entering the game state
        app.add_systems(OnEnter(EditorState::GamePrepare), clean_meshless);
        app.add_systems(
            OnEnter(SaveState::Idle),
            to_game_after_save.run_if(in_state(EditorState::GamePrepare)),
        );

        app.add_systems(OnEnter(EditorState::Game), change_camera_in_play);

        app.add_systems(
            OnEnter(EditorState::Editor),
            (clear_and_load_on_start, set_camera_viewport),
        );

        app.add_systems(
            Update,
            (
                draw_camera_gizmo,
                draw_light_gizmo,
                selection::delete_selected,
            )
                .run_if(in_state(EditorState::Editor).and_then(in_state(ShowEditorUi::Show))),
        );

        if self.disable_no_editor_cams {
            app.add_systems(
                Update,
                disable_no_editor_cams.run_if(in_state(EditorState::Editor)),
            );

            app.add_systems(OnEnter(EditorState::Editor), change_camera_in_editor);
        }

        app.add_event::<selection::SelectEvent>();

        app.init_resource::<BundleReg>();
    }
}

/// System to block camera control if egui is using mouse
pub fn ui_camera_block(
    mut ctxs: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut state: ResMut<EditorCameraEnabled>,
    game_view: Res<GameViewTab>,
) {
    let Ok(mut ctx_ref) = ctxs.get_single_mut() else {
        return;
    };
    let ctx = ctx_ref.get_mut();
    if ctx.is_using_pointer() || ctx.is_pointer_over_area() {
        let Some(pos) = ctx.pointer_latest_pos() else {
            return;
        };
        if let Some(area) = game_view.viewport_rect {
            if area.contains(pos) {
            } else {
                *state = EditorCameraEnabled(false);
            }
        } else {
            *state = EditorCameraEnabled(false);
        }
    }
}

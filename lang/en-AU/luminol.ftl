# Copyright (C) 2023 Lily Lyons
# 
# This file is part of Luminol.
# 
# Luminol is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
# 
# Luminol is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
# 
# You should have received a copy of the GNU General Public License
# along with Luminol.  If not, see <http://www.gnu.org/licenses/>.

luminol = Luminol

# General words
start = Start
unloaded = Unloaded
scale = Scale
events = Events
fog = Fog
new_project = New Project
open_project = Open Project
fullscreen = Fullscreen
maps = Maps
items = Items
common_event = Common Event
common_events = Common Events
scripts = Scripts
sound_test = Sound Test
about = About
ok = Ok
cancel = Cancel
apply = Apply
name = Name
icon = Icon
description = Description
scope = Scope
occasion = Occasion
search = Search
code = Code
type = Type
parameters = Parameters
save = Save
position = Position
variants = Variants
switch = Switch
variable = Variable
example_allcaps = EXAMPLE

# Errors
fatal_error = Fatal Error
deadlock_detected_title = Deadlock #{$deadlockIndex}
deadlock_detected_description = Luminol has deadlocked! Please file an issue.
    { $numOfDeadLocks } deadlocks detected
thread_id = Thread Id {$id}

# Tabs
# > Started
tab_started_title_label = Get Started
tab_started_recent_projects_label = Recent
# > Map
tab_map_title_label = Map {$id}: {$name}
tab_map_layer_section_sv = Layer {$num}
tab_map_panorama_label = Panorama
tab_map_dva_cb = Display Visible Area
tab_map_pemr_cb = Preview event move routes
tab_map_cmrp_btn = Clear move route preview

# Windows
# > About
window_about_title_label = About
window_about_luminol_label = About Luminol
window_about_version_text_label = Luminol version {$version}
window_about_description_text_label = Luminol is a FOSS version of the RPG Maker XP editor.
window_about_authors_label = Authors:
    {$authorsArray}
# > Common Event Editor
window_common_events_editing_label = Editing Common Event {$name}
window_common_events_type_none_sv = None
window_common_events_type_autorun_sv = Autorun
window_common_events_type_parallel_sv = Parallel
# > Config
window_config_title_label = Local Luminol Config
window_config_proj_name_label = Project name
window_config_scripts_path_label = Scripts path
window_config_use_ron_cb = Use RON (Rusty Object Notation)
window_config_rgss_ver_label = RGSS Version
window_config_playtest_exe_btn = Playtest Executable
# > Event Editor
window_event_title_label = Event: {$name}, {$id} in Map {$map_id}
window_event_new_page_btn = New page
window_event_copy_page_btn = Copy page
window_event_paste_page_btn = Paste page
window_event_clear_page_btn = Clear page
window_event_tab_configuration_sv = Configuration
window_event_tab_graphic_sv = Graphic
window_event_tab_commands_sv = Commands
window_event_conf_condition_label = Condition
window_event_conf_switch_cb = Switch
window_event_conf_variable_cb = Variable
window_event_conf_or_above_label = or above
window_event_conf_self_switch_cb = Self Switch
window_event_conf_is_on_label = is on
window_event_conf_options_label = Options
window_event_conf_option_move_anim_cb = Move Animation
window_event_conf_option_stop_anim_cb = Stop Animation
window_event_conf_option_direction_fix_cb = Direction Fix
window_event_conf_option_through_cb = Through
window_event_conf_option_aot_cb = Always on Top
window_event_conf_trigger_label = Trigger
window_event_conf_trigger_action_btn_rv = Action Button
window_event_conf_trigger_player_touch_rv = Player Touch
window_event_conf_trigger_event_touch_rv = Event Touch
window_event_conf_trigger_autorun_rv = Autorun
window_event_conf_trigger_parallel_proc_rv = Parallel Process
window_event_graphic_add_image_btn = Add image
# > Graphic Picker
window_graphic_picker_title_label = Graphic Picker
# > Items
window_items_title_label = Editing item {$name}
window_items_change_max_btn = Change maximum...
window_items_user_anim_field = User Animation
window_items_target_anim_field = Target Animation
window_items_menu_se_field = Menu Use SE
window_items_msep_label = Menu Sound Effect Picker
# > Map Picker
window_map_picker_title_label = Map Picker
window_map_picker_root_label = root
# > Egui Inspection
window_egui_inspec_title_label = Egui Inspection
# > Egui Memory
window_egui_memory_title_label = Egui Memory
# > New Project
window_new_proj_my_proj_str = My Project
window_new_proj_name_label = Project Name
window_new_proj_with_git_cb = Initialize with git repository
window_new_proj_git_branch_label = Git Branch
window_new_proj_rgss_runtime_label = RGSS runtime
window_new_proj_with_exe_download_cb = Download latest version of {$variant}
window_new_proj_dl_and_unzipping_label = Downloading & Unzipping {$current}/{$total}
# > Script Editor
window_script_editor_fallback_title_label = Scripts
window_script_editor_title_label = Editing Script {$name}
window_script_editor_insert_btn = Insert
window_script_editor_delete_btn = Delete
window_script_editor_new_str = New Script
# > Sound
window_sound_test_title_label = Sound Test
window_sound_test_play_btn = Play
window_sound_test_stop_btn = Stop
window_sound_test_volume_label = Volume
window_sound_test_pitch_label = Pitch
# > Command Generator
window_commandgen_title_label = Luminol Command Maker
window_commandgen_desc_label = Description for this command
window_commandgen_lumi_label = Lumi help text
window_commandgen_lumi_onhover_label = This text will be shown by lumi if she's enabled
window_commandgen_contcode_label = Cont. Code
window_commandgen_contcode_onhover_label = Luminol will assume that any following commands with this code are a part of this one
window_commandgen_syntax_highlighting_cb = Enable Ruby syntax highlighting
window_commandgen_endcode_label = End Code
window_commandgen_endcode_onhover_label = Luminol will add this command to denote the end of the branch
window_commandgen_him_cb = Hide in menu
window_commandgen_preview_btn = Preview UI
window_commandgen_position_onhover_label = Position of this parameter, when not set it is assumed to be the index of the parameter
window_commandgen_grouped_params_label = Grouped parameters
window_commandgen_grouped_params_onhover_label = This parameter groups together other parameters
window_commandgen_subparams_label = Subparameters
window_commandgen_subparams_onhover_label = This parameter selects one of the following parameters
window_commandgen_description_onhover_label = Description for this parameter
window_commandgen_variants_onhover_label = Variants for the enum
window_commandgen_ui_example_label = [{$code}] {$name} UI Example
# > General
window_untitled_title = Untitled Window

# Modals
# > Switch
modal_switch_title_label = Switch Picker
# > Variable
modal_variable_title_label = Variable Picker

# Top Bar
# > File Menu
topbar_file_section = File
topbar_file_current_proj_label = Current project:
    {$path}
topbar_file_no_proj_open_label = No project open
topbar_file_proj_config_btn = Project Config
topbar_file_close_proj_btn = Close Project
topbar_file_save_proj_btn = Save Project
topbar_file_command_maker_btn = Command Maker
topbar_file_quit_btn = Quit
# > Appearance Menu
topbar_appearance_section = Appearance
topbar_appearance_egui_conf_btn = Egui Settings
topbar_appearance_egui_catppuccin_section = Catppuccin theme
topbar_appearance_code_theme_section = Code Theme
topbar_appearance_code_sample_label = Code sample
topbar_appearance_clt_btn = Clear Loaded Textures
topbar_appearance_clt_onhover_label = You may need to reopen maps/windows for any changes to take effect.
# > Data Menu
topbar_data_section = Data
# > Help Menu
topbar_help_section = Help
topbar_egui_inspection_btn = Egui Inspection
topbar_egui_memory_btn = Egui Memory
topbar_debug_on_hover_tv = Debug on hover
# > Other UI Controls
topbar_playtest_btn = Playtest
topbar_terminal_btn = Terminal
topbar_brush_label = Brush
topbar_egui_settings_label = Egui Settings

# Toast Notifications
toast_error_load_proj = Error loading the project: {$why}
toast_error_starting_game = Error starting game (tried steamshim.exe and then game.exe): {$why}
toast_error_starting_shell = Error starting shell: {$why}
toast_error_displaying_term = Error displaying terminal: {$why}
toast_error_cannot_load_icon = Could not load `{$icon_path}` icon: {$why}
toast_error_reading_icons = Error while reading `Graphics/Icons`: {$why}
toast_error_creating_proj = Failed to create the project: {$why}
toast_error_init_git = Error while initializing the git repository: {$why}
toast_error_downloading_rgss = Error downloading {$variant}: {$why}
toast_error_getting_body_resp = Error getting response body for {$variant}: {$why}
toast_error_read_zip = Failed to read the zip archive for {$variant}: {$why}
toast_error_invalid_file_path = Invalid file path {$file_path}
toast_error_create_dir = Failed to create directory {$file_path}: {$why}
toast_error_reading_file_data = Failed to read file data {$file_path}: {$why}
toast_error_saving_file_data = Failed to save file data {$file_path}: {$why}
toast_error_loading_rxdata = Failed to load {$file_path}: {$why}
toast_info_saving_proj = Saving project...
toast_info_saved_proj = Saved project successfully!
toast_info_opened_proj = Opened project successfully!
toast_info_successful_load = Successfully opened {$projectName}
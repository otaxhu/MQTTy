# Copyright (c) 2025 Oscar Pernia
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.

# Blueprint compilation
blueprint_files = [
  'gtk/help-overlay.blp',
  'ui/main_window.blp',
  'ui/add_conn_card.blp',
  'ui/conn_card.blp',
  'ui/base_card.blp',
  'ui/edit_conn_list_box.blp',
  'ui/key_value_row.blp',
  'ui/publish_view/publish_view.blp',
  'ui/publish_view/publish_view_notebook.blp',
  'ui/publish_view/publish_general_tab.blp',
  'ui/publish_view/publish_body_tab.blp',
  'ui/publish_view/publish_user_props_tab.blp',
  'ui/publish_view/publish_auth_tab.blp',
  'ui/subscriptions_view/subscriptions_view.blp',
  'ui/subscriptions_view/subscriptions_overview.blp',
  'ui/subscriptions_view/subscription_messages.blp',
  'ui/subscriptions_view/subscription_row.blp',
  'ui/subscriptions_view/subscription_dialog.blp',
  'ui/subscriptions_view/connections_sidebar.blp',
  'ui/subscriptions_view/connection_row.blp',
  'ui/subscriptions_view/connection_dialog.blp',
  'ui/pages/base_page.blp',
  'ui/pages/all_conn_page.blp',
  'ui/pages/add_conn_page.blp',
  'ui/pages/panel_page.blp',
]

blueprint_targets = []
foreach bl_file : blueprint_files
  blueprint_targets += bl_file.replace('.blp', '.ui').split('/')[-1]
endforeach

blueprints = custom_target('blueprints',
  input: files(blueprint_files),
  output: blueprint_targets,
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTDIR@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)

# Sass compilation

# Sass program, with some fallback values, this are defined in subprojects/sass-*.wrap
if host_machine.system() == 'windows'
  sass = find_program('sass', 'sass-win32-x86_64')
else
  sass = find_program('sass', 'sass-linux-x86_64')
endif

sass_files = [
  'styles/style.scss',
  'styles/indicator.scss',
]

css = custom_target('css',
  input: files(sass_files),
  output: 'style.css',
  command: [sass, '--no-source-map', '--style=compressed',  '@INPUT0@', '@OUTPUT@']
)

# Resources
resources = gnome.compile_resources(
  'MQTTy',
  '@0@.gresource.xml'.format(base_id),
  dependencies: [blueprints, css],
  gresource_bundle: true,
  source_dir: meson.current_build_dir(),
  install: true,
  install_dir: pkgdatadir,
)

# Desktop file
desktop_conf = configuration_data()
desktop_conf.set('icon', application_id)
desktop_file = i18n.merge_file(
  type: 'desktop',
  input: configure_file(
    input: '@0@.desktop.in.in'.format(base_id),
    output: '@BASENAME@',
    configuration: desktop_conf
  ),
  output: '@0@.desktop'.format(application_id),
  po_dir: podir,
  install: true,
  install_dir: datadir / 'applications'
)
# Validate Desktop file
if desktop_file_validate.found()
  test(
    'validate-desktop',
    desktop_file_validate,
    args: [
      desktop_file.full_path()
    ],
    depends: desktop_file,
  )
endif

# Appdata
appdata_conf = configuration_data()
appdata_conf.set('app-id', application_id)
appdata_conf.set('gettext-package', gettext_package)
appdata_file = i18n.merge_file(
  input: configure_file(
    input: '@0@.metainfo.xml.in.in'.format(base_id),
    output: '@BASENAME@',
    configuration: appdata_conf
  ),
  output: '@0@.metainfo.xml'.format(application_id),
  po_dir: podir,
  install: true,
  install_dir: datadir / 'metainfo'
)
# Validate Appdata
if appstreamcli.found()
  test(
    'validate-appdata', appstreamcli,
    args: [
      'validate', '--no-net', '--explain', appdata_file.full_path()
    ],
    depends: appdata_file,
  )
endif

# GSchema
gschema_conf = configuration_data()
gschema_conf.set('app-id', application_id)
gschema_conf.set('gettext-package', gettext_package)
configure_file(
  input: '@0@.gschema.xml.in'.format(base_id),
  output: '@0@.gschema.xml'.format(application_id),
  configuration: gschema_conf,
  install: true,
  install_dir: datadir / 'glib-2.0' / 'schemas'
)

# Validate GSchema
test(
  'validate-gschema', glib_compile_schemas,
  args: [
    '--strict', '--dry-run', meson.current_build_dir()
  ],
)

# Icons
install_data(
  '@0@.svg'.format('icons' / application_id),
  install_dir: iconsdir / 'hicolor' / 'scalable' / 'apps'
)
install_data(
  '@0@-symbolic.svg'.format('icons' / base_id),
  install_dir: iconsdir / 'hicolor' / 'symbolic' / 'apps',
  rename: '@0@-symbolic.svg'.format(application_id)
)

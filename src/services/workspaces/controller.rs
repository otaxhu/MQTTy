// Copyright (c) 2025 Oscar Pernia
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::path::PathBuf;

use adw::prelude::*;
use gettextrs::gettext;
use gtk::{gio, glib};

use crate::application::MQTTyApplication;

use super::MQTTyWorkspaceModel;

const RECENT_WORKSPACE_FILE: &'static str = "recent-workspace-file";

#[derive(Debug, thiserror::Error)]
pub enum MQTTyWorkspaceError {
    #[error("Glib returned an error: {0}")]
    GlibError(#[from] glib::Error),
    #[error("TOML serialization error: {0}")]
    TomlSeError(#[from] toml::ser::Error),
    #[error("TOML deserialization error: {0}")]
    TomlDeError(#[from] toml::de::Error),
    #[error("User cancelled a necessary action")]
    UserCancelled,
    #[error("Unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, MQTTyWorkspaceError>;

// No fields, by definition should be stateless.
pub struct MQTTyWorkspacesController;

/// Returned FileDialog is missing title, accept_label and initial_name (for save dialogs),
/// it has every other prop to open/save MQTTy Workspace files.
fn new_file_dialog() -> gtk::FileDialog {
    let filters = gio::ListStore::new::<gtk::FileFilter>();
    let filter = gtk::FileFilter::new();
    filters.append(&filter);
    filter.add_suffix("mqtty.toml");

    // Translators: Used in file dialogs as the filter name for a Workspace file.
    let filter_name = gettext("MQTTy Workspace");
    let filter_name = if cfg!(target_os = "windows") {
        // In windows, the pattern is already shown as part of the dialog,
        // there is not need to add it.
        filter_name
    } else {
        // In Linux (GNOME), the pattern is not shown anywhere in the dialog,
        // we add it here artificially to improve UX.
        format!("{filter_name} (*.mqtty.toml)")
    };
    filter.set_name(Some(&filter_name));

    gtk::FileDialog::builder()
        .filters(&filters)
        .modal(true)
        .build()
}

/// Gets the last opened workspace file, Some(...) if it's set.
fn get_recent_workspace_file() -> Option<gio::File> {
    let app = MQTTyApplication::get_singleton();
    let settings = app.settings();
    settings
        .get::<Option<PathBuf>>(RECENT_WORKSPACE_FILE)
        .map(|p| gio::File::for_path(p))
}

fn set_recent_workspace_file(file: &gio::File) {
    let app = MQTTyApplication::get_singleton();
    let settings = app.settings();
    let _ = settings.set(RECENT_WORKSPACE_FILE, file.path());
}

impl MQTTyWorkspacesController {
    pub fn new() -> Self {
        Self
    }

    fn load_workspace_file(&self, file: &gio::File) -> Result<MQTTyWorkspaceModel> {
        let (contents, _) = file.load_contents(gio::Cancellable::NONE)?;
        Ok(toml::from_slice(&contents)?)
    }

    /// Same as Self::load_workspace, but it opens a dialog to let the user choose a file.
    ///
    /// If the user cancels the dialog, it returns Ok(None), check Self::load_workspace
    /// for more info about the return of this.
    pub async fn load_workspace_from_dialog(&self) -> Result<Option<MQTTyWorkspaceModel>> {
        let app = MQTTyApplication::get_singleton();
        let dialog = new_file_dialog();
        dialog.set_title(&gettext("Open MQTTy workspace"));
        dialog.set_accept_label(Some(&gettext("Open")));
        dialog.set_initial_folder(
            get_recent_workspace_file()
                .and_then(|f| f.parent())
                .as_ref(),
        );
        let file = match dialog.open_future(app.active_window().as_ref()).await {
            Ok(f) => Ok(f),
            Err(e) => match e.kind::<gtk::DialogError>() {
                Some(gtk::DialogError::Cancelled | gtk::DialogError::Dismissed) => return Ok(None),
                _ => Err(e),
            },
        }?;

        let workspace = self.load_workspace_file(&file)?;

        // Always assume that the file path is new
        set_recent_workspace_file(&file);

        Ok(Some(workspace))
    }

    /// Loads the last workspace that's stored in the app's settings, if the
    /// setting is empty, it returns Ok(None).
    ///
    /// In the case where a workspace is found, it will try to open it and parse
    /// it as a TOML file and return Ok(Some(...)), or Err(...) if it fails to
    /// open it, parse it, etc.
    pub fn load_last_workspace(&self) -> Result<Option<MQTTyWorkspaceModel>> {
        let file = match get_recent_workspace_file() {
            Some(f) => f,
            None => return Ok(None),
        };

        self.load_workspace_file(&file).map(|w| Some(w))
    }

    /// Saves the workspace.
    pub async fn save_workspace(
        &self,
        overwrite_last_workspace: bool,
        workspace: &MQTTyWorkspaceModel,
    ) -> Result<()> {
        let last_file = get_recent_workspace_file();

        let file = if overwrite_last_workspace {
            let Some(f) = last_file else {
                return Err(MQTTyWorkspaceError::Unknown);
            };
            f
        } else {
            let app = MQTTyApplication::get_singleton();
            let dialog = new_file_dialog();
            dialog.set_accept_label(Some(&gettext("Save")));
            dialog.set_title(&gettext("Save MQTTy workspace"));
            dialog.set_initial_folder(last_file.and_then(|f| f.parent()).as_ref());
            dialog.set_initial_name(Some(&format!("{}.mqtty.toml", gettext("Workspace"))));
            let file = dialog
                .save_future(app.active_window().as_ref())
                .await
                .map_err(|e| match e.kind::<gtk::DialogError>() {
                    Some(gtk::DialogError::Cancelled | gtk::DialogError::Dismissed) => {
                        MQTTyWorkspaceError::UserCancelled
                    }
                    _ => MQTTyWorkspaceError::GlibError(e),
                })?;

            file
        };

        let workspace_encoded = toml::to_string(workspace)?;

        file.replace_contents(
            workspace_encoded.as_bytes(),
            None,
            false,
            gio::FileCreateFlags::NONE,
            gio::Cancellable::NONE,
        )?;

        if !overwrite_last_workspace {
            set_recent_workspace_file(&file);
        }

        Ok(())
    }
}

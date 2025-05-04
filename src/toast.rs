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

use adw::prelude::*;
use gtk::glib;

pub struct MQTTyToastBuilder {
    inner: adw::builders::ToastBuilder,
    title: Option<String>,
    icon: Option<gtk::Image>,
}

impl MQTTyToastBuilder {
    pub fn new() -> Self {
        Self {
            inner: adw::Toast::builder(),
            title: None,
            icon: None,
        }
    }

    pub fn action_name(self, action_name: impl Into<glib::GString>) -> Self {
        Self {
            inner: self.inner.action_name(action_name),
            ..self
        }
    }

    pub fn action_target(self, action_target: &glib::Variant) -> Self {
        Self {
            inner: self.inner.action_target(action_target),
            ..self
        }
    }

    pub fn button_label(self, button_label: impl Into<glib::GString>) -> Self {
        Self {
            inner: self.inner.button_label(button_label),
            ..self
        }
    }

    pub fn priority(self, priority: adw::ToastPriority) -> Self {
        Self {
            inner: self.inner.priority(priority),
            ..self
        }
    }

    pub fn timeout(self, timeout: u32) -> Self {
        Self {
            inner: self.inner.timeout(timeout),
            ..self
        }
    }

    pub fn use_markup(self, use_markup: bool) -> Self {
        Self {
            inner: self.inner.use_markup(use_markup),
            ..self
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn icon(mut self, icon: &gtk::Image) -> Self {
        self.icon = Some(icon.clone());
        self
    }

    pub fn build(self) -> adw::Toast {
        if self.title.is_some() || self.icon.is_some() {
            let b = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            b.set_valign(gtk::Align::Center);
            if let Some(icon) = self.icon {
                b.append(&icon);
            }
            if let Some(title) = self.title {
                let label = gtk::Label::new(Some(&title));
                label.add_css_class("heading");
                b.append(&label);
            }
            self.inner.custom_title(&b).build()
        } else {
            self.inner.build()
        }
    }
}

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

use adw::subclass::prelude::*;
use gtk::glib;
use sourceview::prelude::*;
use sourceview::subclass::prelude::*;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct MQTTySourceView {}

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySourceView {
        const NAME: &'static str = "MQTTySourceView";

        type Type = super::MQTTySourceView;

        type ParentType = sourceview::View;
    }

    impl ObjectImpl for MQTTySourceView {
        fn constructed(&self) {
            self.parent_constructed();

            self.init_style();
        }
    }
    impl WidgetImpl for MQTTySourceView {}
    impl TextViewImpl for MQTTySourceView {}
    impl ViewImpl for MQTTySourceView {}

    impl MQTTySourceView {
        fn update_style(&self) {
            let obj = self.obj();

            let dark_mode = adw::StyleManager::default().is_dark();

            let color_theme = if dark_mode { "Adwaita-dark" } else { "Adwaita" };

            let scheme_theme = sourceview::StyleSchemeManager::default().scheme(color_theme);

            let buffer = obj.buffer().downcast::<sourceview::Buffer>().unwrap();

            buffer.set_style_scheme(scheme_theme.as_ref());
            buffer.set_highlight_syntax(true);
        }

        fn init_style(&self) {
            self.update_style();

            adw::StyleManager::default().connect_dark_notify(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |_| {
                    this.update_style();
                }
            ));

            let obj = self.obj();
            obj.connect_buffer_notify(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |_| {
                    this.update_style();
                }
            ));
        }
    }
}

glib::wrapper! {
    pub struct MQTTySourceView(ObjectSubclass<imp::MQTTySourceView>)
        @extends gtk::TextView, gtk::Widget, sourceview::View,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Scrollable;
}

use gtk4::prelude::*;
use gtk4::*;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct MainMenu {
    pub popover : PopoverMenu,
    pub action_new : gio::SimpleAction,
    pub action_open : gio::SimpleAction
}

impl MainMenu {

    pub fn build() -> Self {
        let menu = gio::Menu::new();
        menu.append(Some("New"), Some("win.new_file"));
        menu.append(Some("Open"), Some("win.open_file"));
        let popover = PopoverMenu::from_model(Some(&menu));

        let action_new = gio::SimpleAction::new("new_file", None);
        let action_open = gio::SimpleAction::new("open_file", None);

        Self { popover, action_new, action_open }
    }
}
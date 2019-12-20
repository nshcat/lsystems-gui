use imgui::*;

/// Draw a help marker of the form (?) which shows a help text when hovered over.
pub fn help_marker(ui: &Ui, text: &ImStr) {
    ui.text_disabled(im_str!("(?)"));
    if ui.is_item_hovered() {
        ui.tooltip_text(text);
    }
}


/// Enumeration describing the different button types supported by the popup function.
#[derive(Clone, Copy)]
pub enum PopupButton {
    Ok,
    Cancel,
    Yes,
    No
}

impl PopupButton {
    /// Retrieve button label associated with given popup button type.
    pub fn button_label(&self) -> &ImStr {
        match self {
            Self::Ok => im_str!("Ok"),
            Self::Cancel => im_str!("Cancel"),
            Self::Yes => im_str!("Yes"),
            Self::No => im_str!("No")
        }
    }
}

/// Show a pop up dialog with given buttons. Returns the type of clicked button.
pub fn show_popup(ui: &Ui, title: &ImStr, text: &ImStr, buttons: &[PopupButton]) -> Option<PopupButton> {
    let mut result: Option<PopupButton> = None;
    


    ui.popup_modal(title)
        .always_auto_resize(true)
        .always_use_window_padding(true)
        .build(|| {
        ui.text(text);
        ui.separator();
        let style = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0]));

        for button in buttons {
            if ui.button(button.button_label(), [80.0, 0.0]) {
                ui.close_current_popup();
                result = Some(*button);
            }

            ui.same_line(0.0);
        }

        style.pop(ui);
    });

    return result;
}
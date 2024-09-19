use std::{cell::RefCell, rc::Rc, usize};

use gtk4::{glib::MainContext, prelude::{BoxExt, GestureExt, GridExt, WidgetExt}};

use crate::{bar_widget::BarWidget, unix_sockets::ChannelsData};

const EVENTS_LIST: &[&str] = &[
    "hyprland/workspace",
];

//--------------------------------------------------------------------------------------------------------------------------------
//----------------------------------------------------------[ Widget ]------------------------------------------------------------
//--------------------------------------------------------------------------------------------------------------------------------

pub struct HyprlandWorkspacesWidget {
    container:      gtk4::Grid,
    buttons:        Rc<Vec<gtk4::Label>>,
    last_workspace: Rc<RefCell<usize>>,
}

impl BarWidget for HyprlandWorkspacesWidget {
    fn bind_widget(&self, container: &gtk4::Box) {
        container.append(&self.container);
    }

    fn events_list(&self) -> &'static[&'static str] {
        EVENTS_LIST
    }

    fn bind_channels(&self, mut channels_data: ChannelsData) {
        for button_index in 0..self.buttons.len() {
            let gesture = gtk4::GestureClick::new();

            let call_tx = channels_data.call_tx.clone();

            gesture.connect_released(move |gesture, _, _, _| {
                gesture.set_state(gtk4::EventSequenceState::Claimed);
    
                let _ = call_tx.send(format!("hyprland/setWorkspace/{}", button_index + 1));
            });
    
            self.buttons[button_index].add_controller(gesture);
        }

        let buttons        = self.buttons.clone();
        let last_workspace = self.last_workspace.clone();

        

        MainContext::default().spawn_local(async move {
            while let Ok(event) = channels_data.event_rx.recv().await {
                if event.name != EVENTS_LIST[0] {
                    continue;
                }

                let workspace_id = get_workspace_id(&event.value, buttons.len());
                buttons[*last_workspace.borrow() - 1].remove_css_class("hyprland-workspaces-widget-picked");

                if workspace_id.is_none() {
                    continue;
                }

                buttons[(workspace_id.unwrap() - 1) as usize].add_css_class("hyprland-workspaces-widget-picked");

                *last_workspace.borrow_mut() = workspace_id.unwrap();
            }
        });
    }
}

fn get_workspace_id(value: &str, max_id: usize) -> Option<usize> {
    let workspace_id = value.parse::<i32>();

    if workspace_id.is_err() {
        return None;
    }

    let workspace_id_unwrapped = workspace_id.unwrap();

    if workspace_id_unwrapped < 1 || workspace_id_unwrapped as usize > max_id {
        return None;
    }

    Some(workspace_id_unwrapped as usize)
}

impl HyprlandWorkspacesWidget {
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut buttons: Vec<gtk4::Label> = Vec::new();
        let container = HyprlandWorkspacesWidget::create_container();
        
        for row in 0..rows {
            for col in 0..cols {
                HyprlandWorkspacesWidget::create_button(&container, &mut buttons, row, col, cols);
            }
        }

        let widget = Self { 
            container,
            buttons:        Rc::new(buttons),
            last_workspace: Rc::new(RefCell::new(1)) 
        };

        widget
    }

    fn create_button(container: &gtk4::Grid, buttons: &mut Vec<gtk4::Label>, row: usize, col: usize, cols_count: usize) {
        let button_number = row * cols_count + col;
    
        buttons.push(gtk4::Label::new(Some(format!("{}", button_number + 1).as_str())));
    
        buttons[button_number].add_css_class("hyprland-workspaces-widget-button");
        buttons[button_number].add_css_class("hyprland-workspaces-widget");
    
        container.attach(&buttons[button_number], col as i32, row as i32,  1, 1);
    }

    fn create_container() -> gtk4::Grid{
        let container = gtk4::Grid::builder()
            .row_homogeneous(true)
            .row_spacing(6)
            .column_spacing(2)
            .build();

        container.add_css_class("hyprland-workspaces-widget-container");
        container.add_css_class("hyprland-workspaces-widget");

        container
    }
}

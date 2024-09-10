use gtk4::prelude::BoxExt;

use crate::ChannelsData;

pub trait BarWidget {
    fn update_widget(&mut self);
    fn bind_widget(&self, container: &impl BoxExt, channelsData: ChannelsData);
}

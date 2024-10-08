use crate::unix_sockets::ChannelsData;

pub trait BarWidget {
    fn bind_widget  (&self, container: &gtk4::Box);
    fn bind_channels(&self, channels_data: ChannelsData);

    // TODO use just Vec<String>?
    fn events_list(&self) -> &'static[&'static str];
}

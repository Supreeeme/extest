use std::{env, os::unix::net::UnixStream, path::PathBuf};

use wayland_client::{
    backend::Backend,
    delegate_noop,
    protocol::{wl_output, wl_registry},
    Connection, Dispatch, Proxy, QueueHandle,
};
use wayland_protocols::xdg::xdg_output::zv1::client::{
    zxdg_output_manager_v1::ZxdgOutputManagerV1, zxdg_output_v1,
};

#[derive(Copy, Clone, Default)]
pub struct DisplaySize {
    pub width: i32,
    pub height: i32,
}

#[derive(Default)]
struct OutputData {
    wl_outputs: Vec<wl_output::WlOutput>,
    output_man: Option<ZxdgOutputManagerV1>,
    max_size: DisplaySize,
}

fn connect_to_wayland() -> Connection {
    if let Some(socket_name) = env::var_os("EXTEST_DISPLAY").map(Into::<PathBuf>::into) {
        let socket_path = if socket_name.is_absolute() {
            socket_name.clone()
        } else {
            let mut socket_path = env::var_os("XDG_RUNTIME_DIR")
                .map(Into::<PathBuf>::into)
                .unwrap();
            if !socket_path.is_absolute() {
                panic!("XDG_RUNTIME_DIR is not absolute");
            }
            socket_path.push(&socket_name);
            socket_path
        };
        let stream = UnixStream::connect(socket_path)
            .map_err(|_| {
                panic!(
                    "Failed to connect to Wayland socket: {}",
                    socket_name.display()
                )
            })
            .unwrap();
        let backend = Backend::connect(stream)
            .map_err(|_| panic!("Failed to connect to Wayland backend"))
            .unwrap();
        Connection::from_backend(backend)
    } else {
        // Default to standard Wayland connection if EXTEST_DISPLAY is not set
        Connection::connect_to_env().unwrap()
    }
}

/// Returns the largest dimensions of the detected monitors
#[must_use]
pub fn get_axes_range() -> DisplaySize {
    let conn = connect_to_wayland();
    let display = conn.display();
    let mut queue = conn.new_event_queue();
    let handle = queue.handle();
    display.get_registry(&handle, ());
    let mut data = OutputData::default();
    queue.roundtrip(&mut data).unwrap();

    for output in data.wl_outputs.iter() {
        data.output_man
            .as_ref()
            .unwrap()
            .get_xdg_output(output, &handle, ());
    }
    queue.roundtrip(&mut data).unwrap();
    data.max_size
}

impl Dispatch<zxdg_output_v1::ZxdgOutputV1, ()> for OutputData {
    fn event(
        state: &mut Self,
        _: &zxdg_output_v1::ZxdgOutputV1,
        event: zxdg_output_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<OutputData>,
    ) {
        if let zxdg_output_v1::Event::LogicalSize { width, height } = event {
            println!("detected display with width {width} and height {height}");
            state.max_size.width = i32::max(width, state.max_size.width);
            state.max_size.height = i32::max(height, state.max_size.height);
        }
    }
}

delegate_noop!(OutputData: ignore wl_output::WlOutput);
delegate_noop!(OutputData: ZxdgOutputManagerV1);
impl Dispatch<wl_registry::WlRegistry, ()> for OutputData {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        handle: &QueueHandle<OutputData>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == wl_output::WlOutput::interface().name {
                let output: wl_output::WlOutput = registry.bind(name, version, handle, ());
                state.wl_outputs.push(output);
            } else if interface == ZxdgOutputManagerV1::interface().name && version >= 3 {
                let man: ZxdgOutputManagerV1 = registry.bind(name, version, handle, ());
                state.output_man = Some(man);
            }
        }
    }
}

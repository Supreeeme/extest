use wayland_client::{
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
    pub height: i32
}

#[derive(Default)]
struct OutputData {
    wl_outputs: Vec<wl_output::WlOutput>,
    output_man: Option<ZxdgOutputManagerV1>,
    max_size: DisplaySize
}

/// Returns the largest dimensions of the detected monitors
pub fn get_axes_range() -> Result<DisplaySize, String> {
    let conn = Connection::connect_to_env()
        .map_err(|e| format!("could not connect to the Wayland compositor: {e}"))?;
    let display = conn.display();
    let mut queue = conn.new_event_queue();
    let handle = queue.handle();
    display.get_registry(&handle, ());
    let mut data = OutputData::default();
    queue
        .roundtrip(&mut data)
        .map_err(|e| format!("Wayland roundtrip failed: {e}"))?;

    let output_man = data
        .output_man
        .clone()
        .ok_or("compositor does not support xdg-output v3")?;
    for output in data.wl_outputs.iter() {
        output_man.get_xdg_output(output, &handle, ());
    }
    queue
        .roundtrip(&mut data)
        .map_err(|e| format!("Wayland roundtrip failed: {e}"))?;
    if data.max_size.width <= 0 || data.max_size.height <= 0 {
        return Err("no outputs with a size were reported".into());
    }
    Ok(data.max_size)
}

#[cfg(test)]
mod tests {
    use super::get_axes_range;

    #[test]
    fn missing_compositor_is_an_error_not_a_panic() {
        // Steam calls into this from XTest functions, where a panic aborts the whole client.
        std::env::remove_var("WAYLAND_SOCKET");
        std::env::set_var("WAYLAND_DISPLAY", "extest-test-no-such-display");
        assert!(get_axes_range().is_err());
    }
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

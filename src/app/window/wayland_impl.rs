use std::os::fd::AsFd;
use wayland_client::{
    protocol::wl_buffer, protocol::wl_compositor, protocol::wl_seat, protocol::wl_registry,
    protocol::wl_pointer, protocol::wl_keyboard,
    protocol::wl_shm, protocol::wl_shm_pool, protocol::wl_surface, Connection, Dispatch, Proxy,
    QueueHandle,
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

#[derive(Default)]
struct WindowState {
    width: i32,
    height: i32,
    max_width: i32,
    max_height: i32,
    latest_events: Vec<super::Event>,
    compositor: Option<wl_compositor::WlCompositor>,
    base_surface: Option<wl_surface::WlSurface>,
    xdg_wm_base: Option<xdg_wm_base::XdgWmBase>,
    xdg_surface: Option<xdg_surface::XdgSurface>,
    configured_xdg_surface: bool,
    shm: Option<wl_shm::WlShm>,
    buffer: Option<wl_buffer::WlBuffer>,
    file: Option<std::fs::File>,
}

impl WindowState {
    fn init_xdg_surface(&mut self, qh: &QueueHandle<Self>) {
        let xdg_wm_base = self
            .xdg_wm_base
            .as_ref()
            .expect("missing xdg_wm_base binding");
        let base_surface = self
            .base_surface
            .as_ref()
            .expect("missing wl_surface binding");

        let xdg_surface = xdg_wm_base.get_xdg_surface(base_surface, qh, ());

        let toplevel = xdg_surface.get_toplevel(qh, ());
        toplevel.set_title("A pantastic window!".into());

        base_surface.commit();

        self.xdg_surface = Some(xdg_surface);
    }

    fn allocate_shared_buffer(&mut self, qh: &QueueHandle<Self>) {
        let shm = self.shm.as_ref().unwrap();
        let file = tempfile::tempfile().unwrap();
        file.set_len((self.width * self.height * 4) as u64)
            .expect("unable te resize file");

        let shm_pool = shm.create_pool(file.as_fd(), self.width * self.height * 4, qh, ());

        let buffer = shm_pool.create_buffer(
            0,
            self.width,
            self.height,
            self.width * 4,
            wl_shm::Format::Xrgb8888,
            qh,
            (),
        );

        self.file = Some(file);
        self.buffer = Some(buffer);
    }

    fn draw_frame(&mut self, frame: &[u8]) {
        use std::io::{Seek, Write};
        let mut file = self.file.as_ref().unwrap();
        file.rewind().unwrap();
        let mut buf = std::io::BufWriter::new(file);
        buf.write_all(frame).unwrap();
        buf.flush().unwrap();
    }
}

impl Dispatch<wl_shm::WlShm, ()> for WindowState {
    fn event(
        _: &mut Self,
        _shm: &wl_shm::WlShm,
        _event: wl_shm::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WindowState>,
    ) {
        //println!("Recived {} event: {:#?}", wl_shm::WlShm::interface().name, event);
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for WindowState {
    fn event(
        _: &mut Self,
        _: &wl_surface::WlSurface,
        event: wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WindowState>,
    ) {
        println!(
            "Recived {} event: {:#?}",
            wl_surface::WlSurface::interface().name,
            event
        );
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for WindowState {
    fn event(
        _: &mut Self,
        _: &wl_shm_pool::WlShmPool,
        event: wl_shm_pool::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WindowState>,
    ) {
        println!(
            "Recived {} event: {:#?}",
            wl_shm_pool::WlShmPool::interface().name,
            event
        );
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for WindowState {
    fn event(
        _: &mut Self,
        _: &wl_buffer::WlBuffer,
        _event: wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WindowState>,
    ) {
        /*
        println!(
            "Recived {} event: {:#?}",
            wl_buffer::WlBuffer::interface().name,
            event
        );
        */
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for WindowState {
    fn event(
        _: &mut Self,
        _: &wl_compositor::WlCompositor,
        event: wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WindowState>,
    ) {
        println!(
            "Recived {} event: {:#?}",
            wl_compositor::WlCompositor::interface().name,
            event
        );
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for WindowState {
    fn event(
        state: &mut Self,
        _: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WindowState>,
    ) {
        println!(
            "Recived {} event: {:#?}",
            xdg_toplevel::XdgToplevel::interface().name,
            event
        );

        match event {
            // Current window bounds (non-maximized)
            xdg_toplevel::Event::ConfigureBounds { width, height } => {
                state.max_width = width;
                state.max_height = height;
            }
            xdg_toplevel::Event::Close => {
                println!("Close button clicked!");
            }
            _ => {}
        }
    }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for WindowState {
    fn event(
        state: &mut Self,
        xdg_surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WindowState>,
    ) {
        println!(
            "Recived {} event: {:#?}",
            xdg_surface::XdgSurface::interface().name,
            event
        );

        if let xdg_surface::Event::Configure { serial } = event {
            xdg_surface.ack_configure(serial);
            state.configured_xdg_surface = true;

            if let Some(ref buffer) = state.buffer {
                let surface = state.base_surface.as_ref().unwrap();
                surface.attach(Some(buffer), 0, 0);
                surface.commit();
            }
        }
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for WindowState {
    fn event(
        _: &mut Self,
        xdg_wm_base: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WindowState>,
    ) {
        println!(
            "Recived {} event: {:#?}",
            xdg_wm_base::XdgWmBase::interface().name,
            event
        );
        if let xdg_wm_base::Event::Ping { serial } = event {
            xdg_wm_base.pong(serial);
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for WindowState {
    fn event(
        _: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        println!(
            "Recived {} event: {:#?}",
            wl_seat::WlSeat::interface().name,
            event
        );
        if let wl_seat::Event::Capabilities { capabilities: wayland_client::WEnum::Value(capabilities) } = event {
            if capabilities.contains(wl_seat::Capability::Keyboard) {
                seat.get_keyboard(qh, ());
            }

            if capabilities.contains(wl_seat::Capability::Pointer) {
                seat.get_pointer(qh, ());
            }
        }
    }
}

impl Dispatch<wl_pointer::WlPointer, ()> for WindowState {
    fn event(
        _state: &mut Self,
        _pointer: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        println!(
            "Recived {} event: {:#?}",
            wl_pointer::WlPointer::interface().name,
            event
        );

        /*
        if let wl_pointer::Event::Enter { serial, surface, surface_x, surface_y } = event {
        // TODO: load cursor theme and attach its buffer to a surface
        //https://docs.rs/wayland-cursor/latest/wayland_cursor/
            pointer.set_cursor(serial, Some(&surface_cursor_theme), surface_x as i32, surface_y as i32);
        }
        */
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for WindowState {
    fn event(
        _state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        println!(
            "Recived {} event: {:#?}",
            wl_keyboard::WlKeyboard::interface().name,
            event
        );
        if let wl_keyboard::Event::Key { key, .. } = event {
            if key == 1 {
                // ESC key
                //state.running = false;
            }
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for WindowState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<WindowState>,
    ) {
        println!(
            "Global: Recived {} event: {:#?}",
            wl_registry::WlRegistry::interface().name,
            event
        );
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_compositor" => {
                    let compositor =
                        registry.bind::<wl_compositor::WlCompositor, _, _>(name, version, qh, ());
                    let surface = compositor.create_surface(qh, ());

                    state.compositor = Some(compositor);
                    state.base_surface = Some(surface);
                }
                "xdg_wm_base" => {
                    let xdg_wm_base =
                        registry.bind::<xdg_wm_base::XdgWmBase, _, _>(name, version, qh, ());

                    state.xdg_wm_base = Some(xdg_wm_base);
                }
                "wl_shm" => {
                    let shm = registry.bind::<wl_shm::WlShm, _, _>(name, version, qh, ());

                    state.shm = Some(shm);
                }
                "wl_seat" => {
                    registry.bind::<wl_seat::WlSeat, _, _>(name, version, qh, ());
                }
                _ => {}
            }
        }
    }
}

pub struct WaylandWindow {
    state: WindowState,
    event_queue: wayland_client::EventQueue<WindowState>,
}

impl super::Window for WaylandWindow {
    fn new(width: usize, height: usize) -> Self {
        let mut state = WindowState {
            width: width as i32,
            height: height as i32,
            ..Default::default()
        };
        let conn = wayland_client::Connection::connect_to_env().unwrap();

        let display = conn.display();

        let mut event_queue = conn.new_event_queue::<WindowState>();
        let qh = event_queue.handle();

        let _registry = display.get_registry(&qh, ());
        event_queue.roundtrip(&mut state).unwrap();

        state.init_xdg_surface(&qh);
        state.allocate_shared_buffer(&qh);

        // TODO: check if this second roundtrip is really necessary
        event_queue.roundtrip(&mut state).unwrap();

        Self { state, event_queue }
    }

    fn handle(&mut self) -> Vec<super::Event> {
        self.event_queue.blocking_dispatch(&mut self.state).unwrap();
        self.state.latest_events.drain(..).collect()
    }
    fn write_frame_from_ptr(&mut self, _src: *const u8, _sz: usize) {}
    fn write_frame_from_slice(&mut self, src: &[u8]) {
        assert_eq!(self.state.width * self.state.height * 4, src.len() as i32);
        self.state.draw_frame(src);

        if self.state.configured_xdg_surface {
            let buffer = self.state.buffer.as_ref().unwrap();
            let surface = self.state.base_surface.as_ref().unwrap();
            surface.attach(Some(buffer), 0, 0);
            surface.damage_buffer(0, 0, self.state.width, self.state.height);
            surface.commit();
        }
    }
    fn get_window_position(&self) -> (i32, i32) {
        (0, 0)
    }
    fn get_screen_dim(&self) -> (usize, usize) {
        // FIXME: Currently we use the dimensions of the maximized window.
        // In order to get the fullscreen dimensions we need to bind de wl_output
        // and handle a Geometry event.
        // See: https://docs.rs/wayland-client/latest/wayland_client/protocol/wl_output/enum.Event.html#variant.Geometry
        (
            self.state.max_width as usize,
            self.state.max_height as usize,
        )
    }
    fn get_window_dim(&self) -> (usize, usize) {
        (self.state.width as usize, self.state.height as usize)
    }
    fn hide_mouse_cursor(&mut self) {}
    fn show_mouse_cursor(&mut self) {}
    fn toggle_fullscreen(&mut self) {}
    fn update_mouse_cursor(&mut self, _cursor: super::MouseCursor) {}
    fn set_mouse_position(&mut self, _x: i32, _y: i32) {}
}

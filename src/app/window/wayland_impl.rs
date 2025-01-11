use std::os::fd::AsFd;
use wayland_client::{
    protocol::wl_buffer, protocol::wl_compositor, protocol::wl_keyboard, protocol::wl_pointer,
    protocol::wl_registry, protocol::wl_seat, protocol::wl_shm, protocol::wl_shm_pool,
    protocol::wl_subcompositor, protocol::wl_surface, Connection, Dispatch, Proxy, QueueHandle,
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

const CURSOR_SIZE: u32 = 24;

#[derive(Default)]
struct WindowState {
    width: i32,
    height: i32,
    max_width: i32,
    max_height: i32,
    latest_events: Vec<super::Event>,
    mouse_cursor: super::MouseCursor,
    mouse_pointer_info: super::MouseInfo,
    cursor_visibility: bool,
    cursor_theme: Option<wayland_cursor::CursorTheme>,
    compositor: Option<wl_compositor::WlCompositor>,
    seat: Option<wl_seat::WlSeat>,
    subcompositor: Option<wl_subcompositor::WlSubcompositor>,
    base_surface: Option<wl_surface::WlSurface>,
    xdg_wm_base: Option<xdg_wm_base::XdgWmBase>,
    xdg_surface: Option<xdg_surface::XdgSurface>,
    xdg_toplevel: Option<xdg_toplevel::XdgToplevel>,
    is_fullscreen: bool,
    configured_xdg_surface: bool,
    shm: Option<wl_shm::WlShm>,
    buffer: Option<wl_buffer::WlBuffer>,
    file: Option<std::fs::File>,
}

impl WindowState {
    // https://docs.rs/smithay-client-toolkit/latest/src/smithay_client_toolkit/seat/pointer/mod.rs.html#599
    fn load_cursor_theme(&mut self, conn: &Connection) {
        use wayland_cursor::CursorTheme;

        let system_theme = linicon::get_system_theme().unwrap_or("default".to_string());

        let shm = self.shm.as_ref().unwrap();
        let cursor_theme =
            CursorTheme::load_or(conn, shm.clone(), system_theme.as_str(), CURSOR_SIZE)
                .expect("Could not load cursor theme");

        self.cursor_theme = Some(cursor_theme);
    }

    fn init_xdg_surface(&mut self, qh: &QueueHandle<Self>) {
        let xdg_wm_base = self.xdg_wm_base.as_ref().unwrap();
        let base_surface = self.base_surface.as_ref().unwrap();

        let xdg_surface = xdg_wm_base.get_xdg_surface(base_surface, qh, ());

        let toplevel = xdg_surface.get_toplevel(qh, ());
        toplevel.set_title("A pantastic window!".into());

        base_surface.commit();

        self.xdg_surface = Some(xdg_surface);
        self.xdg_toplevel = Some(toplevel);
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

pub struct WaylandWindow {
    state: WindowState,
    event_queue: wayland_client::EventQueue<WindowState>,
}

impl super::Window for WaylandWindow {
    fn new(width: usize, height: usize) -> Self {
        let mut state = WindowState {
            width: width as i32,
            height: height as i32,
            cursor_visibility: true,
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
        state.load_cursor_theme(&conn);

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
            //self.frame.draw();
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
    fn hide_mouse_cursor(&mut self) {
        self.state.cursor_visibility = false;
    }
    fn show_mouse_cursor(&mut self) {
        self.state.cursor_visibility = true;
    }
    fn toggle_fullscreen(&mut self) {
        let toplevel = self.state.xdg_toplevel.as_ref().unwrap();
        if self.state.is_fullscreen {
            toplevel.unset_fullscreen();
        } else {
            // FIXME: We need to pass the current (or last??)
            // wl_output which the wl_surface entered
            // https://docs.rs/wayland-client/latest/wayland_client/protocol/wl_surface/enum.Event.html#variant.Enter
            toplevel.set_fullscreen(None);
        }

        self.state.is_fullscreen = !self.state.is_fullscreen;
    }
    fn update_mouse_cursor(&mut self, cursor: super::MouseCursor) {
        self.state.mouse_cursor = cursor;
    }
    fn set_mouse_position(&mut self, _x: i32, _y: i32) {}
}

impl From<super::MouseCursor> for &str {
    fn from(value: super::MouseCursor) -> Self {
        match value {
            super::MouseCursor::Arrow => "arrow",
            super::MouseCursor::TextInput => "xterm",
            super::MouseCursor::ResizeAll => "fleur",
            super::MouseCursor::ResizeNS => "sb_v_double_arrow",
            super::MouseCursor::ResizeEW => "sb_h_double_arrow",
            super::MouseCursor::ResizeNESW => "bottom_left_corner",
            super::MouseCursor::ResizeNWSE => "bottom_right_corner",
            super::MouseCursor::Hand => "hand1",
            super::MouseCursor::NotAllowed => "circle",
        }
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

impl Dispatch<wl_subcompositor::WlSubcompositor, ()> for WindowState {
    fn event(
        _: &mut Self,
        _: &wl_subcompositor::WlSubcompositor,
        event: wl_subcompositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WindowState>,
    ) {
        println!(
            "Recived {} event: {:#?}",
            wl_subcompositor::WlSubcompositor::interface().name,
            event
        );
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

        // https://docs.rs/wayland-protocols/latest/wayland_protocols/xdg/shell/client/xdg_toplevel/enum.Event.html
        match event {
            // 'Suggest' a surface change
            xdg_toplevel::Event::Configure { width, height, .. } => {
                if width != 0 && height != 0 {
                    state.width = width;
                    state.height = height;
                }
            }
            // Current window bounds (non-fullscreen)
            xdg_toplevel::Event::ConfigureBounds { width, height } => {
                if width != 0 && height != 0 {
                    state.max_width = width;
                    state.max_height = height;
                } else {
                    // TODO: Unknown bounds
                }
            }
            xdg_toplevel::Event::Close => {
                // TODO;
                println!("Compositor wants us to stop!");
                unimplemented!();
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
        if let wl_seat::Event::Capabilities {
            capabilities: wayland_client::WEnum::Value(capabilities),
        } = event
        {
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
        win_state: &mut Self,
        pointer: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        println!(
            "Recived {} event: {:#?}",
            wl_pointer::WlPointer::interface().name,
            event
        );

        match event {
            wl_pointer::Event::Button { button, state, .. } => {
                let mouse_button = match button {
                    input_event_codes::BTN_LEFT!() => super::Button::MouseLeft,
                    input_event_codes::BTN_MIDDLE!() => super::Button::MouseMiddle,
                    input_event_codes::BTN_RIGHT!() => super::Button::MouseRight,
                    input_event_codes::BTN_GEAR_UP!() => super::Button::WheelUp,
                    input_event_codes::BTN_GEAR_DOWN!() => super::Button::WheelDown,
                    _ => super::Button::Unknown,
                };

                let button_event = match state.into_result().unwrap() {
                    wl_pointer::ButtonState::Released => super::Event::ButtonRelease(mouse_button),
                    wl_pointer::ButtonState::Pressed => {
                        /*  TODO: In order to move the window we need to verify
                         *  if the ButtonPress occurred over the title bar of the
                         *  window frame.
                         *
                         *  let seat = win_state.seat.as_ref().unwrap();
                         *  let toplevel = win_state.xdg_toplevel.as_ref().unwrap();
                         *  toplevel._move(seat, serial);
                         */

                        super::Event::ButtonPress(mouse_button)
                    }
                    _ => super::Event::Empty,
                };

                win_state.latest_events.push(button_event);
            }

            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                // TODO: Enable cursor animations

                let dx = surface_x as i32 - win_state.mouse_pointer_info.x;
                let dy = surface_y as i32 - win_state.mouse_pointer_info.y;

                let pointer_info = super::MouseInfo {
                    x: surface_x as i32,
                    y: surface_y as i32,
                    dx,
                    dy,
                };

                win_state.mouse_pointer_info = pointer_info.clone();

                win_state
                    .latest_events
                    .push(super::Event::MouseMotion(pointer_info));
            }

            wl_pointer::Event::Enter { serial, .. } => {
                let cursor_theme = win_state.cursor_theme.as_mut().unwrap();
                let cursor_name: &str = From::from(win_state.mouse_cursor);
                let cursor = cursor_theme
                    .get_cursor(cursor_name)
                    .expect(format!("Failed to get '{}' cursor", cursor_name).as_str());

                // TODO: Do We need to create a surface every time?
                let comp = win_state.compositor.as_ref().unwrap();
                let cursor_surface = comp.create_surface(qh, ());

                // TODO: Enable cursor animations
                let cursor_image = if win_state.cursor_visibility {
                    Some(&*cursor[0])
                } else {
                    None
                };

                cursor_surface.attach(cursor_image, 0, 0);
                cursor_surface.commit();

                let (hotspot_x, hotspot_y) = cursor[0].hotspot();
                pointer.set_cursor(
                    serial,
                    Some(&cursor_surface),
                    hotspot_x as i32,
                    hotspot_y as i32,
                );
            }

            _ => {}
        }
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
                "wl_subcompositor" => {
                    let subcompositor = registry.bind::<wl_subcompositor::WlSubcompositor, _, _>(
                        name,
                        version,
                        qh,
                        (),
                    );

                    state.subcompositor = Some(subcompositor);
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
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(name, version, qh, ());

                    state.seat = Some(seat);
                }
                _ => {}
            }
        }
    }
}

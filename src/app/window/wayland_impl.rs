use std::os::fd::AsFd;
use wayland_client::{
    protocol::wl_buffer, protocol::wl_compositor, protocol::wl_display, protocol::wl_registry,
    protocol::wl_shm, protocol::wl_shm_pool, protocol::wl_surface, Connection, Dispatch, Proxy,
    QueueHandle,
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

#[derive(Default)]
struct WindowState {
    configured: bool,
    display: Option<wl_display::WlDisplay>,
    compositor: Option<wl_compositor::WlCompositor>,

    base_surface: Option<wl_surface::WlSurface>,
    xdg_wm_base: Option<xdg_wm_base::XdgWmBase>,
    xdg_surface: Option<xdg_surface::XdgSurface>,
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
        let (width, height) = (800, 600);

        let shm_pool = shm.create_pool(file.as_fd(), (width * height * 4) as i32, qh, ());

        let buffer = shm_pool.create_buffer(
            0,
            width as i32,
            height as i32,
            (width * 4) as i32,
            wl_shm::Format::Argb8888,
            qh,
            (),
        );

        self.file = Some(file);
        self.buffer = Some(buffer);
    }

    fn draw(&mut self) {
        use std::{cmp::min, io::Write};
        let file = self.file.as_ref().unwrap();
        let mut buf = std::io::BufWriter::new(file);
        let (buf_x, buf_y) = (800, 600);
        for y in 0..buf_y {
            for x in 0..buf_x {
                let a = 0xFF;
                let r = min(((buf_x - x) * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
                let g = min((x * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
                let b = min(((buf_x - x) * 0xFF) / buf_x, (y * 0xFF) / buf_y);
                buf.write_all(&[b as u8, g as u8, r as u8, a as u8])
                    .unwrap();
            }
        }
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
        surface: &wl_surface::WlSurface,
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
        shm_pool: &wl_shm_pool::WlShmPool,
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
        buffer: &wl_buffer::WlBuffer,
        event: wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WindowState>,
    ) {
        println!(
            "Recived {} event: {:#?}",
            wl_buffer::WlBuffer::interface().name,
            event
        );
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for WindowState {
    fn event(
        _: &mut Self,
        compositor: &wl_compositor::WlCompositor,
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
        _: &mut Self,
        xdg_toplevel: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WindowState>,
    ) {
        //println!("Recived {} event: {:#?}", xdg_toplevel::XdgToplevel::interface().name, event);
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

impl Dispatch<wl_registry::WlRegistry, ()> for WindowState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<WindowState>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            //println!("[{}] {} (v{})", name, interface, version);

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
                _ => {}
            }
        }
    }
}

#[derive(Default)]
pub struct WaylandWindow {
    state: WindowState,
    qh: Option<QueueHandle<WindowState>>,
}

impl super::Window for WaylandWindow {
    fn new(_width: usize, _height: usize) -> Self {
        let mut state = WindowState::default();
        let conn = wayland_client::Connection::connect_to_env().unwrap();

        let display = conn.display();

        let mut event_queue = conn.new_event_queue::<WindowState>();
        let qh = event_queue.handle();

        let _registry = display.get_registry(&qh, ());
        event_queue.roundtrip(&mut state).unwrap();

        state.init_xdg_surface(&qh);
        state.allocate_shared_buffer(&qh);

        event_queue.roundtrip(&mut state).unwrap();

        Self::default()
    }

    fn handle(&mut self) -> Vec<super::Event> {
        /*
        state.draw();
        loop {
            event_queue.blocking_dispatch(&mut state).unwrap();
        };
        */
        vec![]
    }
    fn hide_mouse_cursor(&mut self) {}
    fn show_mouse_cursor(&mut self) {}
    fn toggle_fullscreen(&mut self) {}
    fn update_mouse_cursor(&mut self, _cursor: super::MouseCursor) {}
    fn set_mouse_position(&mut self, _x: i32, _y: i32) {}
    fn write_frame_from_ptr(&mut self, _src: *const u8, _sz: usize) {}
    fn write_frame_from_slice(&mut self, _src: &[u8]) {}
    fn get_window_position(&self) -> (i32, i32) {
        (0, 0)
    }
    fn get_screen_dim(&self) -> (usize, usize) {
        (0, 0)
    }
    fn get_window_dim(&self) -> (usize, usize) {
        (0, 0)
    }
}

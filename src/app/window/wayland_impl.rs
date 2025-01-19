use std::os::fd::AsFd;
use wayland_client::{
    backend::ObjectId,
    protocol::{
        wl_buffer, wl_compositor, wl_keyboard, wl_pointer, wl_region, wl_registry, wl_seat, wl_shm,
        wl_shm_pool, wl_subcompositor, wl_subsurface, wl_surface, wl_output,
    },
    Connection, Dispatch, Proxy, QueueHandle,
};
use wayland_protocols::xdg::shell::client::{
    xdg_surface, xdg_toplevel, xdg_toplevel::ResizeEdge, xdg_wm_base,
};

use super::window_decorator::{DecorationArea, WindowDecorator};

const CURSOR_SIZE: u32 = 24;

struct WindowFrame {
    base_surface: Option<wl_surface::WlSurface>,
    subsurface_role: Option<wl_subsurface::WlSubsurface>,
    file: Option<std::fs::File>,
    buffer: Option<wl_buffer::WlBuffer>,
    shm_pool: wl_shm_pool::WlShmPool,
    shm_pool_len: u32,
    input_region: wl_region::WlRegion,

    width: i32,
    height: i32,
    content_dimensions: (i32, i32),
    title_bar_height: u32,
    side_bar_thickness: u32,

    decorator: WindowDecorator,

}

impl WindowFrame {
    fn new(
        compositor: &wl_compositor::WlCompositor,
        subcompositor: &wl_subcompositor::WlSubcompositor,
        parent_surface: &wl_surface::WlSurface,
        shm: &wl_shm::WlShm,
        qh: &QueueHandle<WindowState>,
        content_dimensions: (i32, i32),
    ) -> Self {
        let new_surface = compositor.create_surface(qh, ());

        // PARAMETERS
        let title_bar_height = 50;
        let side_bar_thickness = 10;

        let (content_width, content_height) = content_dimensions;
        let window_dimensions = (
            content_width + side_bar_thickness * 2,
            content_height + title_bar_height + side_bar_thickness * 2,
        );
        let (win_width, win_height) = window_dimensions;
        let subsurface_position = (-side_bar_thickness, -title_bar_height - side_bar_thickness);

        // Add input region using surface coordinates
        let input_region = compositor.create_region(qh, ());
        input_region.add(0, 0, win_width, win_height);
        input_region.subtract(
            side_bar_thickness,
            title_bar_height + side_bar_thickness,
            content_width,
            content_height,
        );
        new_surface.set_input_region(Some(&input_region));

        // Create the subsurface role
        let subsurface = subcompositor.get_subsurface(&new_surface, parent_surface, qh, ());
        subsurface.set_position(subsurface_position.0, subsurface_position.1);

        let shm_pool_len = win_width * win_height * 4;
        let file = tempfile::tempfile().unwrap();
        file.set_len(shm_pool_len as u64).expect("Unable te resize file");
        let shm_pool = shm.create_pool(file.as_fd(), shm_pool_len, qh, ());

        let buffer = shm_pool.create_buffer(
            0,
            win_width,
            win_height,
            win_width * 4,
            wl_shm::Format::Argb8888,
            qh,
            (),
        );

        Self {
            base_surface: Some(new_surface.clone()),
            subsurface_role: Some(subsurface),
            input_region,
            file: Some(file),
            buffer: Some(buffer),
            shm_pool,
            shm_pool_len: shm_pool_len as u32,
            width: win_width,
            height: win_height,
            decorator: WindowDecorator::new(content_dimensions, title_bar_height, side_bar_thickness),
            title_bar_height: title_bar_height as u32,
            side_bar_thickness: side_bar_thickness as u32,
            content_dimensions,
        }
    }

    fn draw(&mut self) {
        self.decorator.render();
        //let frame = vec![255_u8; 800 * 4];
        let frame = self.decorator.frame_as_bytes_slice();

        use std::io::{Seek, Write};
        let mut file = self.file.as_ref().unwrap();
        file.rewind().unwrap();
        let mut buf = std::io::BufWriter::new(file);
        buf.write_all(&frame).unwrap();
        buf.flush().unwrap();
    }

    fn commit_surface(&mut self) {
        let subsurface = self.subsurface_role.as_ref().unwrap();
        let surface = self.base_surface.as_ref().unwrap();

        subsurface.set_sync();
        surface.attach(Some(self.buffer.as_ref().unwrap()), 0, 0);
        surface.damage(0, 0, self.width, self.height);
        surface.commit();
    }

    fn resize_buffer(&mut self, qh: &QueueHandle<WindowState>) {

        let new_len = (self.height * self.width * 4) as u32;

        // NOTE: We can only make the pool bigger
        if self.shm_pool_len < new_len {
            let file = self.file.as_ref().unwrap();
            file.set_len(new_len as u64).expect("Failed to resize file!");
            self.shm_pool.resize(new_len as i32);
            self.shm_pool_len = new_len;
        }

        let buffer = self.shm_pool.create_buffer(
            0,
            self.width,
            self.height,
            self.width * 4,
            wl_shm::Format::Argb8888,
            qh,
            (),
        );

        //self.buffer.as_ref().unwrap().destroy();
        self.buffer = Some(buffer);
    }
}

// TODO: rm this Default, and unnecessery Option's
#[derive(Default)]
struct WindowState {
    content_width: i32,
    content_height: i32,
    max_width: i32,
    max_height: i32,
    output_events: Vec<super::Event>,
    mouse_cursor: super::MouseCursor,
    mouse_pointer_info: super::MouseInfo,
    cursor_visibility: bool,
    cursor_theme: Option<wayland_cursor::CursorTheme>,
    compositor: Option<wl_compositor::WlCompositor>,
    seat: Option<wl_seat::WlSeat>,
    output: Option<wl_output::WlOutput>,
    subcompositor: Option<wl_subcompositor::WlSubcompositor>,
    base_surface: Option<wl_surface::WlSurface>,
    window_frame: Option<WindowFrame>,
    pointer_focused_surface: Option<ObjectId>,
    xdg_wm_base: Option<xdg_wm_base::XdgWmBase>,
    xdg_surface: Option<xdg_surface::XdgSurface>,
    xdg_toplevel: Option<xdg_toplevel::XdgToplevel>,
    queue_handle: Option<QueueHandle<Self>>,
    is_fullscreen: bool,
    configured_xdg_surface: bool,
    shm: Option<wl_shm::WlShm>,
    shm_pool: Option<wl_shm_pool::WlShmPool>,
    shm_pool_len: u32,
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

    // TODO: We could set the opaque region here
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

        file.set_len((self.content_width * self.content_height * 4) as u64)
            .expect("unable te resize file");
        let shm_pool = shm.create_pool(
            file.as_fd(),
            self.content_width * self.content_height * 4,
            qh,
            (),
        );

        self.shm_pool_len = (self.content_width * self.content_height * 4) as u32;

        let buffer = shm_pool.create_buffer(
            0,
            self.content_width,
            self.content_height,
            self.content_width * 4,
            wl_shm::Format::Xrgb8888,
            qh,
            (),
        );

        self.file = Some(file);
        self.buffer = Some(buffer);
        self.shm_pool = Some(shm_pool);
    }

    fn draw_frame(&mut self, frame: &[u8]) {
        use std::io::{Seek, Write};
        let mut file = self.file.as_ref().unwrap();
        file.rewind().unwrap();
        let mut buf = std::io::BufWriter::new(file);
        buf.write_all(frame).unwrap();
        buf.flush().unwrap();
    }

    fn treat_button_press_event(&mut self, serial: u32) {
        // In order to move the window we need to verify
        // if the ButtonPress occurred over the title bar of the
        // window frame.
        let window_frame = self.window_frame.as_ref().unwrap();
        let decoration_surface = window_frame.base_surface.as_ref().unwrap();
        let pointer_focus = self.pointer_focused_surface.as_ref();
        let pointer_x = self.mouse_pointer_info.x;
        let pointer_y = self.mouse_pointer_info.y;

        if pointer_focus.is_some_and(|focus| focus == &decoration_surface.id()) {
            if let Some(area_pressed) = window_frame.decorator.inside_area(pointer_x, pointer_y) {
                let seat = self.seat.as_ref().unwrap();
                let toplevel = self.xdg_toplevel.as_ref().unwrap();

                match area_pressed {
                    DecorationArea::TitleBar => {
                        toplevel._move(seat, serial);
                    }
                    DecorationArea::TopSideBar
                    | DecorationArea::BottomSideBar
                    | DecorationArea::RightSideBar
                    | DecorationArea::LeftSideBar => {
                        let edge = match area_pressed {
                            DecorationArea::TopSideBar => ResizeEdge::Top,
                            DecorationArea::BottomSideBar => ResizeEdge::Bottom,
                            DecorationArea::RightSideBar => ResizeEdge::Right,
                            DecorationArea::LeftSideBar => ResizeEdge::Left,
                            _ => ResizeEdge::None,
                        };

                        // TODO: 
                        // https://pop-os.github.io/cosmic-protocols/wayland_protocols/xdg/shell/client/xdg_toplevel/struct.XdgToplevel.html#method.resize
                        toplevel.resize(seat, serial, edge);
                    }
                }
            }
        }
    }

    fn push_out_mouse_event(&mut self, event: super::Event) {
        let base_surface = self.base_surface.as_ref().unwrap();
        let pointer_focus = self.pointer_focused_surface.as_ref();

        // Only output events if the pointer is focused on the window content
        if pointer_focus.is_some_and(|focus| focus == &base_surface.id()) {
            self.output_events.push(event);
        }
    }

    fn resize_buffer(&mut self) {
        let qh = self.queue_handle.as_ref().unwrap();
        let shm_pool = self.shm_pool.as_ref().unwrap();
        let new_len = (self.content_height * self.content_width * 4) as u32;

        // NOTE: We can only make the pool bigger
        if self.shm_pool_len < new_len {
            let file = self.file.as_ref().unwrap();
            file.set_len(new_len as u64).expect("Failed to resize file!");
            shm_pool.resize(new_len as i32);
            self.shm_pool_len = new_len;
        }


        let buffer = shm_pool.create_buffer(
            0,
            self.content_width,
            self.content_height,
            self.content_width * 4,
            wl_shm::Format::Xrgb8888,
            qh,
            (),
        );

        //self.buffer.as_ref().unwrap().destroy();
        self.buffer = Some(buffer);
    }

    fn resize_window(&mut self, width: u32, height: u32) {

        let (content_width, content_height) = if self.window_frame.is_some() {
            // TODO: Update WindowFrame input region
            let frame = self.window_frame.as_mut().unwrap();

            // Subtract the current input region
            frame.input_region.subtract(0, 0, frame.width, frame.height);

            frame.content_dimensions = 
            ((width - frame.side_bar_thickness * 2) as i32, (height - frame.side_bar_thickness * 2 - frame.title_bar_height) as i32);
            frame.decorator.resize_content(frame.content_dimensions);
            let (content_width, content_height) = frame.content_dimensions;
            frame.width = width as i32;
            frame.height = height as i32;

            let qh = self.queue_handle.as_ref().unwrap();
            frame.resize_buffer(qh);

            // Update input_region
            frame.input_region.add(0, 0, frame.width, frame.height);
            frame.input_region.subtract(
                frame.side_bar_thickness as i32,
                (frame.title_bar_height + frame.side_bar_thickness) as i32,
                content_width,
                content_height,
            );

            let base_surface = frame.base_surface.as_ref().unwrap();
            base_surface.set_input_region(Some(&frame.input_region));

            frame.content_dimensions
        } else {
            (width as i32, height as i32)
        };

        self.content_width = content_width;
        self.content_height = content_height;

        self.resize_buffer();

        self.output_events.push(
            super::Event::RedimWindow(
                (content_width as usize, content_height as usize)
            )
        );
    }
}

pub struct WaylandWindow {
    state: WindowState,
    event_queue: wayland_client::EventQueue<WindowState>,
}

impl super::Window for WaylandWindow {
    fn new(width: usize, height: usize) -> Self {
        let mut state = WindowState {
            content_width: width as i32,
            content_height: height as i32,
            cursor_visibility: true,
            pointer_focused_surface: Some(ObjectId::null()),
            ..Default::default()
        };
        let conn = wayland_client::Connection::connect_to_env().unwrap();

        let display = conn.display();

        let mut event_queue = conn.new_event_queue::<WindowState>();
        let qh = event_queue.handle();
        // Store QueueHandle
        state.queue_handle = Some(qh.clone());

        let _registry = display.get_registry(&qh, ());
        event_queue.roundtrip(&mut state).unwrap();

        state.init_xdg_surface(&qh);
        state.allocate_shared_buffer(&qh);
        state.load_cursor_theme(&conn);

        // TODO: check if this second roundtrip is really necessary
        event_queue.roundtrip(&mut state).unwrap();

        let base_surface = state.base_surface.as_ref().unwrap();
        let compositor = state.compositor.as_ref().unwrap();
        let subcompositor = state.subcompositor.as_ref().unwrap();
        let shm = state.shm.as_ref().unwrap();

        let window_frame = WindowFrame::new(
            compositor,
            subcompositor,
            base_surface,
            shm,
            &qh,
            (width as i32, height as i32),
        );

        state.window_frame = Some(window_frame);

        Self { state, event_queue }
    }

    fn handle(&mut self) -> Vec<super::Event> {
        self.event_queue.blocking_dispatch(&mut self.state).unwrap();
        self.state.output_events.drain(..).collect()
    }
    fn write_frame_from_ptr(&mut self, _src: *const u8, _sz: usize) {}
    fn write_frame_from_slice(&mut self, src: &[u8]) {
        assert_eq!(
            self.state.content_width * self.state.content_height * 4,
            src.len() as i32
        );
        self.state.draw_frame(src);
        self.state.window_frame.as_mut().unwrap().draw();

        if self.state.configured_xdg_surface {
            self.state.window_frame.as_mut().unwrap().commit_surface();

            let buffer = self.state.buffer.as_ref().unwrap();
            let surface = self.state.base_surface.as_ref().unwrap();
            surface.attach(Some(buffer), 0, 0);
            surface.damage_buffer(0, 0, self.state.content_width, self.state.content_height);
            surface.commit();
        }
    }
    fn get_window_position(&self) -> (i32, i32) {
        (0, 0)
    }
    fn get_screen_dim(&self) -> (usize, usize) {
        (
            self.state.max_width as usize,
            self.state.max_height as usize,
        )
    }
    fn get_window_dim(&self) -> (usize, usize) {
        (
            self.state.content_width as usize,
            self.state.content_height as usize,
        )
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
        event: wl_shm::Event,
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
        /*
        println!(
            "Recived {} event: {:#?}",
            wl_surface::WlSurface::interface().name,
            event
        );
        */
    }
}

impl Dispatch<wl_subsurface::WlSubsurface, ()> for WindowState {
    fn event(
        _: &mut Self,
        _: &wl_subsurface::WlSubsurface,
        event: wl_subsurface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WindowState>,
    ) {
        /*
        println!(
            "Recived {} event: {:#?}",
            wl_subsurface::WlSubsurface::interface().name,
            event
        );
        */
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
        /*
        println!(
            "Recived {} event: {:#?}",
            wl_shm_pool::WlShmPool::interface().name,
            event
        );
        */
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for WindowState {
    fn event(
        _: &mut Self,
        _: &wl_buffer::WlBuffer,
        event: wl_buffer::Event,
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
        /*
        println!(
            "Recived {} event: {:#?}",
            wl_subcompositor::WlSubcompositor::interface().name,
            event
        );
        */
    }
}

impl Dispatch<wl_region::WlRegion, ()> for WindowState {
    fn event(
        _: &mut Self,
        _: &wl_region::WlRegion,
        event: wl_region::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WindowState>,
    ) {
        /*
        println!(
            "Recived {} event: {:#?}",
            wl_region::WlRegion::interface().name,
            event
        );
        */
    }
}

impl Dispatch<wl_output::WlOutput, ()> for WindowState {
    fn event(
        state: &mut Self,
        _: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WindowState>,
    ) {
        println!(
            "Recived {} event: {:#?}",
            wl_output::WlOutput::interface().name,
            event
        );

        if let wl_output::Event::Mode{ width, height, .. } = event {
            state.max_width = width;
            state.max_height = height;
        }
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
        /*
        println!(
            "Recived {} event: {:#?}",
            wl_compositor::WlCompositor::interface().name,
            event
        );
        */
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
                let size_changed = width != state.content_width || height != state.content_height;
                if width != 0 && height != 0 && size_changed
                {
                    println!("resize WINDOW!!");
                    // NOTE: This dimensions includes the area ocupied by the subsurfaces
                    // TODO: We need to check if the WindowDecoration subsurface is alreday
                    // attached before updating the content dimensions state.
                    state.resize_window(width as u32, height as u32);
                }
            }
            // Current window bounds (maximized)
            xdg_toplevel::Event::ConfigureBounds { width, height } => {
                if width != 0 && height != 0 {
                    //state.max_width = width;
                    //state.max_height = height;
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
        /*
        println!(
            "Recived {} event: {:#?}",
            xdg_surface::XdgSurface::interface().name,
            event
        );
        */

        if let xdg_surface::Event::Configure { serial } = event {
            xdg_surface.ack_configure(serial);
            state.configured_xdg_surface = true;
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
            wl_pointer::Event::Button {
                button,
                state,
                serial,
                ..
            } => {
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
                        win_state.treat_button_press_event(serial);
                        super::Event::ButtonPress(mouse_button)
                    }
                    _ => super::Event::Empty,
                };

                win_state.push_out_mouse_event(button_event);
            }

            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                // TODO: Enable cursor animations

                // Update pointer position
                win_state
                    .mouse_pointer_info
                    .update(surface_x as i32, surface_y as i32);

                win_state.push_out_mouse_event(super::Event::MouseMotion(
                    win_state.mouse_pointer_info.clone(),
                ));
            }

            wl_pointer::Event::Enter {
                serial,
                surface,
                surface_x,
                surface_y,
            } => {
                // Update focused surface
                win_state.pointer_focused_surface = Some(surface.id());

                // Update pointer position
                win_state
                    .mouse_pointer_info
                    .update(surface_x as i32, surface_y as i32);
                win_state.mouse_pointer_info.dx = 0;
                win_state.mouse_pointer_info.dy = 0;

                win_state.push_out_mouse_event(super::Event::MouseMotion(
                    win_state.mouse_pointer_info.clone(),
                ));

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

            wl_pointer::Event::Leave { .. } => {
                win_state.pointer_focused_surface = None;

                // TODO: Send output MouseUpdate event when leaving the
                // content's surface. The GUI needs to know that the pointer
                // position is not hovering any elements.
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
        /*
        println!(
            "Recived {} event: {:#?}",
            wl_keyboard::WlKeyboard::interface().name,
            event
        );
        */
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
        /*
        println!(
            "Global: Recived {} event: {:#?}",
            wl_registry::WlRegistry::interface().name,
            event
        );
        */
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
                "wl_output" => {
                    let output = registry.bind::<wl_output::WlOutput, _, _>(name, version, qh, ());

                    state.output = Some(output);
                }
                _ => {}
            }
        }
    }
}

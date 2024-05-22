mod draw;

// shared memory iamge -> https://www.x.org/releases/X11R7.7/doc/xextproto/shm.html
// https://handmade.network/forums/articles/t/2834-tutorial_a_tour_through_xlib_and_related_technologies
// events -> https://www.oreilly.com/library/view/xlib-reference-manual/9780937175262/13_appendix-e.html
// xlib c header -> https://codebrowser.dev/gtk/include/X11/X.h.html
fn main() {
    let mut win =  draw::Window::new(800, 600);
    win.run();
}

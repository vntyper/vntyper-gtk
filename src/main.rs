// Copyright 2016 Do Duy.
// Licensed under the MIT license, see the LICENSE file or
// <http://opensource.org/licenses/MIT>

extern crate gtk;
extern crate gdk;
extern crate vntyper;

use vntyper::input;
use vntyper::input_method;
use std::io::Write;

use gtk::prelude::*;
use gtk::Window;
use gtk::TextView;
use gtk::TextBuffer;
use gtk::TextIter;
use gtk::ScrolledWindow;
use gdk::enums::modifier_type as modifier;

trait Buffer {
    fn get_insert_iter(&self) -> Option<TextIter>;
    fn complete(&self, c: char) -> Inhibit;
    fn get_content(&self) -> String;
}

macro_rules! gtk_try {
    ( $x:expr ) => {
        {
            let tmp = $x;
            if $x.is_none() { return Inhibit(false) }
            tmp.unwrap()
        }
    };
}

impl Buffer for TextBuffer {
    fn get_insert_iter(&self) -> Option<TextIter> {
        self.get_insert().map(|x| self.get_iter_at_mark(&x))
    }
    fn complete(&self, c: char) -> Inhibit {
        let mut end_iter = gtk_try!(self.get_insert_iter());

        let mut start_iter = end_iter.clone();
        start_iter.backward_chars(15);

        let text = gtk_try!(self.get_text(&start_iter, &end_iter, false));
        let vntyper = input::Input::new(text, c, input_method::InputMethod::telex());
        let output = vntyper.output();
        let mut set_text = |s: &str| {
            self.delete(&mut start_iter, &mut end_iter);
            self.insert(&mut start_iter, s);
        };
        match output {
            Err(s) => {
                set_text(&s);
                Inhibit(false)
            },
            Ok(s) => {
                set_text(&s);
                Inhibit(true)
            },
        }
    }
    fn get_content(&self) -> String {
        self.get_text(&self.get_start_iter(), &self.get_end_iter(), false)
            .unwrap_or(String::new())
    }
}

fn clipboard_copy(s: &str) {
    let xclip = match std::process::Command::new("xclip").args(&["-sel", "clip"])
        .stdin(std::process::Stdio::piped()).spawn() {
        Err(_) => return (),
        Ok(x) => x,
    };
    if let Err(e) = xclip.stdin.unwrap().write_all(s.as_bytes()) {
        println!("{:?}", e);
    }
}

fn main() {
    if gtk::init().is_err() {
        panic!();
    }
    let window = Window::new(gtk::WindowType::Toplevel);
    let text_view = TextView::new();
    let scrolled_window = ScrolledWindow::new(None, None);
    text_view.set_wrap_mode(gtk::WrapMode::Word);
    window.set_default_size(700, 500);
    scrolled_window.add(&text_view);
    window.add(&scrolled_window);
    window.set_title("Vietnamese Input");
    window.show_all();

    text_view.connect_key_press_event(|widget, ev| {
        let key = gtk_try!(std::char::from_u32(ev.get_keyval()));
        match widget.get_buffer() {
            Some(buffer) => {
                // <CTRL + s> pressed
                if key == 's' && (ev.get_state() & modifier::ControlMask).bits() != 0 {
                    clipboard_copy(&buffer.get_content());
                    gtk::main_quit();
                    Inhibit(false)
                } else {
                    buffer.complete(key)
                }
            },
            None => Inhibit(false),
        }
    });

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    gtk::main();
}

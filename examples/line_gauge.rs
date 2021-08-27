//! ## Demo
//!
//! `Demo` shows how to use tui-realm in a real case

/**
 * MIT License
 *
 * tui-realm - Copyright (C) 2021 Christian Visintin
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
mod utils;

use utils::context::Context;
use utils::keymap::*;

use std::thread::sleep;
use std::time::{Duration, Instant};

use tui_realm_stdlib::{Label, LabelPropsBuilder, LineGauge, LineGaugePropsBuilder};
use tuirealm::props::{
    borders::{BorderType, Borders},
    Alignment,
};
use tuirealm::{Msg, PropsBuilder, Update, View};
// tui
use tuirealm::tui::layout::{Constraint, Direction, Layout};
use tuirealm::tui::style::Color;

const COMPONENT_PROGRESSBAR: &str = "prog1";
const COMPONENT_PROGRESSBAR_2: &str = "prog2";
const COMPONENT_EVENT: &str = "LABEL";

struct Model {
    quit: bool,           // Becomes true when the user presses <ESC>
    redraw: bool,         // Tells whether to refresh the UI; performance optimization
    last_redraw: Instant, // Last time the ui has been redrawed
    view: View,
}

impl Model {
    fn new(view: View) -> Self {
        Model {
            quit: false,
            redraw: true,
            last_redraw: Instant::now(),
            view,
        }
    }

    fn quit(&mut self) {
        self.quit = true;
    }

    fn redraw(&mut self) {
        self.redraw = true;
    }

    fn reset(&mut self) {
        self.redraw = false;
        self.last_redraw = Instant::now();
    }
}

fn main() {
    // let's create a context: the context contains the backend of crossterm and the input handler
    let mut ctx: Context = Context::new();
    // Enter alternate screen
    ctx.enter_alternate_screen();
    // Clear screen
    ctx.clear_screen();
    // let's create a `View`, which will contain the components
    let mut myview: View = View::init();
    // Mount the component you need; we'll use a Label and an Input
    myview.mount(
        COMPONENT_PROGRESSBAR,
        Box::new(LineGauge::new(
            LineGaugePropsBuilder::default()
                .with_background(Color::Black)
                .with_progbar_color(Color::Yellow)
                .with_borders(Borders::ALL, BorderType::Thick, Color::Yellow)
                .with_progress(0.64)
                .with_title("Downloading termscp 0.5.0", Alignment::Center)
                .with_label("64.2% - ETA 00:48")
                .with_line_rounded()
                .build(),
        )),
    );
    myview.mount(
        COMPONENT_PROGRESSBAR_2,
        Box::new(LineGauge::new(
            LineGaugePropsBuilder::default()
                .with_background(Color::Black)
                .with_progbar_color(Color::LightBlue)
                .with_progress(0.80)
                .with_title("Downloading termscp 0.5.0", Alignment::Center)
                .with_label("0.0% - ETA --:--")
                .with_line_thick()
                .build(),
        )),
    );
    myview.mount(
        COMPONENT_EVENT,
        Box::new(Label::new(
            LabelPropsBuilder::default()
                .with_foreground(Color::Cyan)
                .build(),
        )),
    );
    // We need to give focus to input then
    myview.active(COMPONENT_PROGRESSBAR);
    // Now we use the Model struct to keep track of some states
    let mut model: Model = Model::new(myview);
    // let's loop until quit is true
    while !model.quit {
        // Listen for input events
        if let Ok(Some(ev)) = ctx.input_hnd.read_event() {
            // Pass event to view
            let msg = model.view.on(ev);
            model.redraw();
            // Call the elm friend update
            model.update(msg);
        }
        // If redraw, draw interface
        if model.redraw || model.last_redraw.elapsed() > Duration::from_millis(50) {
            // Call the elm friend vie1 function
            view(&mut ctx, &model.view);
            model.reset();
        }
        sleep(Duration::from_millis(10));
    }
    // Let's drop the context finally
    drop(ctx);
}

fn view(ctx: &mut Context, view: &View) {
    let _ = ctx.terminal.draw(|f| {
        // Prepare chunks
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(f.size());
        view.render(COMPONENT_PROGRESSBAR, f, chunks[0]);
        view.render(COMPONENT_PROGRESSBAR_2, f, chunks[1]);
        view.render(COMPONENT_EVENT, f, chunks[2]);
    });
}

impl Update for Model {
    fn update(&mut self, msg: Option<(String, Msg)>) -> Option<(String, Msg)> {
        let ref_msg: Option<(&str, &Msg)> = msg.as_ref().map(|(s, msg)| (s.as_str(), msg));
        match ref_msg {
            None => None, // Exit after None
            Some(msg) => match msg {
                (COMPONENT_PROGRESSBAR, key) if key == &MSG_KEY_TAB => {
                    self.view.active(COMPONENT_PROGRESSBAR_2);
                    None
                }
                (COMPONENT_PROGRESSBAR_2, key) if key == &MSG_KEY_TAB => {
                    self.view.active(COMPONENT_PROGRESSBAR);
                    None
                }
                (_, key) if key == &MSG_KEY_ESC => {
                    // Quit on esc
                    self.quit();
                    None
                }
                (component, event) => {
                    // Update span
                    let props =
                        LabelPropsBuilder::from(self.view.get_props(COMPONENT_EVENT).unwrap())
                            .with_text(format!("{} => '{:?}'", component, event))
                            .build();
                    // Report submit
                    let _ = self.view.update(COMPONENT_EVENT, props);
                    None
                }
            },
        }
    }
}
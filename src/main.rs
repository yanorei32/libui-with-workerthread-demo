use tokio::runtime;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

#[derive(Debug, Clone)]
enum ServerMessage {
    Died,
}

#[derive(Debug, Clone)]
enum UiMessage {
    HeartBeat,
}

use libui::controls::*;
use libui::prelude::*;

struct ServerCommunication {
    tx: Sender<UiMessage>,
    reverse_rx: Receiver<ServerMessage>,
}

struct State {
    server: Option<ServerCommunication>,
}

async fn server_thread(mut rx: Receiver<UiMessage>, reverse_sender: Sender<ServerMessage>) {
    loop {
        match tokio::time::timeout(Duration::from_millis(1000), rx.recv()).await {
            Ok(_v) => {}
            Err(_) => {
                reverse_sender.send(ServerMessage::Died).await.unwrap();
            }
        }
    }
}

fn main() {
    let ui = UI::init().unwrap();

    let state = Rc::new(RefCell::new(State { server: None }));

    let rt = runtime::Builder::new_multi_thread()
        .enable_time()
        .build()
        .unwrap();

    let mut win = Window::new(&ui, "Example", 300, 200, WindowType::NoMenubar);
    let mut layout = VerticalBox::new();

    let label = Label::new("-");

    let mut start = Button::new("Start");
    let mut heart = Button::new("Heart");
    heart.disable();

    start.on_clicked({
        let mut start = start.clone();
        let s = state.clone();
        let mut label = label.clone();
        let mut heart = heart.clone();

        move |_| {
            let (tx, rx) = mpsc::channel(128);
            let (reverse_tx, reverse_rx) = mpsc::channel(128);

            label.set_text("おはよう！");
            s.borrow_mut().server = Some(ServerCommunication { reverse_rx, tx });

            rt.spawn(server_thread(rx, reverse_tx));

            start.disable();
            heart.enable();
        }
    });

    heart.on_clicked({
        let s = state.clone();

        move |_| {
            s.borrow_mut()
                .server
                .as_mut()
                .unwrap()
                .tx
                .blocking_send(UiMessage::HeartBeat)
                .unwrap();
        }
    });

    layout.append(label.clone(), LayoutStrategy::Stretchy);
    layout.append(start.clone(), LayoutStrategy::Stretchy);
    layout.append(heart.clone(), LayoutStrategy::Stretchy);

    win.set_child(layout);
    win.show();

    let mut event_loop = ui.event_loop();

    event_loop.on_tick({
        let state = state.clone();
        let mut label = label.clone();
        let mut heart = heart.clone();
        let mut start = start.clone();
        move || {
            let state = &mut state.borrow_mut();

            let Some(server) = &mut state.server else {
                return;
            };

            if let Ok(message) = &mut server.reverse_rx.try_recv() {
                match message {
                    ServerMessage::Died => {
                        state.server = None;
                        heart.disable();
                        start.enable();
                        label.set_text("ぐえー、しんだんご");
                    }
                }
            };
        }
    });

    event_loop.run_delay(1000 / 30);
}

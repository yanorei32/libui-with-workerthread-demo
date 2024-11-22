use tokio::runtime;
use tokio::sync::mpsc;

use std::time::Duration;

#[derive(Debug, Clone)]
enum ServerMessage {
    Update(i32),
}

#[derive(Debug, Clone)]
enum UiMessage {
    ChangeDelta(i32),
}

use libui::controls::*;
use libui::prelude::*;

fn main() {
    let ui = UI::init().unwrap();

    let (tx, mut rx) = mpsc::channel(128);
    let (reverse_tx, mut reverse_rx) = mpsc::channel(128);

    let rt = runtime::Builder::new_multi_thread()
        .enable_time()
        .build()
        .unwrap();

    rt.spawn(async move {
        let mut delta = 0;
        let mut current = 0;
        let mut interval = tokio::time::interval(Duration::from_millis(50));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    current += delta;
                    reverse_tx.send(ServerMessage::Update(current)).await.unwrap();
                }
                message = rx.recv() => {
                    let message: UiMessage = message.unwrap();
                    match message {
                        UiMessage::ChangeDelta(n) => delta = n,
                    }
                }
            }
        }
    });

    let mut win = Window::new(&ui, "Example", 300, 200, WindowType::NoMenubar);
    let mut layout = VerticalBox::new();

    let mut slider = Slider::new(-10, 10);
    slider.set_value(0);

    slider.on_changed(move |v| {
        tx.blocking_send(UiMessage::ChangeDelta(v)).unwrap();
    });

    let label = Label::new("Hello");

    let button = Button::new("Start");

    layout.append(label.clone(), LayoutStrategy::Stretchy);
    layout.append(slider, LayoutStrategy::Stretchy);
    layout.append(button, LayoutStrategy::Stretchy);

    win.set_child(layout);
    win.show();

    let mut event_loop = ui.event_loop();

    event_loop.on_tick({
        let mut label = label.clone();
        move || {
            if let Ok(message) = reverse_rx.try_recv() {
                match message {
                    ServerMessage::Update(v) => label.set_text(&format!("{v}")),
                }
            };
        }
    });

    event_loop.run_delay(1000 / 30);
}

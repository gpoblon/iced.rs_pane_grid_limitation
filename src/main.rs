use iced::alignment::{self, Alignment};
use iced::executor;
use iced::theme::{Button, Theme};
use iced::widget::pane_grid::{self, PaneGrid};
use iced::widget::{button, column, container, responsive, row, scrollable, text};
use iced::{Application, Command, Element, Length, Settings, Size};
use std::collections::HashMap;

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

// nothing interesting in Example struct. Go to PaneSystem struct directly

struct Example {
    pane_system: PaneSystem,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Pane(PaneMessage),
}

impl Application for Example {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Example {
                pane_system: PaneSystem::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Recursive Pane grid SSCCE - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        let Message::Pane(pane_msg) = message;
        let _ = self.pane_system.update(pane_msg);
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        self.pane_system.view().map(Message::Pane)
    }
}

// here we go

type PaneId = usize;

#[derive(Debug, Clone, Copy)]
enum PaneMessage {
    Split(pane_grid::Axis, pane_grid::Pane),
    InsertChild(PaneId),
    RemoveChild(PaneId),
}

// only two depths but it is enough to reproduce the issue/limitation:
// just insert a child, and scrollable height will become "infinite"
// if scrollable is removed, the parent/child pane height works properly.

struct PaneSystem {
    panes: pane_grid::State<PaneId>,
    children: HashMap<PaneId, PaneSystem>,
}

impl PaneSystem {
    fn new() -> Self {
        Self {
            panes: pane_grid::State::new(0).0,
            children: HashMap::new(),
        }
    }

    fn update(&mut self, message: PaneMessage) -> Command<PaneMessage> {
        match message {
            PaneMessage::Split(axis, pane) => {
                self.panes.split(axis, &pane, self.panes.len());
            }
            PaneMessage::InsertChild(pane_id) => {
                self.children.insert(pane_id, PaneSystem::new());
            }
            PaneMessage::RemoveChild(pane_id) => {
                self.children.remove(&pane_id);
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<PaneMessage> {
        let pane_grid = PaneGrid::new(&self.panes, |id, pane_id, _| {
            pane_grid::Content::new(responsive(move |size| {
                self.view_content(id, *pane_id, size)
            }))
            .style(style::pane)
        })
        .width(Length::Fill)
        // Shrink height sets the pane height to 0
        .height(Length::Fill);

        container(pane_grid)
            .width(Length::Fill)
            // Shrink height sets the pane height to 0
            .height(Length::Fill)
            .padding(20)
            .into()
    }

    fn view_content<'a>(
        &'a self,
        pane: pane_grid::Pane,
        pane_id: PaneId,
        size: Size,
    ) -> Element<'a, PaneMessage> {
        let button = |label, message| {
            button(
                text(label)
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .width(Length::Fill)
            .padding(10)
            .on_press(message)
        };

        let controls = row![
            text(format!("{}x{}", size.width, size.height)),
            button(
                "hsplit",
                PaneMessage::Split(pane_grid::Axis::Horizontal, pane)
            ),
            button(
                "vsplit",
                PaneMessage::Split(pane_grid::Axis::Vertical, pane)
            ),
            button("insert child", PaneMessage::InsertChild(pane_id)),
            button("remove child", PaneMessage::RemoveChild(pane_id)).style(Button::Destructive),
        ]
        .width(Length::Shrink)
        .align_items(Alignment::Center);

        // here is where the inner pane system is rendered, creating the infinite size due to the scrollable below
        let child_if_any: Element<'a, PaneMessage> = {
            match self.children.get(&pane_id) {
                Some(child) => child.view().into(),
                None => column!().into(),
            }
        };

        // I don't see how I could get rid of scrollable, as many usecases imply having a height bigger than the screen
        scrollable(
            container(column![controls, child_if_any])
                .width(Length::Fill)
                .height(Length::Shrink)
                .padding(20),
        )
        .height(Length::Shrink)
        .into()
    }
}

mod style {
    use iced::widget::container;
    use iced::Theme;

    pub fn pane(_: &Theme) -> container::Appearance {
        container::Appearance {
            border_width: 1.0,
            border_color: iced_core::Color::BLACK,
            ..Default::default()
        }
    }
}

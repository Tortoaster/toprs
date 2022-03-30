use crate::component::event::{Event, Feedback};
use crate::component::id::{ComponentCreator, Id};
use crate::component::{Component, Widget};
use crate::editor::{Editor, EditorError, Report};
use crate::viewer::Viewer;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChoiceEditor<V> {
    id: Id,
    options: Vec<V>,
    choice: Option<usize>,
}

impl<V> ChoiceEditor<V>
where
    V: Viewer,
{
    pub fn new(options: Vec<V::Input>) -> Self {
        let options = options.into_iter().map(|option| V::start(option)).collect();

        ChoiceEditor {
            id: Id::INVALID,
            options,
            choice: None,
        }
    }
}

impl<V> Editor for ChoiceEditor<V>
where
    V: Viewer,
{
    type Input = usize;
    type Output = V::Output;

    fn component(&mut self, ctx: &mut ComponentCreator) -> Component {
        let options = self
            .options
            .iter()
            .map(|option| option.component(ctx))
            .collect();
        let component = ctx.create(Widget::RadioGroup { options });
        self.id = component.id();
        component
    }

    fn on_event(&mut self, event: Event, _ctx: &mut ComponentCreator) -> Option<Feedback> {
        match event {
            Event::Update { id, value } if self.id == id => match value.parse() {
                Ok(usize) => {
                    self.choice = Some(usize);
                    Some(Feedback::Valid { id })
                }
                Err(_) => Some(Feedback::Invalid { id }),
            },
            _ => None,
        }
    }

    fn read(&self) -> Report<Self::Output> {
        match self.choice {
            None => Err(EditorError::Invalid),
            Some(index) => self
                .options
                .get(index)
                .map(|viewer| viewer.read())
                .ok_or(EditorError::Invalid),
        }
    }

    fn write(&mut self, value: Self::Input) {
        self.choice = Some(value);
    }
}
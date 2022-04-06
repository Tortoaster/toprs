use crate::editor::{Editor, EditorError};
use crate::event::{Event, Feedback};
use crate::html::{AsHtml, Html, RadioGroup};
use crate::id::{Generator, Id};
use crate::viewer::Viewer;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChoiceEditor<V> {
    id: Id,
    choices: Vec<V>,
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
            choices: options,
            choice: None,
        }
    }
}

impl<V> AsHtml for ChoiceEditor<V>
where
    V: AsHtml,
{
    fn as_html(&self) -> Html {
        let options = self.choices.iter().map(V::as_html).collect();
        RadioGroup::new(self.id, options).as_html()
    }
}

impl<V> Editor for ChoiceEditor<V>
where
    V: Viewer,
{
    type Input = usize;
    type Output = V::Output;

    fn start(&mut self, value: Option<Self::Input>, gen: &mut Generator) {
        self.choice = value;
        self.id = gen.next();
    }

    fn on_event(&mut self, event: Event, _gen: &mut Generator) -> Option<Feedback> {
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

    fn finish(&self) -> Result<Self::Output, EditorError> {
        match self.choice {
            None => Err(EditorError::Invalid),
            Some(index) => self
                .choices
                .get(index)
                .map(|viewer| viewer.finish())
                .ok_or(EditorError::Invalid),
        }
    }
}

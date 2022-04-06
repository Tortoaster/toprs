use crate::editor::{Editor, EditorError};
use crate::event::{Event, Feedback};
use crate::html::{AsHtml, Div, Html, Icon, IconButton, Layout};
use crate::id::{Generator, Id};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VecEditor<E> {
    /// Represents the list containing all choices.
    group_id: Id,
    /// Represents the plus button.
    add_id: Id,
    /// Represents of each of the choices with their respective identifiers.
    choices: Vec<Row>,
    editors: Vec<E>,
    template: E,
}

impl<E> VecEditor<E> {
    pub fn new(editor: E) -> Self {
        VecEditor {
            group_id: Id::INVALID,
            add_id: Id::INVALID,
            choices: Vec::new(),
            editors: Vec::new(),
            template: editor,
        }
    }
}

impl<E> AsHtml for VecEditor<E>
where
    E: AsHtml,
{
    fn as_html(&self) -> Html {
        let children = self
            .editors
            .iter()
            .zip(&self.choices)
            .map(|(editor, row)| row.as_html(editor))
            .collect();

        let group = Div::new(children).with_id(self.group_id).as_html();
        let button = Row::add_button(self.add_id);

        Div::new(vec![group, button]).as_html()
    }
}

impl<E> Editor for VecEditor<E>
where
    E: Editor + Clone,
{
    type Input = Vec<E::Input>;
    type Output = Vec<E::Output>;

    fn start(&mut self, value: Option<Self::Input>, gen: &mut Generator) {
        self.group_id = gen.next();
        self.add_id = gen.next();
        self.editors = value
            .into_iter()
            .flatten()
            .map(|input| {
                let mut editor = self.template.clone();
                editor.start(Some(input), gen);
                editor
            })
            .collect();
        self.choices = self.editors.iter().map(|_| Row::new(gen)).collect();
    }

    fn on_event(&mut self, event: Event, gen: &mut Generator) -> Option<Feedback> {
        match event {
            Event::Press { id } if id == self.add_id => {
                // Add a new row
                let mut editor = self.template.clone();
                editor.start(None, gen);
                let row = Row::new(gen);
                let html = row.as_html(&editor);

                self.editors.push(editor);
                self.choices.push(row);

                Some(Feedback::Append {
                    id: self.group_id,
                    html,
                })
            }
            Event::Press { id } if self.choices.iter().any(|row| row.sub_id == id) => {
                // Remove an existing row
                let index = self
                    .choices
                    .iter()
                    .position(|row| row.sub_id == id)
                    .unwrap();
                let Row { id, .. } = self.choices.remove(index);
                self.editors.remove(index);

                Some(Feedback::Remove { id })
            }
            _ => self
                .editors
                .iter_mut()
                .find_map(|editor| editor.on_event(event.clone(), gen)),
        }
    }

    fn finish(&self) -> Result<Self::Output, EditorError> {
        // TODO: Return all errors
        self.editors
            .iter()
            .map(|editor| editor.finish())
            .collect::<Result<Vec<_>, _>>()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptionEditor<E> {
    /// Represents the row containing the editor and the minus button if a value is present.
    row: Row,
    /// Represents the plus button if there is no value present.
    add_id: Id,
    editor: E,
    /// True if this editor contains a value, false otherwise.
    enabled: bool,
}

impl<E> OptionEditor<E>
where
    E: Editor,
{
    pub fn new(editor: E) -> Self {
        OptionEditor {
            row: Row {
                id: Id::INVALID,
                sub_id: Id::INVALID,
            },
            add_id: Id::INVALID,
            editor,
            enabled: false,
        }
    }
}

impl<E> AsHtml for OptionEditor<E>
where
    E: AsHtml,
{
    fn as_html(&self) -> Html {
        if self.enabled {
            self.row.as_html(&self.editor)
        } else {
            Row::add_button(self.add_id)
        }
    }
}

impl<E> Editor for OptionEditor<E>
where
    E: Editor,
{
    type Input = Option<E::Input>;
    type Output = Option<E::Output>;

    fn start(&mut self, value: Option<Self::Input>, gen: &mut Generator) {
        self.row = Row::new(gen);
        self.add_id = gen.next();
        self.enabled = value.is_some();

        self.editor.start(value.flatten(), gen);
    }

    fn on_event(&mut self, event: Event, gen: &mut Generator) -> Option<Feedback> {
        match event {
            Event::Press { id } if id == self.add_id && !self.enabled => {
                // Add value
                let id = self.add_id;
                let html = self.row.as_html(&mut self.editor);
                self.enabled = true;

                Some(Feedback::Replace { id, html })
            }
            Event::Press { id } if id == self.row.sub_id && self.enabled => {
                // Remove value
                let id = self.row.id;
                let html = Row::add_button(self.add_id);
                self.enabled = false;

                Some(Feedback::Replace { id, html })
            }
            _ => self
                .enabled
                .then(|| self.editor.on_event(event, gen))
                .flatten(),
        }
    }

    fn finish(&self) -> Result<Self::Output, EditorError> {
        if self.enabled {
            Ok(Some(self.editor.finish()?))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Row {
    id: Id,
    sub_id: Id,
}

impl Row {
    fn new(gen: &mut Generator) -> Self {
        Row {
            id: gen.next(),
            sub_id: gen.next(),
        }
    }

    /// Creates a row consisting of the editor and a button to remove it.
    fn as_html<E>(&self, editor: &E) -> Html
    where
        E: AsHtml,
    {
        let editor = editor.as_html();
        let button = IconButton::new(self.sub_id, Icon::Minus).as_html();
        Div::new(vec![editor, button])
            .with_layout(Layout::Row)
            .as_html()
    }

    fn add_button(id: Id) -> Html {
        IconButton::new(id, Icon::Plus).as_html()
    }
}

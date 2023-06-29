use gtk::prelude::*;
use lemmy_api_common::lemmy_db_views::structs::CommentView;
use relm4::prelude::*;
use relm4_components::web_image::WebImage;

use crate::api;
use crate::dialogs::editor::DialogMsg;
use crate::dialogs::editor::EditorData;
use crate::dialogs::editor::EditorDialog;
use crate::dialogs::editor::EditorOutput;
use crate::dialogs::editor::EditorType;
use crate::settings;
use crate::util;
use crate::util::get_web_image_url;
use crate::util::markdown_to_pango_markup;

use super::post_page::PostInput;
use super::voting_row::VotingRowModel;
use super::voting_row::VotingStats;

pub struct CommentRow {
    pub comment: CommentView,
    avatar: Controller<WebImage>,
    voting_row: Controller<VotingRowModel>,
    comment_editor_dialog: Controller<EditorDialog>,
}

#[derive(Debug)]
pub enum CommentRowMsg {
    OpenPerson,
    DeleteComment,
    OpenEditCommentDialog,
    EditCommentRequest(EditorData),
    UpdateComment(CommentView),
}

#[relm4::factory(pub)]
impl FactoryComponent for CommentRow {
    type Init = CommentView;
    type Input = CommentRowMsg;
    type Output = PostInput;
    type CommandOutput = ();
    type Widgets = PostViewWidgets;
    type ParentInput = PostInput;
    type ParentWidget = gtk::Box;

    view! {
        root = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,
            set_margin_end: 10,
            set_margin_start: 10,
            set_margin_top: 10,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                set_vexpand: false,

                if self.comment.creator.avatar.is_some() {
                    gtk::Box {
                        set_hexpand: false,
                        #[local_ref]
                        community_image -> gtk::Box {}
                    }
                } else {
                    gtk::Box {}
                },

                gtk::Button {
                    set_label: &self.comment.creator.name,
                    connect_clicked => CommentRowMsg::OpenPerson,
                },

                gtk::Label {
                    set_margin_start: 10,
                    set_label: &util::format_elapsed_time(self.comment.comment.published),
                }
            },

            gtk::Label {
                #[watch]
               set_markup: &markdown_to_pango_markup(self.comment.comment.content.clone()),
               set_halign: gtk::Align::Start,
               set_use_markup: true,
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                #[local_ref]
                voting_row -> gtk::Box {},

                if self.comment.creator.id.0 == settings::get_current_account().id {
                    gtk::Button {
                        set_icon_name: "document-edit",
                        connect_clicked => CommentRowMsg::OpenEditCommentDialog,
                        set_margin_start: 5,
                    }
                } else {
                    gtk::Box {}
                },

                if self.comment.creator.id.0 == settings::get_current_account().id {
                    gtk::Button {
                        set_icon_name: "edit-delete",
                        connect_clicked => CommentRowMsg::DeleteComment,
                        set_margin_start: 10,
                    }
                } else {
                    gtk::Box {}
                }
            },

            gtk::Separator {}
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        Some(output)
    }

    fn init_model(value: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let avatar = WebImage::builder()
            .launch(get_web_image_url(value.creator.avatar.clone()))
            .detach();
        let voting_row = VotingRowModel::builder()
            .launch(VotingStats::from_comment(
                value.counts.clone(),
                value.my_vote,
            ))
            .detach();
        let comment_editor_dialog = EditorDialog::builder().launch(EditorType::Comment).forward(
            sender.input_sender(),
            |msg| match msg {
                EditorOutput::EditRequest(data, _) => CommentRowMsg::EditCommentRequest(data),
                _ => unreachable!(),
            },
        );

        Self {
            comment: value,
            avatar,
            voting_row,
            comment_editor_dialog,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let community_image = self.avatar.widget();
        let voting_row = self.voting_row.widget();
        let widgets = view_output!();
        widgets
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            CommentRowMsg::OpenPerson => {
                sender.output(PostInput::PassAppMessage(crate::AppMsg::OpenPerson(
                    self.comment.creator.id.clone(),
                )));
            }
            CommentRowMsg::DeleteComment => {
                let comment_id = self.comment.comment.id;
                std::thread::spawn(move || {
                    let _ = api::comment::delete_comment(comment_id);
                    let _ = sender.output(PostInput::PassAppMessage(
                        crate::AppMsg::StartFetchPosts(None, true),
                    ));
                });
            }
            CommentRowMsg::OpenEditCommentDialog => {
                let data = EditorData {
                    name: String::from(""),
                    body: self.comment.comment.content.clone(),
                    url: None,
                    id: Some(self.comment.comment.id.0),
                };
                let sender = self.comment_editor_dialog.sender();
                sender.emit(DialogMsg::UpdateData(data));
                sender.emit(DialogMsg::UpdateType(EditorType::Comment, false));
                sender.emit(DialogMsg::Show);
            }
            CommentRowMsg::UpdateComment(comment) => {
                self.comment = comment;
            }
            CommentRowMsg::EditCommentRequest(data) => {
                std::thread::spawn(move || {
                    let message = match api::comment::edit_comment(data.body, data.id.unwrap()) {
                        Ok(comment) => Some(CommentRowMsg::UpdateComment(comment.comment_view)),
                        Err(err) => {
                            println!("{}", err.to_string());
                            None
                        }
                    };
                    if let Some(message) = message {
                        sender.input(message)
                    };
                });
            }
        }
    }
}

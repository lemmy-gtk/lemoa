use lemmy_api_common::lemmy_db_schema::newtypes::DbUrl;
use relm4_components::web_image::WebImageMsg;

pub fn get_web_image_msg(url: Option<DbUrl>) -> WebImageMsg {
    return if let Some(url) = url {
        WebImageMsg::LoadImage(url.to_string())
    } else {
        WebImageMsg::Unload
    };
}

pub fn get_web_image_url(url: Option<DbUrl>) -> String {
    return if let Some(url) = url {
        url.to_string()
    } else {
        String::from("")
    };
}

pub fn markdown_to_pango_markup(text: String) -> String {
    return html2pango::markup_html(&markdown::to_html(&text)).unwrap_or(text);
}

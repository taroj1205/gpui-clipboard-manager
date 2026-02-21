use crate::clipboard::link_metadata::LinkMetadata;

pub struct ClipboardEntry {
    pub content_type: String,
    pub content_hash: String,
    pub content: String,
    pub text_content: Option<String>,
    pub ocr_text: Option<String>,
    pub image_path: Option<String>,
    pub file_paths: Option<String>,
    pub link_url: Option<String>,
    pub link_title: Option<String>,
    pub link_description: Option<String>,
    pub link_site_name: Option<String>,
    pub source_app_title: Option<String>,
    pub source_exe_path: Option<String>,
}

pub struct ClipboardEntryInput {
    pub content_type: String,
    pub content_hash: String,
    pub content: String,
    pub text_content: Option<String>,
    pub ocr_text: Option<String>,
    pub image_path: Option<String>,
    pub file_paths: Option<String>,
    pub link_metadata: Option<LinkMetadata>,
}

impl From<ClipboardEntryInput> for ClipboardEntry {
    fn from(input: ClipboardEntryInput) -> Self {
        let ClipboardEntryInput {
            content_type,
            content_hash,
            content,
            text_content,
            ocr_text,
            image_path,
            file_paths,
            link_metadata,
        } = input;

        let (link_url, link_title, link_description, link_site_name) = match link_metadata {
            Some(metadata) => (
                Some(metadata.url),
                metadata.title,
                metadata.description,
                metadata.site_name,
            ),
            None => (None, None, None, None),
        };

        ClipboardEntry {
            content_type,
            content_hash,
            content,
            text_content,
            ocr_text,
            image_path,
            file_paths,
            link_url,
            link_title,
            link_description,
            link_site_name,
            source_app_title: None,
            source_exe_path: None,
        }
    }
}

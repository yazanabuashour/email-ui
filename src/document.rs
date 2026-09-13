use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub title: String,
    pub heading: String,
    pub preheader: String,
    pub header: Header,
    pub footer: String,
    pub sections: Vec<Section>,
    #[serde(default)]
    pub notice: Option<Notice>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Header {
    pub edition: String,
    pub date: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Section {
    Cards {
        heading: String,
        rows: Vec<Row>,
    },
    Stories {
        heading: String,
        rows: Vec<Row>,
    },
    Schedule {
        heading: String,
        groups: Vec<ScheduleGroup>,
    },
    Summary {
        heading: String,
        lines: Vec<String>,
        #[serde(default)]
        links: Vec<Link>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub title: String,
    pub url: String,
    pub eyebrow: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScheduleGroup {
    pub heading: String,
    pub rows: Vec<ScheduleRow>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScheduleRow {
    pub weekday: String,
    pub day: String,
    pub month: String,
    pub title: String,
    pub url: String,
    pub metadata: String,
    #[serde(default)]
    pub images: Vec<Image>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Notice {
    pub label: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub label: String,
    pub url: String,
}

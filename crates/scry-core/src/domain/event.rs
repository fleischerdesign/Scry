use serde::Deserialize;
use ts_rs::TS;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams, TS)]
#[ts(export)]
pub struct ListParams {
    pub category: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Deserialize, IntoParams, TS)]
#[ts(export)]
pub struct SummaryParams {
    pub date: Option<String>,
}

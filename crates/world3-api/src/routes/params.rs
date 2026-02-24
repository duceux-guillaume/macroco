use axum::Json;
use world3_core::model::params::{parameter_descriptors, ParameterDescriptor};

pub async fn schema() -> Json<Vec<ParameterDescriptor>> {
    Json(parameter_descriptors())
}

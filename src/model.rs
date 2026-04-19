use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct BadRequestResponse {
    #[schema(example = "Invalid data provided")]
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct NotFoundResponse {
    #[schema(example = "Not found")]
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct InternalServerErrorResponse {
    #[schema(example = "Internal server error")]
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct UnauthorizedResponse {
    #[schema(example = "Unauthorized")]
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct TooManyRequestsResponse {
    #[schema(example = "Too many requests")]
    pub message: String,
}

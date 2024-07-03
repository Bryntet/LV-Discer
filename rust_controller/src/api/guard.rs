use crate::api::Coordinator;
use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::dto::CoordinatorBuilder;
use rocket::http::{Header, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::response::{status, Responder};
use rocket::serde::json::Json;
use rocket::{Request, State};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::{MediaType, Responses};
use rocket_okapi::okapi::schemars;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use rocket_okapi::{openapi, JsonSchema};
use std::ops::{Deref, DerefMut};
use tokio::sync::{Mutex, MutexGuard};

pub struct CoordinatorLoader(pub Mutex<Option<Coordinator>>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Coordinator {
    type Error = MyError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match &*request
            .rocket()
            .state::<CoordinatorLoader>()
            .unwrap()
            .0
            .lock()
            .await
        {
            None => Outcome::Error((
                Status::FailedDependency,
                MyError::UnloadedDependency("Coordinator has not been loaded yet"),
            )),
            Some(a) => Outcome::Success(a.clone()),
        }
    }
}

#[derive(Debug, Responder, Clone)]
pub enum MyError {
    #[response(status = 424)]
    UnloadedDependency(&'static str),
    #[response(status = 404)]
    IpNotFound(&'static str),
    PlayerNotFound(&'static str),
    #[response(status = 500)]
    UnableToParse(&'static str),
}

impl<'a> OpenApiFromRequest<'a> for Coordinator {
    fn from_request_input(
        gen: &mut OpenApiGenerator,
        name: String,
        required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(rocket_okapi::request::RequestHeaderInput::None)
    }
    fn get_responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        use rocket_okapi::{okapi, okapi::openapi3::RefOr};

        Ok(Responses {
            // Recommended and most strait forward.
            // And easy to add or remove new responses.
            responses: okapi::map! {
                "424".to_owned() => RefOr::Object(bad_request_response(gen)),
            },
            ..Default::default()
        })
    }
}

/// Create my custom response
///
/// Putting this in a separate function somewhere will resolve issues like
/// <https://github.com/GREsau/okapi/issues/57>
pub fn bad_request_response(gen: &mut OpenApiGenerator) -> rocket_okapi::okapi::openapi3::Response {
    use rocket_okapi::okapi;
    okapi::openapi3::Response {
        description: "\
        # 424 Failed Dependency\n\
        You have not loaded the internal services to start the coordinator. \
        "
        .to_owned(),
        content: okapi::map! {
            "application/json".to_owned() =>MediaType::default()
        },
        ..Default::default()
    }
}

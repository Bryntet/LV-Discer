use std::fmt::Debug;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::response::Responder;
use rocket::{response, Request};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::{MediaType, Responses};
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use rocket_okapi::response::OpenApiResponderInner;
use tokio::sync::Mutex;

use crate::api::websocket::ChannelAttributes;
use crate::api::{Coordinator, GeneralChannel};
use crate::controller::coordinator::FlipUpVMixCoordinator;

pub struct CoordinatorLoader(pub Mutex<Option<Coordinator>>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Coordinator {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match &*request
            .rocket()
            .state::<CoordinatorLoader>()
            .unwrap()
            .0
            .lock()
            .await
        {
            None => Outcome::Error((Status::FailedDependency, Error::UnloadedDependency)),
            Some(a) => Outcome::Success(a.clone()),
        }
    }
}

#[rocket::async_trait]
impl<'r, T: ChannelAttributes + 'static> FromRequest<'r> for GeneralChannel<T> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(
            request
                .rocket()
                .state::<GeneralChannel<T>>()
                .unwrap()
                .clone(),
        )
    }
}

impl<'r, T: ChannelAttributes + 'static> OpenApiFromRequest<'r> for GeneralChannel<T> {
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
                "424".to_owned() => RefOr::Object(failed_dependency(gen)),
            },
            ..Default::default()
        })
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Coordinator has not been loaded yet")]
    UnloadedDependency,
    #[error("IP: `{0}` not found")]
    IpNotFound(String),
    #[error("Player: `{0}` not found")]
    PlayerNotFound(String),
    #[error("Unable to parse")]
    UnableToParse,
    #[error("No score found on player: `{player}` at hole `{hole}`")]
    NoScoreFound { player: String, hole: usize },
    #[error("Index `{0}` does not exist in current card")]
    CardIndexNotFound(usize),
    #[error("Too many holes")]
    TooManyHoles,
    #[error("Length not found on the hole: {0}")]
    HoleLengthNotFound(u8),
    #[error("Par not found on the hole: {0}")]
    HoleParNotFound(u8),
    #[error("Not enough holes. Only {holes} found. Expected 18.")]
    NotEnoughHoles { holes: usize },
    #[error("Invalid division: \"{0}\"")]
    InvalidDivision(String),
    #[error("Player index {0} not found in focused card")]
    PlayerInCardNotFound(usize),
    #[error("Group not found")]
    GroupNotFound,
    #[error("Round not initialised")]
    RoundNotInitialised,
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'o> {
        // log `self` to your favored error tracker, e.g.
        // sentry::capture_error(&self);
        warn!("{}", self);

        use Error::*;
        match self {
            NoScoreFound { .. }
            | PlayerNotFound(_)
            | IpNotFound(_)
            | UnableToParse
            | HoleLengthNotFound(_)
            | HoleParNotFound(_)
            | NotEnoughHoles { .. }
            | GroupNotFound => Err(Status::InternalServerError),
            UnloadedDependency => Err(Status::FailedDependency),
            CardIndexNotFound(_) | TooManyHoles | InvalidDivision(_) | PlayerInCardNotFound(_) => {
                Err(Status::BadRequest)
            }
            RoundNotInitialised => Err(Status::FailedDependency),
        }
    }
}

impl OpenApiResponderInner for self::Error {
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        use rocket_okapi::{okapi, okapi::openapi3::RefOr};

        Ok(Responses {
            // Recommended and most strait forward.
            // And easy to add or remove new responses.
            responses: okapi::map! {
                "424".to_owned() => RefOr::Object(failed_dependency(gen)),
            },
            ..Default::default()
        })
    }
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
                "424".to_owned() => RefOr::Object(failed_dependency(gen)),
            },
            ..Default::default()
        })
    }
}

pub fn failed_dependency(gen: &mut OpenApiGenerator) -> rocket_okapi::okapi::openapi3::Response {
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

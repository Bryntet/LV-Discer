
use hyper::Client;
use cynic::{http::SurfExt};
extern crate hyper;
extern crate hyper_rustls;
extern crate google_sheets4 as sheets4;
use sheets4::api::ValueRange;
use sheets4::{Result, Error};
use std::default::Default;
use sheets4::{api::Response, Sheets, oauth2, };

async fn do_something() {
    
    
    // Get an ApplicationSecret instance by some means. It contains the `client_id` and 
    // `client_secret`, among other things.
    let secret = oauth2::read_application_secret("credentials.json")
        .await
        .expect("client secret not read");

    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk("tokencache.json")
    .build()
    .await
    .unwrap();
    // Instantiate the authenticator. It will choose a suitable authentication flow for you, 
    // unless you replace  `None` with the desired Flow.
    // Provide your own `AuthenticatorDelegate` to adjust the way it operates and get feedback about 
    // what's going on. You probably want to bring in your own `TokenStorage` to persist tokens and
    // retrieve them from storage.
    
    let mut hub = Sheets::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
    // As the method needs a request, you would usually fill it with the desired information
    // into the respective structure. Some of the parts shown here might not be applicable !
    // Values shown here are possibly random and not representative !
    let mut req = ValueRange::default();
    let values_to_write: Vec<Vec<String>> = vec![
        vec!["Hello".to_string(), "World".to_string()],
        vec!["Foo".to_string(), "Bar".to_string()],
    ];
    let value_range_object: ValueRange = ValueRange {
        major_dimension: Some("ROWS".to_string()),
        range: None,
        values: Some(values_to_write),
    };
    // You can configure optional parameters by calling the respective setters at will, and
    // execute the final call using `doit()`.
    // Values shown here are possibly random and not representative !
    let result = hub
                 .spreadsheets()
                 .values_update(
                    value_range_object,
                    "1HfGHHsuRZ7_ToIoBKxswPVXKvJbQxsia6aodBCrZrRw",
                    "Events!A1:B4"
                 )
                 .value_input_option("USER_ENTERED")
                 .doit().await;
    
    match result {
        Err(e) => match e {
        // The Error enum provides details about what exactly happened.
        // You can also just use its `Debug`, `Display` or `Error` traits
         Error::HttpError(_)
        |Error::Io(_)
        |Error::MissingAPIKey
        |Error::MissingToken(_)
        |Error::Cancelled
        |Error::UploadSizeLimitExceeded(_, _)
        |Error::Failure(_)
        |Error::BadRequest(_)
        |Error::FieldClash(_)
        |Error::JsonDecodeError(_, _) => println!("{}", e),
        },
        Ok(res) => println!("Success: {:?}", res),
    }
    
    
}

 


#[tokio::main]
async fn main() {
    use queries::*;
    use cynic::QueryBuilder;
    
    let operation = EventInfo::build(
        EventInfoVariables {
            event_id: "d9956c2f-78ff-42e5-bf97-6975009566d5".into(),
        }
    );
    let response = surf::post("https://api.tjing.se/graphql")
    .run_graphql(operation)
    .await
    .unwrap();
    let test = vec![cynic::query_dsl::query::<EventInfo>(EventInfoVariables { event_id: "d9956c2f-78ff-42e5-bf97-6975009566d5".to_string() })];

    println!("{:#?}\n{:#?}", response, test);

    do_something().await
}


use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_attribute]
pub fn comp(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = input.ident;
    let fields = input.data;

    let expanded = quote! {
        #input

        impl #name {
            fn fields() -> Vec<&'static str> {
                vec![#(stringify!(#fields)),*]
            }
        }
    };

    TokenStream::from(expanded)
}

#[comp]
struct Foo {
    bar: i32,
    baz: String,
}

#[cynic::schema_for_derives(
    file = r#"src/schema.graphql"#,
    module = "schema",
)]
mod queries {
    
    use super::schema;

    #[derive(cynic::QueryVariables, Debug)]
    pub struct PoolLBVariables {
        pub pool_id: cynic::Id,
    }
    
    #[derive(cynic::QueryVariables, Debug)]
    pub struct LivescoreVariables {
        pub pool_id: cynic::Id,
    }

    #[derive(cynic::QueryVariables, Debug)]
    pub struct EventInfoVariables {
        pub event_id: cynic::Id,
    }

    #[derive(cynic::QueryVariables, Debug)]
    pub struct EventResultVariables {
        pub event_id: cynic::Id,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "RootQuery", variables = "LivescoreVariables")]
    pub struct Livescore {
        #[arguments(poolId: $pool_id)]
        pub pool: Option<Pool>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "RootQuery", variables = "PoolLBVariables")]
    pub struct PoolLB {
        #[arguments(poolId: $pool_id)]
        pub pool: Option<Pool2>,
    }
    
    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "RootQuery", variables = "EventResultVariables")]
    pub struct EventResult {
        #[arguments(eventId: $event_id)]
        pub event: Option<Event>,
    }
    
    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "RootQuery", variables = "EventInfoVariables")]
    pub struct EventInfo {
        #[arguments(eventId: $event_id)]
        pub event: Option<Event2>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PoolLivescoreDivision {
        pub id: cynic::Id,
        pub name: String,
        pub players: Vec<PoolLivescorePlayer>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PoolLivescorePlayer {
        pub place: f64,
        pub first_name: String,
        pub last_name: String,
        pub pdga_number: Option<f64>,
        pub starts_at: Option<DateTime>,
        pub is_dnf: bool,
        pub is_dns: bool,
        pub total_par: Option<f64>,
        pub total_score: Option<f64>,
        pub results: Vec<PoolLivescoreResult>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PoolLivescoreResult {
        pub id: cynic::Id,
        pub score: f64,
        pub is_circle_hit: bool,
        pub is_outside_putt: bool,
        pub is_inside_putt: bool,
        pub is_out_of_bounds: bool,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PoolLeaderboardDivision {
        pub id: cynic::Id,
        pub name: String,
        pub players: Vec<PoolLeaderboardPlayer>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PoolLeaderboardPlayer {
        pub place: f64,
        pub first_name: String,
        pub last_name: String,
        pub pdga_number: Option<f64>,
        pub is_dnf: bool,
        pub is_dns: bool,
        pub score: Option<f64>,
        pub par: Option<f64>,
        pub results: Vec<SimpleResult>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct SimpleResult {
        pub id: cynic::Id,
        pub score: f64,
        pub is_circle_hit: bool,
        pub is_outside_putt: bool,
        pub is_inside_putt: bool,
        pub is_out_of_bounds: bool,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Pool {
        pub id: cynic::Id,
        pub date: DateTime,
        pub status: PoolStatus,
        pub layout_version: LayoutVersion,
        pub livescore: Option<Vec<PoolLivescoreDivisionCombined>>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Pool")]
    pub struct Pool2 {
        pub id: cynic::Id,
        pub date: DateTime,
        pub status: PoolStatus,
        pub layout_version: LayoutVersion,
        pub leaderboard: Option<Vec<Option<PoolLeaderboardDivisionCombined>>>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct LayoutVersion {
        pub id: cynic::Id,
        pub holes: Vec<Hole>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Hole {
        pub number: f64,
        pub par: Option<f64>,
        pub name: Option<String>,
        pub length: Option<f64>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct EventLeaderboardDivision {
        pub id: cynic::Id,
        pub name: String,
        #[cynic(rename = "type")]
        pub type_: String,
        pub players: Vec<EventLeaderboardPlayer>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct EventLeaderboardPlayer {
        pub first_name: String,
        pub last_name: String,
        pub pdga_number: Option<f64>,
        pub pdga_rating: Option<f64>,
        pub place: f64,
        pub score: Option<f64>,
        pub par: Option<f64>,
        pub pool_leaderboards: Vec<EventLeaderboardPool>,
        pub is_dnf: bool,
        pub is_dns: bool,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct EventLeaderboardPool {
        pub place: f64,
        pub score: Option<f64>,
        pub points: Option<f64>,
        pub pool_id: cynic::Id,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Event {
        pub leaderboard: Option<Vec<Option<EventLeaderboardDivisionCombined>>>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Event")]
    pub struct Event2 {
        pub id: cynic::Id,
        pub name: String,
        pub rounds: Vec<Option<Round>>,
        pub players: Vec<Player>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Player {
        pub user: User,
        pub division: Division,
        pub dnf: DNF,
        pub dns: DNS,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct User {
        pub id: cynic::Id,
        pub first_name: Option<String>,
        pub last_name: Option<String>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Round {
        pub pools: Vec<Pool3>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Pool")]
    pub struct Pool3 {
        pub date: DateTime,
        pub id: cynic::Id,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Division {
        pub id: cynic::Id,
        #[cynic(rename = "type")]
        pub type_: String,
        pub name: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct DNS {
        pub is_dns: bool,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct DNF {
        pub is_dnf: bool,
    }

    #[derive(cynic::InlineFragments, Debug)]
    pub enum EventLeaderboardDivisionCombined {
        EventLeaderboardDivision(EventLeaderboardDivision),
        #[cynic(fallback)]
        Unknown
    }

    #[derive(cynic::InlineFragments, Debug)]
    pub enum PoolLeaderboardDivisionCombined {
        PoolLeaderboardDivision(PoolLeaderboardDivision),
        #[cynic(fallback)]
        Unknown
    }

    #[derive(cynic::InlineFragments, Debug)]
    pub enum PoolLivescoreDivisionCombined {
        PoolLivescoreDivision(PoolLivescoreDivision),
        #[cynic(fallback)]
        Unknown
    }

    #[derive(cynic::Enum, Clone, Copy, Debug)]
    pub enum PoolStatus {
        Closed,
        Prepare,
        Open,
        Completed,
    }

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct DateTime(pub String);

}

#[allow(non_snake_case, non_camel_case_types)]
mod schema {
    cynic::use_schema!(r#"src/schema.graphql"#);
}
#[cynic::schema_for_derives(
    file = r#"src/schema.graphql"#,
    module = "schema",
)]

mod queries {
    use super::schema;

    #[derive(cynic::QueryVariables, Debug)]
    pub struct LivescoreVariables {
        pub pool_id: Option<cynic::Id>,
    }

    #[derive(cynic::QueryVariables, Debug)]
    pub struct EventResultVariables {
        pub event_id: Option<cynic::Id>,
    }

    #[derive(cynic::QueryVariables, Debug)]
    pub struct PoolLBVariables {
        pub pool_id: Option<cynic::Id>,
    }

    #[derive(cynic::QueryVariables, Debug)]
    pub struct EventInfoVariables {
        pub event_id: Option<cynic::Id>,
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


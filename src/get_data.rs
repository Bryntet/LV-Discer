use cynic::{http::SurfExt};
extern crate hyper;

pub async fn request_tjing(pool_id: cynic::Id) -> std::result::Result<cynic::GraphQlResponse<queries::StatusPool>, surf::Error> {
    use queries::*;
    use cynic::QueryBuilder;
    
    let operation = StatusPool::build(
        StatusPoolVariables {
            pool_id: pool_id.clone(),
        }
    );
    return surf::post("https://api.tjing.se/graphql")
    .run_graphql(operation)
    .await;
}

pub async fn post_status(pool_id: cynic::Id) -> Result<cynic::GraphQlResponse<queries::PoolLBAfter, serde::de::IgnoredAny>, surf::Error> {
    use queries::*;
    use cynic::QueryBuilder;
   
    let operation = PoolLBAfter::build(
        PoolLBAfterVariables {
            pool_id: pool_id,
        }
    );
    return surf::post("https://api.tjing.se/graphql")
    .run_graphql(operation)
    .await;
}



#[cynic::schema_for_derives(
    file = r#"src/schema.graphql"#,
    module = "schema",
)]
pub mod queries {
    
    use super::schema;

    #[derive(cynic::QueryVariables, Debug)]
    pub struct StatusPoolVariables {
        pub pool_id: cynic::Id,
    }

    #[derive(cynic::QueryVariables, Debug)]
    pub struct PoolLBAfterVariables {
        pub pool_id: cynic::Id,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "RootQuery", variables = "PoolLBAfterVariables")]
    pub struct PoolLBAfter {
        #[arguments(poolId: $pool_id)]
        pub pool: Option<Pool>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "RootQuery", variables = "StatusPoolVariables")]
    pub struct StatusPool {
        #[arguments(poolId: $pool_id)]
        pub pool: Option<Pool2>,
    }

    #[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct PoolLeaderboardDivision {
        pub id: cynic::Id,
        pub name: String,
        pub players: Vec<PoolLeaderboardPlayer>,
        #[cynic(rename = "type")]
        pub type_: String,
    }

    #[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct PoolLeaderboardPlayer {
        pub first_name: String,
        pub is_dnf: bool,
        pub is_dns: bool,
        pub last_name: String,
        pub par: Option<f64>,
        pub pdga_number: Option<f64>,
        pub pdga_rating: Option<f64>,
        pub place: f64,
        pub player_id: cynic::Id,
        pub results: Vec<SimpleResult>,
        pub points: Option<f64>,
        pub score: Option<f64>,
    }

    #[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct SimpleResult {
        pub hole: Hole,
        pub score: f64,
        pub is_circle_hit: bool,
        pub is_inside_putt: bool,
        pub is_out_of_bounds: bool,
        pub is_outside_putt: bool,
    }
    impl SimpleResult {
        pub fn get_score_colour(&self) -> &str {
            match self.score as i64 {
                3 => "AB8E77",
                2 => "CA988D",
                1 => "EC928F",
                0 => "7E8490",
                -1 => "A6F8BB",
                -2 => "6A8BE7",
                -3 => "DD6AC9",
                _ => "000000",
            }
        }
    }
    #[derive(cynic::QueryFragment, Debug)]
    pub struct Pool {
        pub status: PoolStatus,
        pub layout_version: LayoutVersion,
        pub leaderboard: Option<Vec<Option<PoolLeaderboardDivisionCombined>>>,
        pub position: f64,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Pool")]
    pub struct Pool2 {
        pub status: PoolStatus,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct LayoutVersion {
        pub holes: Vec<Hole2>,
    }

    #[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct Hole {
        pub par: Option<f64>,
        pub number: f64,
        pub length: Option<f64>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Hole")]
    pub struct Hole2 {
        pub number: f64,
        pub par: Option<f64>,
        pub length: Option<f64>,
    }

    #[derive(cynic::InlineFragments, Debug)]
    pub enum PoolLeaderboardDivisionCombined {
        PoolLeaderboardDivision(PoolLeaderboardDivision),
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

}
#[allow(non_snake_case, non_camel_case_types)]
mod schema {
    cynic::use_schema!(r#"src/schema.graphql"#);
}
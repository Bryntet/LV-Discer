use cynic::{http::SurfExt};
extern crate hyper;
extern crate hyper_rustls;
extern crate google_sheets4 as sheets4;
use sheets4::api::ValueRange;
use sheets4::{Error};
use sheets4::{Sheets, oauth2};
use std::io;

async fn get_auth() -> oauth2::authenticator::Authenticator<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>> {
    let secret = oauth2::read_application_secret("src/credentials.json")
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
    auth
}


async fn do_something(value_range_object: ValueRange, range: &str, auth: oauth2::authenticator::Authenticator<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>) {
    
    let hub = Sheets::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
    
    // You can configure optional parameters by calling the respective setters at will, and
    // execute the final call using `doit()`.
    // Values shown here are possibly random and not representative !
    let result = hub
                 .spreadsheets()
                 .values_update(
                    value_range_object,
                    "1HfGHHsuRZ7_ToIoBKxswPVXKvJbQxsia6aodBCrZrRw",
                    range
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
        Ok(_res) => println!("Success!"),
    }
    
    
}

 


#[tokio::main]
async fn main() {
    use queries::*;
    use cynic::QueryBuilder;
    use cynic::Id;
    let mut input = String::new();
    println!("Enter pool id:");
    io::stdin().read_line(&mut input).expect("Error!");
    let pool_id = Id::new(input.trim());
    
    let operation = StatusPool::build(
        StatusPoolVariables {
            pool_id: pool_id.clone(),
        }
    );
    let response = surf::post("https://api.tjing.se/graphql")
    .run_graphql(operation)
    .await
    .unwrap();
    
    if let Some(data) = response.data {
        if let Some(pool) = data.pool {
            match pool.status {
                PoolStatus::Completed => post_status(pool_id).await,
                _ => println!("Status is not completed")
            }
        }
    } 
    if let Some(_err) = response.errors {
        println!("Got error, probably invalid pool id");
    }
}

async fn post_status(pool_id: cynic::Id) {
    use queries::*;
    use cynic::QueryBuilder;
   
    let operation = PoolLBAfter::build(
        PoolLBAfterVariables {
            pool_id: pool_id,
        }
    );
    let response = surf::post("https://api.tjing.se/graphql")
    .run_graphql(operation)
    .await
    .unwrap();

    if let Some(data) = response.data {
        if let Some(pool) = data.pool {
            let mut hole_vec: Vec<Vec<String>> = Vec::new();
            let holes_amount = pool.layout_version.holes.len();
            for hole in pool.layout_version.holes {
                let number = format!("{}", hole.number as i8);
                let mut par = String::new();
                let mut lent = String::new();
                if let Some(parr) = hole.par { 
                    par = format!("{}", parr as i8);
                }
                if let Some(leng) = hole.length { 
                    lent = format!("{}", leng as i8);
                }
                
                hole_vec.push(vec![number, par, lent]);
            }
            println!("{:?}", hole_vec);
            do_something(ValueRange {
                major_dimension: Some("COLUMNS".to_string()),
                range: None,
                values: Some(hole_vec),
            }, "Bladet!E1:Z1000", get_auth().await).await;
            if let Some(lb) = pool.leaderboard {
                
                let mut good_divs: Vec<PoolLeaderboardDivision> = Vec::new();
                for division in lb {
                    let new_div = division.unwrap();

                    match new_div {
                        PoolLeaderboardDivisionCombined::PoolLeaderboardDivision(test) => good_divs.push(test),
                        _ => println!("fuck")
                    }
                    //println!("DIVISION: {:#?}", test)
                    
                }
                let mut start_row: u16 = 4;
                for div in good_divs {
                    
                    let mut player_vec: Vec<Vec<String>> = Vec::new();
                    for player in div.players {
                        let mut par = 100;
                        if let Some(parr) = player.par {
                            par = parr as u8
                        } 

                        let place = format!("{}", player.place as u16);
                        let mut personal_vec: Vec<String> = vec![
                                String::from(player.first_name.chars().collect::<Vec<_>>()[0]) + &". ".to_string() + &player.last_name, 
                                place,
                                format!("{}", par),
                                holes_amount.to_string()
                            ];
                        
                        let mut ob_s = Vec::from(vec!["OB:".to_string(), "".to_string(), "".to_string(), "".to_string()]);
                        let mut circle_hits = Vec::from(vec!["Circle hits:".to_string(), "".to_string(), "".to_string(), "".to_string()]);
                        let mut inside_putts = Vec::from(vec!["Inside putts:".to_string(), "".to_string(), "".to_string(), "".to_string()]);
                        let mut outside_putts = Vec::from(vec!["Outside putts::".to_string(), "".to_string(), "".to_string(), "".to_string()]);
                        for result in player.results {
                            personal_vec.push(result.score.to_string());
                            ob_s.push((result.is_out_of_bounds as u8).to_string());
                            circle_hits.push((result.is_circle_hit as u8).to_string());
                            inside_putts.push((result.is_inside_putt as u8).to_string());
                            outside_putts.push((result.is_outside_putt as u8).to_string());
                        }
                        personal_vec.push(player.is_dnf.to_string());
                        personal_vec.push(player.is_dns.to_string());
                        personal_vec.push(player.first_name);
                        personal_vec.push(player.last_name);
                        if let Some(par) = player.par {
                            personal_vec.push(par.to_string());
                        } else {
                            personal_vec.push("".to_string());
                        }
                        if let Some(pdga) = player.pdga_number {
                            personal_vec.push(pdga.to_string());
                        } else {
                            personal_vec.push("".to_string());
                        }
                        if let Some(rating) = player.pdga_rating {
                            personal_vec.push(rating.to_string());
                        } else {
                            personal_vec.push("".to_string());
                        }
                        
                        personal_vec.push(player.place.to_string());
                        personal_vec.push("issue".to_string());
                        
                        if let Some(points) = player.points {
                            personal_vec.push(points.to_string());
                        } else {
                            personal_vec.push("".to_string());
                        }
                        if let Some(score) = player.score {
                            personal_vec.push(score.to_string());
                        } else {
                            personal_vec.push("".to_string());
                        }
                        
                        player_vec.push(personal_vec);
                        player_vec.push(ob_s);
                        player_vec.push(circle_hits);
                        player_vec.push(inside_putts);
                        player_vec.push(outside_putts);

                    }
                    do_something(ValueRange {
                        major_dimension: Some("ROWS".to_string()),
                        range: None,
                        values: Some(vec![vec![div.name.clone()]]),
                    }, &("Bladet!A".to_owned() + &start_row.to_string() + ":AG10000"), get_auth().await).await;
                    
                    do_something(ValueRange {
                        major_dimension: Some("ROWS".to_string()),
                        range: None,
                        values: Some(player_vec),
                    }, &("Bladet!A".to_owned() + &(start_row+1).to_string() + ":AG10000"), get_auth().await).await;
                    start_row += 400
                    //println!("{:#?}", div_vec)
                    
                }   
            }
        }
    }
}



#[cynic::schema_for_derives(
    file = r#"src/schema.graphql"#,
    module = "schema",
)]
mod queries {
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

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PoolLeaderboardDivision {
        pub id: cynic::Id,
        pub name: String,
        pub players: Vec<PoolLeaderboardPlayer>,
        #[cynic(rename = "type")]
        pub type_: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
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

    #[derive(cynic::QueryFragment, Debug)]
    pub struct SimpleResult {
        pub hole: Hole,
        pub score: f64,
        pub is_circle_hit: bool,
        pub is_inside_putt: bool,
        pub is_out_of_bounds: bool,
        pub is_outside_putt: bool,
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

    #[derive(cynic::QueryFragment, Debug)]
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